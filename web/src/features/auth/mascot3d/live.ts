// The page state the login mascot reacts to. MascotStage keeps it up to date from DOM events; the
// 3D scene and the SVG fallback both read it and ask pose.ts what to show.
import type { Focus, Outcome, PoseInput, Reaction } from "./pose";

export type Point = { readonly x: number; readonly y: number };

export type Live = {
  hover: boolean;
  lastActivityAt: number;
  reaction: Reaction | null;
  outcome: Outcome | null;
  /** Pokes so far, to cycle the reactions. */
  pokes: number;
  pointer: Point;
};

export const newLive = (now: number): Live => ({
  hover: false,
  lastActivityAt: now,
  reaction: null,
  outcome: null,
  pokes: 0,
  pointer: { x: 0, y: 0 },
});

/** The pose input for this moment: `live` plus whichever field has focus right now. */
export const poseInput = (live: Live, now: number): PoseInput => ({
  ...live,
  ...focusOf(document.activeElement),
  now,
});

/** Which kind of form control `el` is, for the mascot: the password, a text field, or neither. */
export function focusOf(el: Element | null): { focus: Focus; passwordShown: boolean } {
  if (!(el instanceof HTMLInputElement)) return { focus: null, passwordShown: false };
  if (el.hasAttribute("data-password")) {
    return { focus: "password", passwordShown: el.type === "text" };
  }
  const text = ["text", "email", "search", "tel", "url"].includes(el.type);
  return { focus: text ? "text" : null, passwordShown: false };
}

let measure: CanvasRenderingContext2D | null = null;

/** Where the caret of the focused input is on screen, or its middle for the password. */
export function fieldPoint(): Point | null {
  const el = document.activeElement;
  if (!(el instanceof HTMLInputElement)) return null;
  const box = el.getBoundingClientRect();
  const y = box.top + box.height / 2;
  if (el.hasAttribute("data-password")) return { x: box.left + box.width * 0.6, y };
  measure ??= document.createElement("canvas").getContext("2d");
  if (measure === null) return { x: box.left + box.width / 2, y };
  measure.font = getComputedStyle(el).font;
  const before = el.value.slice(0, el.selectionStart ?? el.value.length);
  const width = measure.measureText(before).width;
  return { x: box.left + 14 + Math.min(width, box.width - 28), y };
}

/** True when this browser can draw WebGL at all (checked before the 3D chunk is fetched). */
export function hasWebGL(): boolean {
  try {
    const c = document.createElement("canvas");
    return c.getContext("webgl2") !== null || c.getContext("webgl") !== null;
  } catch {
    return false;
  }
}
