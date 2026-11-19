import { cx } from "./cx";
import { progressPercent } from "./progress";

export type ProgressBarProps = {
  readonly value: number;
  readonly max: number;
  readonly label: string;
  /** Read out instead of the bare numbers, e.g. "3 dari 5 soal siap". */
  readonly valueText?: string;
  readonly color?: string;
  readonly size?: "sm" | "lg";
  readonly className?: string;
};

export function ProgressBar({
  value,
  max,
  label,
  valueText,
  color = "#22a06b",
  size = "sm",
  className,
}: ProgressBarProps) {
  const pct = progressPercent(value, max);
  return (
    <div
      role="progressbar"
      aria-label={label}
      aria-valuemin={0}
      aria-valuemax={max}
      aria-valuenow={Math.min(max, Math.max(0, value))}
      aria-valuetext={valueText}
      className={cx(
        "overflow-hidden rounded-full",
        size === "lg" ? "h-[22px] border-2 border-line-sky bg-primary-soft" : "h-2 bg-white",
        className,
      )}
    >
      <div
        className="pressable h-full rounded-full"
        style={{ width: `${pct}%`, background: color, transitionProperty: "width" }}
      />
    </div>
  );
}
