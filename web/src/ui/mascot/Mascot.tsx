import { type KeyboardEvent, useEffect, useId, useRef, useState } from "react";
import { cx } from "../cx";
import { AccentMarks } from "./accents";
import {
  type Arm,
  Blush,
  BrowShape,
  EYE_X,
  EyeShape,
  FACES,
  fill,
  type MascotExpression,
  MouthShape,
} from "./face";
import { useReducedMotion } from "./motion";
import { lookOffset, type Point } from "./track";

/** The older names, kept so existing screens need no change. */
export type MascotMood = "happy" | "thinking" | "neutral" | "celebrate" | "hmm";
const FROM_MOOD: Readonly<Record<MascotMood, MascotExpression>> = {
  happy: "happy",
  thinking: "thinking",
  neutral: "cheerful",
  celebrate: "celebration",
  hmm: "confused",
};

export type MascotProps = {
  readonly expression?: MascotExpression;
  /** @deprecated Use `expression`; mapped onto it (neutral is cheerful, hmm is confused). */
  readonly mood?: MascotMood;
  readonly size?: number;
  /** Circle behind the mascot: `light` (sky; dark blue in the dark theme), a colour, or `none`. */
  readonly backdrop?: "light" | "none" | (string & {});
  /** On a coloured panel: the marks around the mascot and its shadow turn white. */
  readonly inverse?: boolean;
  /** Idle breathing, a blink and a swaying tassel. */
  readonly animate?: boolean;
  /** The face follows the pointer and the mascot squashes when tapped. */
  readonly interactive?: boolean;
  /** With `interactive`: the mascot becomes a button (Enter and Space too) that calls this. */
  readonly onActivate?: () => void;
  /** Announces the mascot as an image (or names the button). Without it the mascot is decorative. */
  readonly title?: string;
  readonly className?: string;
};

// Right-hand positions [x, y, rotation]; the left hand mirrors them across x = 100.
const ARMS: Readonly<Record<Arm, readonly [number, number, number]>> = {
  rest: [164, 144, -28],
  low: [138, 166, 30],
  up: [170, 90, 20],
  wave: [174, 108, 40],
  chin: [118, 156, -50],
  cheek: [152, 112, 10],
  head: [160, 76, -30],
};

function Hand({ arm, left }: { readonly arm: Arm; readonly left: boolean }) {
  const [rx, y, r] = ARMS[arm];
  const x = left ? 200 - rx : rx;
  const turn = left ? -r : r;
  return (
    <g transform={`rotate(${turn} ${x} ${y})`}>
      <ellipse cx={x} cy={y} rx="13" ry="17" style={fill("mascot-arm")} />
      <ellipse cx={x - 3} cy={y - 6} rx="4" ry="6" style={fill("snow")} opacity="0.3" />
    </g>
  );
}

const BODY =
  "M100 56C142 56 170 84 172 122C174 160 142 180 100 180C58 180 26 160 28 122C30 84 58 56 100 56Z";

/**
 * The belajar.ai mascot: a gradient blob in a mortarboard. Ten expressions (see EXPRESSIONS);
 * `animate` adds idle motion, `interactive` pointer-tracking and a squash. Reduced motion shows
 * the static expression only. Decorative (aria-hidden) unless `title` is given.
 */
