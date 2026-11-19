import { type FormEvent, useState } from "react";
import { ChatBubble, cx, Mascot, SendIcon, subjectById, ToggleChip } from "../../ui";
import { AnswerBubble } from "./AnswerBubble";
import type { SubjectFilter, TanyaView } from "./types";

export type TanyaScreenProps = {
  readonly view: TanyaView;
  readonly onFilterChange: (filter: SubjectFilter) => void;
  readonly onSend: (question: string) => void;
  readonly onFollowUp: (followUpId: string) => void;
};

function FilterChips({ view, onFilterChange }: Pick<TanyaScreenProps, "view" | "onFilterChange">) {
  const all = { id: "semua", shortName: "Semua", color: "#0069a8", tint: "#e0f0fd" };
  const chips = [
    all,
    ...view.filterSubjects.map((id) => subjectById(id) ?? undefined).filter((s) => s !== undefined),
  ];
  return (
    <fieldset className="m-0 flex min-w-0 flex-wrap items-center gap-2 border-0 p-0">
      <legend className="float-left mr-2 p-0 text-[15px] font-bold text-ink-muted">
        Tanya tentang:
      </legend>
      {chips.map((s) => (
        <ToggleChip
          key={s.id}
          pressed={view.filter === s.id}
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

export function TanyaScreen({ view, onFilterChange, onSend, onFollowUp }: TanyaScreenProps) {
  const [draft, setDraft] = useState("");
  const submit = (e: FormEvent) => {
    e.preventDefault();
    const q = draft.trim();
    if (q.length === 0 || view.busy) return;
    onSend(q);
    setDraft("");
  };
  const last = view.messages.at(-1);
  const showFollowUps =
    last?.kind === "answer" && last.status === "done" && view.followUps.length > 0;
  return (
    <div className="mx-auto flex w-full max-w-[860px] flex-1 flex-col gap-[18px]">
      <FilterChips view={view} onFilterChange={onFilterChange} />
      {view.messages.length === 0 ? (
        <EmptyState suggestions={view.suggestions} onSend={onSend} />
      ) : (
        <>
          <h1 className="sr-only">Percakapan Tanya</h1>
          {view.messages.map((m) => {
            if (m.kind === "question")
              return (
                <ChatBubble key={m.id} from="student">
                  {m.text}
                </ChatBubble>
              );
            if (m.kind === "answer") return <AnswerBubble key={m.id} answer={m} />;
            return (
              <ChatBubble key={m.id} from="mascot" mood="hmm" variant="ungrounded">
                <span className="text-[17px] font-extrabold">
                  Hmm, itu tidak ada di buku pelajaranmu.
                </span>
                <span className="text-[16px] font-semibold leading-relaxed text-ink-muted">
                  Aku cuma menjawab dari buku {m.gradeLabel}, supaya jawabanku pasti benar. Coba
                  tanya soal pelajaran, ya!
                </span>
              </ChatBubble>
            );
          })}
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
      )}
      <form
        onSubmit={submit}
        className="sticky bottom-[calc(76px+env(safe-area-inset-bottom))] mt-auto bg-[linear-gradient(to_top,#f3f9ff_75%,rgb(243_249_255/0))] pt-3.5 pb-3 md:bottom-0 md:pb-[22px]"
      >
        <div className="ring-within flex items-center gap-2.5 rounded-[20px] border-2 border-line-sky bg-white py-2 pr-2 pl-4 shadow-[0_6px_20px_rgb(0_105_168/0.08)] sm:pl-5">
          <label htmlFor="tanya-input" className="sr-only">
            Pertanyaanmu
          </label>
          <input
            id="tanya-input"
            type="text"
            value={draft}
            onChange={(e) => setDraft(e.target.value)}
            placeholder="Tulis pertanyaanmu di sini…"
            className="min-h-[52px] min-w-0 flex-1 border-0 bg-transparent text-[18px] font-semibold placeholder:text-ink-faint focus-visible:outline-none"
          />
          <button
            type="submit"
            disabled={view.busy}
            className={cx(
              "pressable inline-flex min-h-[52px] min-w-[52px] items-center justify-center gap-2 rounded-[14px] bg-primary px-4 text-[19px] font-extrabold text-white shadow-[0_4px_0_var(--color-primary-edge)] sm:px-[22px]",
              view.busy
                ? "cursor-not-allowed opacity-55"
                : "cursor-pointer active:translate-y-[4px] active:shadow-none",
            )}
          >
            <SendIcon size={20} />
            <span className="max-sm:sr-only">Kirim</span>
          </button>
        </div>
      </form>
    </div>
  );
}
