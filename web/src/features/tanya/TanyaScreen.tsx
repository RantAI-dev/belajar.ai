import { ErrorState, LoadingState, Mascot, subjectById, ToggleChip } from "../../ui";
import { Composer } from "./Composer";
import { ConversationTitle, type ConversationTitleProps } from "./ConversationTitle";
import { HistoryList, type HistoryView } from "./HistoryList";
import { ChatThread } from "./Thread";
import type { SubjectFilter, TanyaView } from "./types";

export type TanyaScreenProps = {
  readonly view: TanyaView;
  readonly onFilterChange: (filter: SubjectFilter) => void;
  readonly onSend: (question: string) => void;
  readonly onFollowUp: (followUpId: string) => void;
  /** Inside a conversation: its name, rename and the way back to the list. */
  readonly conversation?: ConversationTitleProps;
  /** On a new conversation: the earlier ones. */
  readonly history?: { readonly view: HistoryView; readonly onLoadMore: () => void };
  /** The conversation's messages are still loading. */
  readonly loading?: boolean;
  /** The conversation could not be loaded. */
  readonly loadError?: { readonly message: string; readonly onRetry: () => void };
};

function FilterChips({
  filter,
  subjects,
  onFilterChange,
}: {
  readonly filter: SubjectFilter;
  readonly subjects: readonly string[];
  readonly onFilterChange: (filter: SubjectFilter) => void;
}) {
  const all = { id: "semua", shortName: "Semua", color: "#0069a8", tint: "#e0f0fd" };
  const chips = [
    all,
    ...subjects.map((id) => subjectById(id) ?? undefined).filter((s) => s !== undefined),
  ];
  return (
    <fieldset className="m-0 flex min-w-0 flex-wrap items-center gap-2 border-0 p-0">
      <legend className="float-left mr-2 p-0 text-[15px] font-bold text-ink-muted">
        Tanya tentang:
      </legend>
      {chips.map((s) => (
        <ToggleChip
          key={s.id}
          pressed={filter === s.id}
          onPressedChange={() => onFilterChange(s.id)}
          accent={{ color: s.color, tint: s.tint }}
        >
          {s.shortName}
        </ToggleChip>
      ))}
    </fieldset>
  );
}

function EmptyState({
  suggestions,
  onSend,
}: {
  readonly suggestions: readonly string[];
  readonly onSend: (q: string) => void;
}) {
  return (
    <div className="flex flex-col items-center gap-4 py-6 text-center">
      <Mascot mood="happy" size={120} animate />
      <h1 className="m-0 font-display text-[28px] font-semibold">Mau tanya apa hari ini?</h1>
      <p className="m-0 max-w-[460px] text-[17px] font-semibold text-ink-muted">
        Tanya apa saja tentang pelajaranmu. Aku jawab dari bukumu, lengkap dengan halamannya.
      </p>
      <ul className="m-0 flex list-none flex-wrap justify-center gap-2 p-0">
        {suggestions.map((s) => (
          <li key={s}>
            <button
              type="button"
              onClick={() => onSend(s)}
              className="inline-flex min-h-12 cursor-pointer items-center rounded-full border-2 border-line-sky bg-white px-4 text-[15px] font-extrabold text-primary hover:bg-primary-soft"
            >
              {s}
            </button>
          </li>
        ))}
      </ul>
    </div>
  );
}

export function TanyaScreen(props: TanyaScreenProps) {
  const { view, onFilterChange, onSend, onFollowUp, conversation, history } = props;
  const last = view.messages.at(-1);
  const showFollowUps =
    last?.kind === "answer" && last.status === "done" && view.followUps.length > 0;
  const body = () => {
    if (props.loadError !== undefined)
      return <ErrorState message={props.loadError.message} onRetry={props.loadError.onRetry} />;
    if (props.loading === true) return <LoadingState text="Sebentar, aku buka percakapanmu…" />;
    if (view.messages.length === 0)
      return (
        <>
          <EmptyState suggestions={view.suggestions} onSend={onSend} />
          {history !== undefined && (
            <HistoryList view={history.view} onLoadMore={history.onLoadMore} />
          )}
        </>
      );
    return (
      <>
        {conversation === undefined && <h1 className="sr-only">Percakapan Tanya</h1>}
        <ChatThread messages={view.messages} />
        {showFollowUps && (
          <div className="flex flex-wrap gap-2 sm:pl-[60px]">
            {view.followUps.map((f) => (
              <button
                key={f.id}
                type="button"
                onClick={() => onFollowUp(f.id)}
                className="inline-flex min-h-12 cursor-pointer items-center rounded-full border-2 border-line-sky bg-white px-4 text-[15px] font-extrabold text-primary hover:bg-primary-soft"
              >
                {f.label}
              </button>
            ))}
          </div>
        )}
      </>
    );
  };
  return (
    <div className="mx-auto flex w-full max-w-[860px] flex-1 flex-col gap-[18px]">
      {conversation !== undefined && <ConversationTitle {...conversation} />}
      {view.filter !== null && (
        <FilterChips
          filter={view.filter}
          subjects={view.filterSubjects}
          onFilterChange={onFilterChange}
        />
      )}
      {body()}
      <Composer busy={view.busy || props.loading === true} onSend={onSend} />
    </div>
  );
}
