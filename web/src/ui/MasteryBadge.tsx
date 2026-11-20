import type { components } from "../contracts/api.gen";
import { cx } from "./cx";

/** The server decides the level (one rule in `belajar_core::mastery`); the UI only names it. */
export type MasteryLevel = components["schemas"]["MasteryLevelDto"];

export const MASTERY_LABEL: Readonly<Record<MasteryLevel, string>> = {
  not_started: "Belum mulai",
  learning: "Sedang belajar",
  almost: "Hampir",
  mastered: "Dikuasai",
};

const TONE: Readonly<Record<MasteryLevel, string>> = {
  not_started: "bg-page text-ink-muted border-line-strong",
  learning: "bg-primary-soft text-primary-strong border-line-sky",
  almost: "bg-warm text-warm-ink border-warm-line",
  mastered: "bg-success-soft text-success-edge border-success",
};

/** How many of the four segments are filled: the level reads as a shape, not only a colour. */
const FILLED: Readonly<Record<MasteryLevel, number>> = {
  not_started: 0,
  learning: 1,
  almost: 2,
  mastered: 3,
};

/** Three little steps, filled up to the level: an icon that still works in greyscale. */
export function MasteryMeter({
  level,
  size = 18,
}: {
  readonly level: MasteryLevel;
  readonly size?: number;
}) {
  const filled = FILLED[level];
  return (
    <svg aria-hidden="true" width={size} height={size} viewBox="0 0 18 18" className="shrink-0">
      {[0, 1, 2].map((i) => (
        <rect
          key={i}
          x={1 + i * 6}
          y={12 - i * 5}
          width="4.5"
          height={5 + i * 5}
          rx="1.5"
          fill={i < filled ? "currentColor" : "none"}
          stroke="currentColor"
          strokeWidth="1.5"
        />
      ))}
    </svg>
  );
}

/** "Dikuasai", "Hampir", …: a pill with the level's name, its step icon and its tone. */
export function MasteryBadge({
  level,
  className,
}: {
  readonly level: MasteryLevel;
  readonly className?: string;
}) {
  return (
    <span
      className={cx(
        "inline-flex min-h-8 shrink-0 items-center gap-1.5 rounded-full border-2 px-2.5 text-[14px] font-extrabold",
        TONE[level],
        className,
      )}
    >
      <MasteryMeter level={level} />
      {MASTERY_LABEL[level]}
    </span>
  );
}
