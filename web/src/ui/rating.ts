export type Rating = 0 | 1 | 2 | 3;

/** Clamps any number to a whole 0..3 star rating. */
export const clampRating = (n: number): Rating => {
  if (!Number.isFinite(n) || n <= 0) return 0;
  if (n >= 3) return 3;
  return n < 1.5 ? 1 : n < 2.5 ? 2 : 3;
};

/** Stars for a score: all correct is 3, at least 60% is 2, anything right is 1. */
export const starsForScore = (correct: number, total: number): Rating => {
  if (total <= 0 || correct <= 0) return 0;
  const ratio = correct / total;
  if (ratio >= 1) return 3;
  return ratio >= 0.6 ? 2 : 1;
};