export function Mascot({
  expression,
  mood = "happy",
  size = 64,
  backdrop = "none",
  inverse = false,
  animate = false,
  interactive = false,
  onActivate,
  title,
  className,
}: MascotProps) {
  const name = expression ?? FROM_MOOD[mood];
  const face = FACES[name];
  const id = useId();
  const ref = useRef<SVGSVGElement>(null);
  const reduced = useReducedMotion();
  const [look, setLook] = useState<Point>({ x: 0, y: 0 });
  const [popping, setPopping] = useState(false);
  const tracking = interactive && !reduced;

  useEffect(() => {
    if (!tracking) return;
    let frame = 0;
    const onMove = (e: PointerEvent) => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(() => {
        const box = ref.current?.getBoundingClientRect();
        if (box !== undefined) setLook(lookOffset({ x: e.clientX, y: e.clientY }, box));
      });
    };
    window.addEventListener("pointermove", onMove, { passive: true });
    return () => {
      cancelAnimationFrame(frame);
      window.removeEventListener("pointermove", onMove);
    };
  }, [tracking]);

  const button = interactive && onActivate !== undefined;
  const activate = () => {
    if (!reduced) setPopping(true);
    onActivate?.();
  };
  const onKeyDown = (e: KeyboardEvent<SVGSVGElement>) => {
    if (e.key !== "Enter" && e.key !== " ") return;
    e.preventDefault();
    activate();
  };
  const body = `${id}-body`;
  const sleepy = name === "sleepy";
  const label = title ?? "Maskot belajar.ai";

  return (
    <svg
      ref={ref}
      width={size}
      height={size}
      viewBox="0 0 200 200"
      className={cx(
        "shrink-0 overflow-visible",
        button && "cursor-pointer rounded-card",
        className,
      )}
      data-expression={name}
      role={button ? "button" : "img"}
      tabIndex={button ? 0 : undefined}
      aria-label={button ? label : title}
      aria-hidden={button || title !== undefined ? undefined : true}
      onClick={interactive ? activate : undefined}
      onKeyDown={button ? onKeyDown : undefined}
    >
      {title !== undefined && <title>{title}</title>}
      <defs>
        <linearGradient id={body} x1="0.3" y1="0" x2="0.6" y2="1">
          <stop offset="0" style={{ stopColor: "var(--color-brand-teal)" }} />
          <stop offset="0.55" style={{ stopColor: "var(--color-brand-blue)" }} />
          <stop offset="1" style={{ stopColor: "var(--color-brand-purple)" }} />
        </linearGradient>
      </defs>
      {backdrop !== "none" && (
        <circle
          cx="100"
          cy="104"
          r="96"
          style={{ fill: backdrop === "light" ? "var(--color-mascot-backdrop)" : backdrop }}
        />
      )}
      <ellipse
        cx="100"
        cy="186"
        rx="54"
        ry="7"
        style={fill(inverse ? "snow" : "mascot-shadow")}
        opacity={inverse ? 0.25 : 1}
      />
      <AccentMarks kind={face.accent} inverse={inverse} />
      <g
        className={cx(popping && "mascot-pop")}
        onAnimationEnd={() => setPopping(false)}
        transform={sleepy ? "translate(0 11) scale(1 0.94)" : undefined}
      >
        <g className={cx(animate && "mascot-breathe")}>
          <path d={BODY} style={{ fill: `url(#${body})` }} />
          <ellipse
            cx="68"
            cy="88"
            rx="20"
            ry="10"
            transform="rotate(-35 68 88)"
            style={fill("snow")}
            opacity="0.35"
          />
          <g className="mascot-look" style={{ transform: `translate(${look.x}px, ${look.y}px)` }}>
            <Blush />
            <g className={cx(animate && !sleepy && "mascot-blink")}>
              <EyeShape kind={face.eyes[0]} x={EYE_X[0]} />
              <EyeShape kind={face.eyes[1]} x={EYE_X[1]} />
            </g>
            <BrowShape kind={face.brow} />
            <MouthShape kind={face.mouth} />
          </g>
          <Hand arm={face.arms[0]} left />
          <Hand arm={face.arms[1]} left={false} />
          <g transform={`rotate(${face.tilt} 100 62)`}>
            <path d="M68 62H132L130 78Q100 86 70 78Z" style={fill("mascot-cap-band")} />
            <path d="M46 50L100 64L154 50V55L100 69L46 55Z" style={fill("mascot-cap-band")} />
            <path d="M100 36L154 50L100 64L46 50Z" style={fill("mascot-cap")} />
            <circle cx="100" cy="50" r="3.5" style={fill("mascot-cap-band")} />
            <path
              d="M100 50Q128 50 148 54"
              style={{ stroke: "var(--color-mascot-cap-band)", fill: "none" }}
              strokeWidth="2.5"
            />
            <g className={cx(animate && "mascot-sway")}>
              <path
                d="M148 54V74"
                style={{ stroke: "var(--color-mascot-cap-band)" }}
                strokeWidth="2.5"
              />
              <rect x="143" y="72" width="10" height="15" rx="4" style={fill("mascot-cap")} />
            </g>
          </g>
        </g>
      </g>
    </svg>
  );
}
