import { cx } from "./cx";
import { StarIcon } from "./icons";
import { clampRating } from "./rating";

/** The student's star total, as a pill. */
export function StarCount({
  count,
  className,
}: {
  readonly count: number;
  readonly className?: string;
}) {
  return (
    <span
      title="Bintang"
      className={cx(
        "inline-flex min-h-10 items-center gap-1.5 rounded-[12px] bg-warm px-3 text-[15px] font-extrabold text-star-ink",
        className,
      )}
    >
      <StarIcon />
      {count}
      <span className="sr-only"> bintang</span>
    </span>
  );
}

/** 0 to 3 stars, e.g. what a chapter or a practice set earned. */
export function StarRating({
  value,
  size = 16,
  emptyColor = "var(--color-star-empty)",
  className,
}: {
  readonly value: number;
  readonly size?: number;
  readonly emptyColor?: string;
  readonly className?: string;
}) {
  const rating = clampRating(value);
  return (
    <span
      role="img"
      aria-label={`${rating} dari 3 bintang`}
      className={cx("inline-flex gap-0.5", className)}
    >
      {[1, 2, 3].map((n) => (
        <StarIcon key={n} size={size} fill={n <= rating ? "var(--color-star)" : emptyColor} />
      ))}
    </span>
  );
}
