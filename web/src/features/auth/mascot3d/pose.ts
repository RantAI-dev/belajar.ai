// What the login mascot does, as a pure function of what is happening on the page. The 3D scene
// asks every frame and the SVG fallback a few times a second; neither needs WebGL to be tested.
import {
  MASCOT_FACES,
  type MascotArm,
  type MascotBrow,
  type MascotExpression,
  type MascotEye,
  type MascotMouth,
} from "../../../ui";

/** The SVG arms, plus a hand over an eye (the password is being typed). */
export type Arm3d = MascotArm | "cover";
/** Where the eyes go: the pointer, the focused field (its caret), or straight ahead. */
export type Look = "pointer" | "field" | "none";
/** The focused form control: a text input, the password input, or anything else. */
export type Focus = "text" | "password" | null;

export type Pose = {
  /** The SVG fallback shows this expression; the 3D face starts from its row in MASCOT_FACES. */
  readonly expression: MascotExpression;
  /** 3D-only overrides of that row, for looks the ten expressions do not have (eyes covered). */
  readonly eyes?: readonly [MascotEye, MascotEye];
  readonly mouth?: MascotMouth;
  readonly brow?: MascotBrow;
  readonly arms: readonly [Arm3d, Arm3d];
  readonly look: Look;
};

/** The real result of a sign-in attempt, stamped with when it arrived (performance.now()). */
export type Outcome = { readonly kind: "success" | "failure"; readonly at: number };
/** A short reaction to a poke (click, Enter, waking up), stamped with when it started. */
export type Reaction = { readonly pose: Pose; readonly at: number; readonly ms: number };

export type PoseInput = {
  readonly now: number;
  readonly focus: Focus;
  readonly passwordShown: boolean;
  readonly hover: boolean;
  readonly lastActivityAt: number;
  readonly reaction: Reaction | null;
  readonly outcome: Outcome | null;
};

export const SAD_MS = 1200;
export const FAILURE_MS = 2400;
export const SLEEP_AFTER_MS = 22_000;
export const REACTION_MS = 1300;

const as = (expression: MascotExpression, look: Look = "pointer"): Pose => ({
  expression,
  arms: MASCOT_FACES[expression].arms,
  look,
});

/** What a click or Enter on the mascot cycles through. */
export const POKES: readonly Pose[] = [
  as("cheerful"),
  as("excited"),
  as("surprised"),
  { ...as("happy"), arms: ["up", "rest"] },
];

/** The wave when the page opens. */
export const GREETING: Pose = { ...as("happy"), arms: ["rest", "wave"] };
/** Woken from a nap. */
export const WAKE: Pose = as("surprised");

const IDLE: Pose = {
  expression: "happy",
  eyes: ["open", "open"],
  mouth: "smile",
  arms: ["rest", "rest"],
  look: "pointer",
};

export const isSleeping = (i: PoseInput): boolean =>
  i.focus === null && i.outcome?.kind !== "success" && i.now - i.lastActivityAt > SLEEP_AFTER_MS;

/** The mascot's pose for this moment. Earlier rules win: the sign-in result over everything. */
export function decidePose(i: PoseInput): Pose {
  if (i.outcome?.kind === "success") return as("celebration");
  if (i.outcome?.kind === "failure") {
    const age = i.now - i.outcome.at;
    if (age < SAD_MS) return as("sad");
    if (age < FAILURE_MS) return as("confused");
  }
  if (i.reaction !== null && i.now - i.reaction.at < i.reaction.ms) return i.reaction.pose;
  if (isSleeping(i)) return { ...as("sleepy", "none"), arms: ["rest", "rest"] };
  if (i.focus === "password" && !i.passwordShown) {
    // The SVG cannot cover its eyes, so it closes them (sleepy) instead of peeking.
    return {
      expression: "sleepy",
      eyes: ["happy", "happy"],
      mouth: "smile",
      brow: "none",
      arms: ["cover", "cover"],
      look: "none",
    };
  }
  if (i.focus === "password") {
    return { expression: "cheerful", mouth: "small", arms: ["cover", "rest"], look: "field" };
  }
  if (i.hover) return { ...IDLE, mouth: "grin", arms: ["rest", "wave"] };
  if (i.focus === "text") {
    return { ...as("idea", "field"), mouth: "small", brow: "none", arms: ["rest", "rest"] };
  }
  return IDLE;
}

/** The next poke reaction in the cycle; none while the sign-in result is showing. */
export function pokeReaction(i: PoseInput, count: number): Reaction | null {
  const busy =
    i.outcome !== null && (i.outcome.kind === "success" || i.now - i.outcome.at < FAILURE_MS);
  if (busy) return null;
  return { pose: POKES[count % POKES.length] ?? GREETING, at: i.now, ms: REACTION_MS };
}
