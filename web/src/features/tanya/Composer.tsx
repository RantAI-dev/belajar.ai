import { type FormEvent, useState } from "react";
import { cx, SendIcon } from "../../ui";

/** The sticky "ask" box under a chat. */
export function Composer({
  busy,
  onSend,
  placeholder = "Tulis pertanyaanmu di sini…",
}: {
  readonly busy: boolean;
  readonly onSend: (text: string) => void;
  readonly placeholder?: string;
}) {
  const [draft, setDraft] = useState("");
  const submit = (e: FormEvent) => {
    e.preventDefault();
    const q = draft.trim();
    if (q.length === 0 || busy) return;
    onSend(q);
    setDraft("");
  };
  return (
    <form
      onSubmit={submit}
      className="sticky bottom-[calc(76px+env(safe-area-inset-bottom))] mt-auto bg-[linear-gradient(to_top,var(--color-page)_75%,transparent)] pt-3.5 pb-3 md:bottom-0 md:pb-[22px]"
    >
      <div className="ring-within flex items-center gap-2.5 rounded-[20px] border-2 border-line-sky bg-surface py-2 pr-2 pl-4 shadow-[0_6px_20px_color-mix(in_srgb,var(--color-primary)_8%,transparent)] sm:pl-5">
        <label htmlFor="tanya-input" className="sr-only">
          Pertanyaanmu
        </label>
        <input
          id="tanya-input"
          type="text"
          value={draft}
          maxLength={2000}
          onChange={(e) => setDraft(e.target.value)}
          placeholder={placeholder}
          className="min-h-[52px] min-w-0 flex-1 border-0 bg-transparent text-[18px] font-semibold placeholder:text-ink-faint focus-visible:outline-none"
        />
        <button
          type="submit"
          disabled={busy}
          className={cx(
            "pressable inline-flex min-h-[52px] min-w-[52px] items-center justify-center gap-2 rounded-[14px] bg-primary px-4 text-[19px] font-extrabold text-on-fill shadow-[0_4px_0_var(--color-primary-edge)] sm:px-[22px]",
            busy
              ? "cursor-not-allowed opacity-55"
              : "cursor-pointer active:translate-y-[4px] active:shadow-none",
          )}
        >
          <SendIcon size={20} />
          <span className="max-sm:sr-only">Kirim</span>
        </button>
      </div>
    </form>
  );
}
