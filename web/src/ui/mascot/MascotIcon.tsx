import { useId } from "react";
import { cx } from "../cx";
import {
  type Brow,
  BrowShape,
  EYE_X,
  type Eye,
  EyeShape,
  fill,
  type Mouth,
  MouthShape,
  stroke,
} from "./face";

export const MASCOT_ICON_KINDS = [
  "assistant",
  "thinking",
  "idea",
  "celebrate",
  "confused",
  "sleepy",
  "error",
] as const;
export type MascotIconKind = (typeof MASCOT_ICON_KINDS)[number];

type IconFace = { readonly eyes: readonly [Eye, Eye]; readonly mouth: Mouth; readonly brow: Brow };
const ICON_FACES: Readonly<Record<MascotIconKind, IconFace>> = {
  assistant: { eyes: ["open", "open"], mouth: "smile", brow: "none" },
  thinking: { eyes: ["up", "up"], mouth: "side", brow: "one" },
  idea: { eyes: ["open", "open"], mouth: "smile", brow: "raised" },
  celebrate: { eyes: ["gt", "lt"], mouth: "grin", brow: "none" },
  confused: { eyes: ["open", "open"], mouth: "wavy", brow: "one" },
  sleepy: { eyes: ["sleep", "sleep"], mouth: "small", brow: "none" },
  error: { eyes: ["sad", "sad"], mouth: "frown", brow: "sad" },
};

// One bold mark in the top-right corner, drawn big enough to read at 20px.
function Badge({ kind }: { readonly kind: MascotIconKind }) {
  const mark = stroke("mascot-accent");
  switch (kind) {
    case "thinking":
      return (
        <g style={fill("mascot-accent")}>
          <circle cx="162" cy="40" r="9" />
          <circle cx="182" cy="16" r="14" />
        </g>
      );
    case "idea":
      return (
        <g>
          <circle cx="174" cy="28" r="22" style={fill("brand-sparkle")} />
          <rect x="164" y="48" width="20" height="14" rx="4" style={fill("mascot-accent")} />
        </g>
      );
    case "celebrate":
      return (
        <g>
          <rect
            x="160"
            y="6"
            width="12"
            height="20"
            rx="3"
            transform="rotate(30 166 16)"
            style={fill("brand-sparkle")}
          />
          <rect
            x="180"
            y="34"
            width="12"
            height="20"
            rx="3"
            transform="rotate(-20 186 44)"
            style={fill("brand-pink")}
          />
          <rect
            x="8"
            y="20"
            width="12"
            height="20"
            rx="3"
            transform="rotate(-30 14 30)"
            style={fill("brand-purple")}
          />
        </g>
      );
    case "confused":
      return (
        <g>
          <path
            d="M162 22C162 4 192 4 192 22C192 34 177 34 177 46"
            style={mark}
            strokeWidth="10"
            strokeLinecap="round"
          />
          <circle cx="177" cy="64" r="6" style={fill("mascot-accent")} />
        </g>
      );
    case "sleepy":
      return (
        <path
          d="M160 10h30l-30 30h30"
          style={mark}
          strokeWidth="9"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
      );
    case "assistant":
    case "error":
      return null;
  }
}

/**
 * Small mascot heads for UI (avatars, states, toasts), legible at 20-40px: no arms, a bigger face
 * and one corner mark. `assistant` is the chat avatar, `error` the sad face for failures.
 */
export function MascotIcon({
  kind,
  size = 32,
  title,
  className,
}: {
  readonly kind: MascotIconKind;
  readonly size?: number;
  readonly title?: string;
  readonly className?: string;
}) {
  const id = `${useId()}-head`;
  const face = ICON_FACES[kind];
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 200 200"
      className={cx("shrink-0", className)}
      data-kind={kind}
      role="img"
      aria-label={title}
      aria-hidden={title === undefined ? true : undefined}
    >
      {title !== undefined && <title>{title}</title>}
      <defs>
        <linearGradient id={id} x1="0.3" y1="0" x2="0.6" y2="1">
          <stop offset="0" style={{ stopColor: "var(--color-brand-teal)" }} />
          <stop offset="0.55" style={{ stopColor: "var(--color-brand-blue)" }} />
          <stop offset="1" style={{ stopColor: "var(--color-brand-purple)" }} />
        </linearGradient>
      </defs>
      <path
        d="M100 44C156 44 192 80 192 126C192 172 152 196 100 196C48 196 8 172 8 126C8 80 44 44 100 44Z"
        style={{ fill: `url(#${id})` }}
      />
      <path d="M100 6L170 26L100 46L30 26Z" style={fill("mascot-cap")} />
      <path d="M58 34H142L138 58Q100 68 62 58Z" style={fill("mascot-cap-band")} />
      <g transform="translate(100 126) scale(1.45) translate(-100 -124)">
        <ellipse cx="64" cy="136" rx="9" ry="5.5" style={fill("mascot-blush")} opacity="0.6" />
        <ellipse cx="136" cy="136" rx="9" ry="5.5" style={fill("mascot-blush")} opacity="0.6" />
        <EyeShape kind={face.eyes[0]} x={EYE_X[0]} />
        <EyeShape kind={face.eyes[1]} x={EYE_X[1]} />
        <BrowShape kind={face.brow} />
        <MouthShape kind={face.mouth} />
      </g>
      <Badge kind={kind} />
    </svg>
  );
}
