// The mascot's face parts and the table of what each expression shows. Coordinates are in the
// Mascot's 200x200 viewBox; the face is centred near (100, 122). The icon set reuses these parts.

export const EXPRESSIONS = [
  "happy",
  "excited",
  "thinking",
  "idea",
  "cheerful",
  "sad",
  "sleepy",
  "confused",
  "surprised",
  "celebration",
] as const;
export type MascotExpression = (typeof EXPRESSIONS)[number];

export type Eye = "open" | "big" | "sad" | "up" | "happy" | "sleep" | "gt" | "lt";
export type Mouth = "smile" | "grin" | "o" | "small" | "side" | "frown" | "wavy";
export type Brow = "none" | "sad" | "raised" | "one";
/** Where a hand is, drawn for the right arm; the left arm mirrors it. */
export type Arm = "rest" | "low" | "up" | "wave" | "chin" | "cheek" | "head";
export type Accent =
  | "lines"
  | "bangs"
  | "thought"
  | "bulb"
  | "stars"
  | "tear"
  | "zzz"
  | "question"
  | "surprise"
  | "confetti";

export type Face = {
  readonly eyes: readonly [Eye, Eye];
  readonly mouth: Mouth;
  readonly brow: Brow;
  readonly arms: readonly [Arm, Arm];
  readonly accent: Accent;
  /** The cap's tilt in degrees (negative leans left). */
  readonly tilt: number;
};

// Matches the owner's sheet (mascot-sheet.png) and the EXPR table of the login prototype.
export const FACES: Readonly<Record<MascotExpression, Face>> = {
  happy: {
    eyes: ["happy", "happy"],
    mouth: "grin",
    brow: "none",
    arms: ["rest", "wave"],
    accent: "lines",
    tilt: -10,
  },
  excited: {
    eyes: ["gt", "lt"],
    mouth: "grin",
    brow: "none",
    arms: ["up", "up"],
    accent: "bangs",
    tilt: -8,
  },
  thinking: {
    eyes: ["up", "up"],
    mouth: "side",
    brow: "one",
    arms: ["rest", "chin"],
    accent: "thought",
    tilt: -10,
  },
  idea: {
    eyes: ["open", "open"],
    mouth: "smile",
    brow: "raised",
    arms: ["rest", "up"],
    accent: "bulb",
    tilt: -10,
  },
  cheerful: {
    eyes: ["happy", "open"],
    mouth: "smile",
    brow: "none",
    arms: ["wave", "rest"],
    accent: "stars",
    tilt: -12,
  },
  sad: {
    eyes: ["sad", "sad"],
    mouth: "frown",
    brow: "sad",
    arms: ["low", "low"],
    accent: "tear",
    tilt: -4,
  },
  sleepy: {
    eyes: ["sleep", "sleep"],
    mouth: "small",
    brow: "none",
    arms: ["rest", "rest"],
    accent: "zzz",
    tilt: -22,
  },
  confused: {
    eyes: ["open", "open"],
    mouth: "wavy",
    brow: "one",
    arms: ["rest", "head"],
    accent: "question",
    tilt: -14,
  },
  surprised: {
    eyes: ["big", "big"],
    mouth: "o",
    brow: "raised",
    arms: ["cheek", "cheek"],
    accent: "surprise",
    tilt: -6,
  },
  celebration: {
    eyes: ["gt", "lt"],
    mouth: "grin",
    brow: "none",
    arms: ["up", "up"],
    accent: "confetti",
    tilt: -12,
  },
};

export const fill = (token: string) => ({ fill: `var(--color-${token})` });
export const stroke = (token: string) => ({ stroke: `var(--color-${token})`, fill: "none" });
const INK = stroke("mascot-face");

const EYE_Y = 116;
export const EYE_X = [78, 122] as const;

export function EyeShape({ kind, x }: { readonly kind: Eye; readonly x: number }) {
  const y = EYE_Y;
  switch (kind) {
    case "open":
    case "big":
    case "sad":
    case "up": {
      const [rx, ry, shine] =
        kind === "big" ? [8.5, 11.5, 3.2] : kind === "sad" ? [7.5, 9, 3.6] : [7, 9.5, 2.6];
      const [cx, cy] = kind === "up" ? [x + 2, y - 3] : [x, y];
      const look = kind === "up" ? 1.5 : 0;
      return (
        <g>
          <ellipse cx={cx} cy={cy} rx={rx} ry={ry} style={fill("mascot-face")} />
          <circle cx={cx - 2.2 + look} cy={cy - 3.5 - look} r={shine} style={fill("snow")} />
          <circle cx={cx + 2.2} cy={cy + 3.2} r={1.2} style={fill("snow")} />
        </g>
      );
    }
    case "happy":
      return (
        <path
          d={`M${x - 8} ${y + 3}Q${x} ${y - 8} ${x + 8} ${y + 3}`}
          style={INK}
          strokeWidth="4"
          strokeLinecap="round"
        />
      );
    case "sleep":
      return (
        <path
          d={`M${x - 8} ${y - 1}Q${x} ${y + 7} ${x + 8} ${y - 1}`}
          style={INK}
          strokeWidth="4"
          strokeLinecap="round"
        />
      );
    case "gt":
    case "lt": {
      const s = kind === "gt" ? 1 : -1;
      return (
        <path
          d={`M${x - 6 * s} ${y - 7}L${x + 6 * s} ${y}L${x - 6 * s} ${y + 7}`}
          style={INK}
          strokeWidth="4"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
      );
    }
  }
}

export function MouthShape({ kind }: { readonly kind: Mouth }) {
  switch (kind) {
    case "smile":
      return <path d="M90 133Q100 143 110 133" style={INK} strokeWidth="4" strokeLinecap="round" />;
    case "grin":
      return (
        <g>
          <path
            d="M88 131Q100 129 112 131Q111 149 100 150Q89 149 88 131Z"
            style={fill("mascot-mouth")}
          />
          <ellipse cx="100" cy="144" rx="6.5" ry="3.8" style={fill("mascot-tongue")} />
        </g>
      );
    case "o":
      return <ellipse cx="100" cy="139" rx="6" ry="8" style={fill("mascot-mouth")} />;
    case "small":
      return <ellipse cx="100" cy="137" rx="4" ry="3" style={fill("mascot-face")} />;
    case "side":
      return (
        <path d="M98 138Q105 141 112 136" style={INK} strokeWidth="3.5" strokeLinecap="round" />
      );
    case "frown":
      return <path d="M91 142Q100 133 109 142" style={INK} strokeWidth="4" strokeLinecap="round" />;
    case "wavy":
      return (
        <path d="M88 138q3-5 6 0t6 0t6 0t6 0" style={INK} strokeWidth="3.5" strokeLinecap="round" />
      );
  }
}

const BROWS: Readonly<Record<Exclude<Brow, "none">, string>> = {
  sad: "M68 103L86 97M114 97L132 103",
  raised: "M70 99Q78 92 86 99M114 99Q122 92 130 99",
  one: "M70 101L86 101M114 96L131 91",
};

export function BrowShape({ kind }: { readonly kind: Brow }) {
  if (kind === "none") return null;
  return <path d={BROWS[kind]} style={INK} strokeWidth="3.5" strokeLinecap="round" />;
}

export function Blush() {
  return (
    <g style={fill("mascot-blush")} opacity="0.6">
      <ellipse cx="64" cy="134" rx="10" ry="6" />
      <ellipse cx="136" cy="134" rx="10" ry="6" />
    </g>
  );
}
