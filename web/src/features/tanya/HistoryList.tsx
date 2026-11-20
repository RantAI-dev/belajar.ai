import { AppLink, Button, ChatIcon, subjectById } from "../../ui";
import type { ConversationSummary } from "./types";

export type HistoryView = {
  readonly items: readonly ConversationSummary[];
  readonly hasMore: boolean;
  readonly loading: boolean;
  /** A friendly sentence when the list could not be loaded. */
  readonly error: string | null;
};

/** "Percakapanmu": earlier questions, newest first, with "Muat lagi" for older ones. */
export function HistoryList({
  view,
  onLoadMore,
}: {
  readonly view: HistoryView;
  readonly onLoadMore: () => void;
}) {
  if (view.items.length === 0 && view.error === null) {
    if (view.loading) return null;
    return (
      <p className="m-0 text-center text-[16px] font-semibold text-ink-muted">
        Belum ada percakapan. Pertanyaan pertamamu akan muncul di sini.
      </p>
    );
  }
  return (
    <section aria-labelledby="riwayat-title" className="flex flex-col gap-2.5">
      <h2 id="riwayat-title" className="m-0 font-display text-[22px] font-semibold">
        Percakapanmu
      </h2>
      <ul className="m-0 flex list-none flex-col gap-2 p-0">
        {view.items.map((c) => {
          const subject = c.subjectId === null ? undefined : subjectById(c.subjectId);
          return (
            <li key={c.id}>
              <AppLink
                href={c.href}
                className="flex min-h-[60px] items-center gap-3 rounded-[16px] border-2 border-line bg-surface px-4 py-2 text-ink hover:bg-primary-soft"
              >
                <span className="text-primary-ink" aria-hidden="true">
                  <ChatIcon size={22} />
                </span>
                <span className="min-w-0 flex-1 truncate text-[17px] font-bold">{c.title}</span>
                <span className="shrink-0 text-[14px] font-bold text-ink-muted">
                  {subject === undefined ? "" : `${subject.shortName} · `}
                  {c.dateLabel}
                </span>
              </AppLink>
            </li>
          );
        })}
      </ul>
      {view.error !== null && (
        <p role="alert" className="m-0 text-[16px] font-bold text-oops-ink">
          {view.error}
        </p>
      )}
      {(view.hasMore || view.error !== null) && (
        <Button
          variant="secondary"
          loading={view.loading}
          loadingLabel="Memuat…"
          onClick={onLoadMore}
        >
          {view.error !== null ? "Coba lagi" : "Muat lagi"}
        </Button>
      )}
    </section>
  );
}
