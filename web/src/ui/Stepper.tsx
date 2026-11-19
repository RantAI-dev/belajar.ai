import { cx } from "./cx";
import { CheckIcon } from "./icons";

export type StepperProps = {
  readonly steps: readonly string[];
  /** Index of the step the student is on; earlier steps show as done. */
  readonly current: number;
  readonly color?: string;
  readonly edge?: string;
  readonly ink?: string;
  readonly className?: string;
};

/** Numbered dots joined by a line, e.g. Penjelasan -> Cek paham -> Kuis. */
export function Stepper({
  steps,
  current,
  color = "#0069a8",
  edge = "#c4dcf2",
  ink = "#00598a",
  className,
}: StepperProps) {
  return (
    <ol
      aria-label={`Langkah ${Math.min(current + 1, steps.length)} dari ${steps.length}`}
      className={cx("m-0 flex list-none items-center gap-2 p-0", className)}
    >
      {steps.map((label, i) => {
        const done = i < current;
        const on = i === current;
        return (
          <li
            key={label}
            className={cx("flex items-center gap-2", i < steps.length - 1 && "flex-1")}
          >
            <span
              className={cx(
                "flex shrink-0 items-center justify-center rounded-full font-extrabold",
                on ? "size-[42px] border-4 bg-white" : "size-[34px]",
                !done && !on && "border-[3px] bg-white text-ink-faint",
              )}
              style={
                done
                  ? { background: color, color: "#ffffff" }
                  : on
                    ? { borderColor: color, color: ink }
                    : { borderColor: edge }
              }
            >
              {done ? <CheckIcon size={18} /> : i + 1}
              <span className="sr-only">
                {" "}
                {label}
                {done ? " (selesai)" : on ? " (sekarang)" : ""}
              </span>
            </span>
            {i < steps.length - 1 && (
              <span
                aria-hidden="true"
                className="h-1.5 flex-1 rounded-full"
                style={{ background: done ? color : edge }}
              />
            )}
          </li>
        );
      })}
    </ol>
  );
}
