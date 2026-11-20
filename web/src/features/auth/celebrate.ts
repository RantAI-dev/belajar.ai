/** How long the mascot celebrates a sign-in before the app moves on. */
export const CELEBRATE_MS = 800;

/** CELEBRATE_MS, or 0 when the student asked for less motion (move on at once). */
export function celebrateMs(): number {
  try {
    return window.matchMedia("(prefers-reduced-motion: reduce)").matches ? 0 : CELEBRATE_MS;
  } catch {
    return CELEBRATE_MS;
  }
}
