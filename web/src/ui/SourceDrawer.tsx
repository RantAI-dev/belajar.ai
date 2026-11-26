import { type RefObject, useCallback, useEffect, useId, useRef, useState } from "react";
import { bookKind, clampPage, stepZoom, ZOOMS, type Zoom } from "./bookView";
import { CloseIcon, MinusIcon, PlusIcon, Spinner } from "./icons";
import { PdfPages } from "./PdfPages";
import { openPdf, type PdfDocument } from "./pdf";
import { ErrorState } from "./Status";

export type SourceDrawerProps = {
  /** The book, e.g. "Matematika Buku Siswa Kelas VII". */
  readonly title: string;
  /** e.g. "Bab 2: Bilangan Pecahan · halaman 44". */
  readonly subtitle?: string;
  /** The page image, or the whole book as a PDF: the response's type decides how it is shown. */
  readonly src: string;
  /** In a whole book, the 1-based page the source cites. */
  readonly page?: number;
  readonly onClose: () => void;
};

type Book =
  | { readonly kind: "loading" }
  | { readonly kind: "failed" }
  | { readonly kind: "image"; readonly url: string }
  | { readonly kind: "pdf"; readonly doc: PdfDocument };

/** Fetches `src` once and keeps it as a picture or an open PDF until the drawer closes. */
function useBook(src: string, attempt: number): Book {
  const [book, setBook] = useState<Book>({ kind: "loading" });
  useEffect(() => {
    const abort = new AbortController();
    let dispose = () => {};
    setBook({ kind: "loading" });
    const load = async (): Promise<Book> => {
      // A retry skips the browser cache: the copy there may be the one that failed.
      const cache = attempt > 0 ? "reload" : "default";
      const res = await fetch(src, { signal: abort.signal, credentials: "same-origin", cache });
      const kind = res.ok ? bookKind(res.headers.get("content-type")) : null;
      if (kind === null) return { kind: "failed" };
      if (kind === "image") {
        const url = URL.createObjectURL(await res.blob());
        dispose = () => URL.revokeObjectURL(url);
        return { kind: "image", url };
      }
      const doc = await openPdf(await res.arrayBuffer());
      dispose = () => void doc.loadingTask.destroy();
      return { kind: "pdf", doc };
    };
    load().then(
      (next) => {
        if (abort.signal.aborted) dispose();
        else setBook(next);
      },
      () => {
        if (!abort.signal.aborted) setBook({ kind: "failed" });
      },
    );
    return () => {
      abort.abort();
      dispose();
    };
  }, [src, attempt]);
  return book;
}

/** The drawer's inner width, so pages can be drawn to fit it. */
function useWidth(el: RefObject<HTMLElement | null>): number {
  const [width, setWidth] = useState(0);
  useEffect(() => {
    const node = el.current;
    if (node === null) return;
    const observer = new ResizeObserver(([entry]) => {
      if (entry !== undefined) setWidth(entry.contentRect.width);
    });
    observer.observe(node);
    return () => observer.disconnect();
  }, [el]);
  return width;
}

const iconButton =
  "pressable inline-flex size-12 shrink-0 cursor-pointer items-center justify-center rounded-[14px] border-2 border-line-strong bg-surface text-ink hover:bg-page disabled:cursor-default disabled:opacity-40";

/**
 * The book behind a source, in a drawer from the right (full width on a phone). A modal
 * `<dialog>`: the page behind is inert, Escape closes it, and focus returns to what opened it.
 * Mount it to open it; `onClose` asks the parent to unmount it.
 */
