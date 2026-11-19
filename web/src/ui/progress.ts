/** Share of `value` in `max` as a whole percentage, clamped to 0..100. A zero or bad max is 0. */
export const progressPercent = (value: number, max: number): number => {
  if (!Number.isFinite(value) || !Number.isFinite(max) || max <= 0) return 0;
  return Math.round(Math.min(100, Math.max(0, (value / max) * 100)));
};
