// Where the mascot looks: pure, so the eye tracking is tested without a browser.

export type Point = { readonly x: number; readonly y: number };
export type Box = {
  readonly left: number;
  readonly top: number;
  readonly width: number;
  readonly height: number;
};

/** How far the face may slide toward the pointer, in viewBox units (the eyes stay on the body). */
export const MAX_LOOK = 6;

/**
 * The face's offset toward `pointer` for a mascot drawn in `box`: along the pointer's direction,
 * growing with distance until two mascot-widths away, never longer than `max`. Rounded to 0.1 so
 * pointer jitter does not re-render.
 */
export function lookOffset(pointer: Point, box: Box, max: number = MAX_LOOK): Point {
  const cx = box.left + box.width / 2;
  // The eyes sit a little below the middle of the drawing.
  const cy = box.top + box.height * 0.58;
  const dx = pointer.x - cx;
  const dy = pointer.y - cy;
  const distance = Math.hypot(dx, dy);
  if (distance === 0 || !Number.isFinite(distance)) return { x: 0, y: 0 };
  const reach = Math.max(box.width, 1) * 2;
  const length = max * Math.min(1, distance / reach);
  const round = (n: number) => Math.round(n * 10) / 10 || 0;
  return { x: round((dx / distance) * length), y: round((dy / distance) * length) };
}
