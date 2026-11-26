import { type RefObject, useEffect, useRef, useState } from "react";
import { pageAt, renderScale } from "./bookView";
import { CITE } from "./CitationPill";
import type { PdfDocument } from "./pdf";

export type PdfPagesProps = {
  readonly doc: PdfDocument;
  /** The 1-based page the source cites: scrolled to first, and marked. */
  readonly citedPage: number;
  /** The width every page is drawn at, in CSS pixels. */
  readonly pageWidth: number;
  /** The drawer's scrolling box: pages are drawn only near its view. */
  readonly scroller: RefObject<HTMLDivElement | null>;
  /** The page to scroll to; a new object scrolls again, even to the same page. */
  readonly goTo: { readonly page: number };
  readonly onPageChange: (page: number) => void;
};

type Size = { readonly width: number; readonly height: number };

/** Every page of a PDF, one under the other; only the pages near the view hold a canvas. */
export function PdfPages({
  doc,
  citedPage,
  pageWidth,
  scroller,
  goTo,
  onPageChange,
}: PdfPagesProps) {
  const [firstSize, setFirstSize] = useState<Size | null>(null);
  const [near, setNear] = useState<ReadonlySet<number>>(() => new Set());
  const slots = useRef<(HTMLDivElement | null)[]>([]);
  const count = doc.numPages;

  // Every placeholder takes the first page's shape until its own page is drawn.
  useEffect(() => {
    let live = true;
    void doc.getPage(1).then((page) => {
      const { width, height } = page.getViewport({ scale: 1 });
      if (live) setFirstSize({ width, height });
    });
    return () => {
      live = false;
    };
  }, [doc]);

  // Pages within a screen of the view are drawn; the rest give their canvas back.
  useEffect(() => {
    const root = scroller.current;
    if (root === null || firstSize === null) return;
    const observer = new IntersectionObserver(
      (entries) =>
        setNear((prev) => {
          const next = new Set(prev);
          for (const e of entries) {
            const n = Number((e.target as HTMLElement).dataset.page);
            if (e.isIntersecting) next.add(n);
            else next.delete(n);
          }
          return next;
        }),
      { root, rootMargin: "100% 0px" },
    );
    for (const slot of slots.current) if (slot !== null) observer.observe(slot);
    return () => observer.disconnect();
  }, [scroller, firstSize]);

  useEffect(() => {
    if (firstSize === null) return;
    slots.current[goTo.page - 1]?.scrollIntoView({ block: "start" });
  }, [firstSize, goTo]);

  // The page counter follows the scroll.
  useEffect(() => {
    const root = scroller.current;
    if (root === null) return;
    let frame = 0;
    const onScroll = () => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(() => {
        const tops = slots.current.map((s) => s?.offsetTop ?? Number.POSITIVE_INFINITY);
        onPageChange(pageAt(tops, root.scrollTop));
      });
    };
    root.addEventListener("scroll", onScroll, { passive: true });
    return () => {
      cancelAnimationFrame(frame);
      root.removeEventListener("scroll", onScroll);
    };
  }, [scroller, onPageChange]);

  const aspect = firstSize === null ? Math.SQRT2 : firstSize.height / firstSize.width;
  return (
    <div className="flex flex-col items-center gap-3 py-3">
      {/* scroll-mt: the cited page's outline and badge stay in view when it is scrolled to. */}
      {Array.from({ length: count }, (_, i) => {
        const n = i + 1;
        const cited = n === citedPage;
        return (
          <div
            // Pages are fixed for the life of the document: the number is the identity.
            key={n}
            ref={(el) => {
              slots.current[i] = el;
            }}
            data-page={n}
            className="relative shrink-0 scroll-mt-5 rounded-[6px] bg-paper shadow-[0_1px_6px_var(--color-sheet-shadow)]"
            style={{
              width: pageWidth,
              aspectRatio: `1 / ${aspect}`,
              outline: cited ? `3px solid ${CITE.line}` : undefined,
              outlineOffset: 3,
            }}
          >
            {near.has(n) && <PdfPage doc={doc} number={n} cssWidth={pageWidth} />}
            {cited && (
              <span
                className="absolute -top-3.5 left-3 rounded-full px-2.5 py-0.5 text-[13px] font-extrabold"
                style={{ background: CITE.strong, color: "var(--color-on-fill)" }}
              >
                Halaman yang dikutip
              </span>
            )}
          </div>
        );
      })}
    </div>
  );
}

function PdfPage({
  doc,
  number,
  cssWidth,
}: {
  readonly doc: PdfDocument;
  readonly number: number;
  readonly cssWidth: number;
}) {
  const canvas = useRef<HTMLCanvasElement>(null);
  useEffect(() => {
    let cancel = () => {};
    let live = true;
    void doc.getPage(number).then((page) => {
      const el = canvas.current;
      if (!live || el === null) return;
      const base = page.getViewport({ scale: 1 });
      const scale = renderScale(cssWidth, base.width, base.height / base.width, devicePixelRatio);
      const viewport = page.getViewport({ scale });
      el.width = Math.floor(viewport.width);
      el.height = Math.floor(viewport.height);
      const task = page.render({ canvas: el, viewport });
      cancel = () => task.cancel();
      // A cancelled render rejects; the page is simply gone.
      task.promise.catch(() => {});
    });
    return () => {
      live = false;
      cancel();
    };
  }, [doc, number, cssWidth]);
  return (
    <canvas
      ref={canvas}
      aria-label={`Halaman ${number}`}
      className="absolute inset-0 h-full w-full rounded-[6px]"
    />
  );
}
