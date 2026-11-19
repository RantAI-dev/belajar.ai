import { cx } from "./cx";
import { BangIcon, CheckIcon } from "./icons";

/** `idle`/`selected` while answering; `correct`, `wrong` (chosen but not right) and `missed` (right
 * but not chosen) after the answer is revealed. */
export type ChoiceState = "idle" | "selected" | "correct" | "wrong" | "missed";

const LOOK: Record<ChoiceState, string> = {
  idle: "border-line-strong bg-white hover:border-line-sky hover:bg-page",
  selected: "border-primary bg-primary-soft",
  correct: "border-success bg-success-soft",
  wrong: "border-oops-line bg-oops-soft",
  missed: "border-success border-dashed bg-white",
};

const TAG: Record<ChoiceState, { readonly text: string; readonly className: string } | null> = {
  idle: null,
  selected: null,
  correct: { text: "Jawaban benar", className: "text-success-edge" },
  wrong: { text: "Pilihanmu", className: "text-oops-ink" },
  missed: { text: "Jawaban benar", className: "text-success-edge" },
};

export type ChoiceButtonProps = {
  /** "A", "B", ... */
  readonly letter: string;
  readonly text: string;
  readonly state: ChoiceState;
  /** Square marker for "pick more than one", round for "pick one". */
  readonly multiple?: boolean;
  readonly onChoose?: () => void;
  /** After the reveal the options only show the result. */
  readonly locked?: boolean;
};

export function ChoiceButton({
  letter,
  text,
  state,
  multiple = false,
  onChoose,
  locked = false,
}: ChoiceButtonProps) {
  const pressed = state === "selected" || state === "correct" || state === "wrong";
  const marker = cx(
    "flex size-9 shrink-0 items-center justify-center border-[3px] font-display text-[18px] font-semibold",
    multiple ? "rounded-[10px]" : "rounded-full",
    state === "selected" && "border-primary bg-primary text-white",
    state === "correct" && "border-success bg-success text-white",
    state === "wrong" && "border-oops bg-oops text-white",
    (state === "idle" || state === "missed") && "border-line-strong bg-white text-ink-muted",
  );
  return (
    <button
      type="button"
      aria-pressed={pressed}
      aria-disabled={locked || undefined}
      onClick={locked ? undefined : onChoose}
      className={cx(
        "pressable flex min-h-14 w-full items-center gap-3 rounded-[16px] border-[3px] px-3 py-2 text-left text-[17px] font-bold text-ink",
        LOOK[state],
        locked ? "cursor-default" : "cursor-pointer",
      )}
    >
      <span aria-hidden="true" className={marker}>
        {state === "correct" ? (
          <CheckIcon size={18} />
        ) : state === "wrong" ? (
          <BangIcon size={18} />
        ) : (
          letter
        )}
      </span>
      <span className="flex-1">
        <span className="sr-only">{letter}. </span>
        {text}
      </span>
      {TAG[state] !== null && (
        <span className={cx("shrink-0 text-[13px] font-extrabold", TAG[state]?.className)}>
          <span className="sr-only">(</span>
          {TAG[state]?.text}
          <span className="sr-only">)</span>
        </span>
      )}
    </button>
  );
}

/** The state of one option, given what was chosen and, once revealed, what is right. */
export const choiceState = (
  optionId: string,
  chosen: readonly string[],
  correct: readonly string[] | null,
): ChoiceState => {
  const picked = chosen.includes(optionId);
  if (correct === null) return picked ? "selected" : "idle";
  const right = correct.includes(optionId);
  if (picked) return right ? "correct" : "wrong";
  return right ? "missed" : "idle";
};

export const LETTERS = ["A", "B", "C", "D", "E", "F"] as const;
export const letterAt = (i: number): string => LETTERS[i] ?? String(i + 1);