export function SourceDrawer({ title, subtitle, src, page, onClose }: SourceDrawerProps) {
  const dialog = useRef<HTMLDialogElement>(null);
  const scroller = useRef<HTMLDivElement>(null);
  const titleId = useId();
  const [attempt, setAttempt] = useState(0);
  const book = useBook(src, attempt);
  const [zoom, setZoom] = useState<Zoom>(ZOOMS[0]);
  const [current, setCurrent] = useState(1);
  const width = useWidth(scroller);
  const pageCount = book.kind === "pdf" ? book.doc.numPages : 0;
  const cited = clampPage(page, pageCount);
  const [goTo, setGoTo] = useState<{ readonly page: number }>({ page: cited });

  // No close() on unmount: a dialog taken out of the page leaves the top layer by itself, and a
  // close() there would fire `close` and ask the parent to unmount it again.
  useEffect(() => {
    const el = dialog.current;
    if (el !== null && !el.open) el.showModal();
  }, []);

  // Once the book is open its page count is known, and an impossible page falls back to page 1.
  useEffect(() => {
    if (book.kind === "pdf") {
      setGoTo({ page: cited });
      setCurrent(cited);
    }
  }, [book, cited]);

  const onPageChange = useCallback((n: number) => setCurrent(n), []);
  const zoomBy = (direction: 1 | -1) => {
    setZoom((z) => stepZoom(z, direction));
    // The page being read stays in view after the pages change size.
    if (book.kind === "pdf") setGoTo({ page: current });
  };
  // 16px of air on each side of a page.
  const pageWidth = Math.max(0, (width - 32) * zoom);

  return (
    <dialog
      ref={dialog}
      aria-labelledby={titleId}
      onClose={onClose}
      className="source-drawer m-0 h-dvh max-h-dvh w-full max-w-none overflow-hidden border-0 bg-transparent p-0 text-ink"
    >
      {/* The dimmed page: a click on it closes the drawer, as Escape does from the keyboard. */}
      <button
        type="button"
        tabIndex={-1}
        aria-hidden="true"
        onClick={() => dialog.current?.close()}
        className="absolute inset-0 cursor-default border-0 p-0"
        style={{ background: "var(--color-scrim)" }}
      />
      <div className="source-drawer-panel relative ml-auto flex h-full w-full max-w-[min(640px,100vw)] flex-col bg-surface shadow-[-8px_0_32px_var(--color-sheet-shadow)]">
        <header className="flex items-start gap-3 border-b-2 border-line px-4 py-3">
          <div className="flex min-w-0 flex-1 flex-col">
            <h2 id={titleId} className="m-0 text-[18px] font-extrabold leading-snug">
              {title}
            </h2>
            {subtitle !== undefined && (
              <span className="text-[15px] font-semibold text-ink-muted">{subtitle}</span>
            )}
          </div>
          <button
            type="button"
            className={iconButton}
            aria-label="Tutup buku"
            onClick={() => dialog.current?.close()}
          >
            <CloseIcon />
          </button>
        </header>

        {(book.kind === "pdf" || book.kind === "image") && (
          <div className="flex flex-wrap items-center gap-2 border-b-2 border-line bg-surface-alt px-4 py-2">
            <button
              type="button"
              className={iconButton}
              aria-label="Perkecil"
              disabled={zoom === ZOOMS[0]}
              onClick={() => zoomBy(-1)}
            >
              <MinusIcon />
            </button>
            <span
              className="min-w-[52px] text-center text-[15px] font-extrabold"
              aria-live="polite"
            >
              {Math.round(zoom * 100)}%
            </span>
            <button
              type="button"
              className={iconButton}
              aria-label="Perbesar"
              disabled={zoom === ZOOMS[ZOOMS.length - 1]}
              onClick={() => zoomBy(1)}
            >
              <PlusIcon />
            </button>
            {book.kind === "pdf" && (
              <>
                <span className="ml-auto text-[15px] font-bold text-ink-muted">
                  Hal. {current} dari {pageCount}
                </span>
                {current !== cited && (
                  <button
                    type="button"
                    className="pressable inline-flex min-h-12 cursor-pointer items-center rounded-[14px] border-2 border-line-sky bg-surface px-3.5 text-[15px] font-extrabold hover:bg-primary-soft"
                    onClick={() => setGoTo({ page: cited })}
                  >
                    Ke hal. {cited}
                  </button>
                )}
              </>
            )}
          </div>
        )}

        {/* `relative`: the pages' offsetTop is measured from here, which the page counter needs. */}
        <div ref={scroller} className="relative min-h-0 flex-1 overflow-auto bg-page px-4">
          {book.kind === "loading" && (
            <div
              role="status"
              className="flex flex-col items-center gap-3 py-16 text-[17px] font-bold text-ink-muted"
            >
              <Spinner size={32} />
              Sebentar, bukunya lagi dibuka…
            </div>
          )}
          {book.kind === "failed" && (
            <ErrorState
              message="Bukunya belum bisa dibuka. Coba lagi sebentar, ya."
              onRetry={() => setAttempt((n) => n + 1)}
            />
          )}
          {book.kind === "image" && (
            <div className="py-3">
              <img
                src={book.url}
                alt={`Halaman buku: ${title}`}
                className="mx-auto block max-w-none rounded-[6px] bg-paper shadow-[0_1px_6px_var(--color-sheet-shadow)]"
                style={{ width: pageWidth }}
              />
            </div>
          )}
          {book.kind === "pdf" && pageWidth > 0 && (
            <PdfPages
              doc={book.doc}
              citedPage={cited}
              pageWidth={pageWidth}
              scroller={scroller}
              goTo={goTo}
              onPageChange={onPageChange}
            />
          )}
        </div>
      </div>
    </dialog>
  );
}
