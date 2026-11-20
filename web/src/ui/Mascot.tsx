import { APP_NAME } from "./brand";
import { cx } from "./cx";
import { AppLink } from "./link";

export type MascotMood = "happy" | "thinking" | "neutral" | "celebrate" | "hmm";

const NAVY = "var(--color-brand-navy)";
const SKY = "var(--color-brand-sky)";
const SUN = "var(--color-sun)";
const SNOW = "var(--color-snow)";

const BOOK_LEFT = "M24 114C24 106 30 102 38 104L96 118V184L38 170C30 168 24 164 24 156Z";
const BOOK_RIGHT = "M176 114C176 106 170 102 162 104L104 118V184L162 170C170 168 176 164 176 156Z";

function Face({ mood }: { readonly mood: MascotMood }) {
  switch (mood) {
    case "happy":
      return (
        <path
          d="M84 80q16 12 32 0"
          style={{ stroke: NAVY }}
          strokeWidth="6"
          fill="none"
          strokeLinecap="round"
        />
      );
    case "celebrate":
      return (
        <>
          <path
            d="M82 76q18 18 36 0z"
            style={{ fill: NAVY, stroke: NAVY }}
            strokeWidth="4"
            strokeLinejoin="round"
          />
          <path d="m30 40 4 9 9 4-9 4-4 9-4-9-9-4 9-4z" style={{ fill: SUN }} />
          <path d="m168 26 3 7 7 3-7 3-3 7-3-7-7-3 7-3z" style={{ fill: SUN }} />
          <circle cx="160" cy="88" r="5" style={{ fill: SUN }} />
        </>
      );
    case "thinking":
      return (
        <>
          <circle cx="100" cy="84" r="6" style={{ fill: NAVY }} />
          <circle cx="150" cy="40" r="6" style={{ fill: SKY }} />
          <circle cx="166" cy="24" r="9" style={{ fill: SKY }} />
        </>
      );
    case "hmm":
      return (
        <>
          <path d="M88 88l24-6" style={{ stroke: NAVY }} strokeWidth="6" strokeLinecap="round" />
          <path
            d="M106 36q10-6 20 0"
            style={{ stroke: NAVY }}
            strokeWidth="5"
            fill="none"
            strokeLinecap="round"
          />
        </>
      );
    case "neutral":
      return (
        <path
          d="M90 82q10 5 20 0"
          style={{ stroke: NAVY }}
          strokeWidth="6"
          fill="none"
          strokeLinecap="round"
        />
      );
  }
}

export type MascotProps = {
  readonly mood?: MascotMood;
  readonly size?: number;
  /** Circle behind the mascot: `light` (sky; dark blue in the dark theme), a colour, or `none`. */
  readonly backdrop?: "light" | "none" | (string & {});
  /** On a dark or coloured background the book turns white (the dark theme does it by itself). */
  readonly inverse?: boolean;
  /** Decorative by default. Pass a label to announce it as an image. */
  readonly label?: string;
  readonly animate?: boolean;
  readonly className?: string;
};

/** The mascot: a sky-blue head with a white eye, reading an open navy book. */
export function Mascot({
  mood = "happy",
  size = 64,
  backdrop = "light",
  inverse = false,
  label,
  animate = false,
  className,
}: MascotProps) {
  const fill = backdrop === "light" ? "var(--color-mascot-backdrop)" : backdrop;
  return (
    <svg
      role="img"
      aria-hidden={label === undefined ? true : undefined}
      aria-label={label}
      width={size}
      height={size}
      viewBox="0 0 200 200"
      className={cx("shrink-0", animate && "bob", className)}
      data-mood={mood}
    >
      {backdrop !== "none" && <circle cx="100" cy="100" r="96" style={{ fill }} />}
      <circle cx="100" cy="64" r="40" style={{ fill: SKY }} />
      <circle cx="116" cy="56" r="9" style={{ fill: SNOW }} />
      <Face mood={mood} />
      <path d={BOOK_LEFT} style={{ fill: inverse ? SNOW : "var(--color-mascot-book)" }} />
      <path
        d={BOOK_RIGHT}
        style={{
          fill: inverse ? "var(--color-mascot-page-shade)" : "var(--color-mascot-book-shade)",
        }}
      />
    </svg>
  );
}

/** The logo mark from the mockup: the mascot's head over the book, no face. */
export function LogoMark({ size = 40 }: { readonly size?: number }) {
  return (
    <svg aria-hidden="true" width={size} height={size} viewBox="0 0 200 200" className="shrink-0">
      <circle cx="100" cy="58" r="40" style={{ fill: SKY }} />
      <circle cx="116" cy="50" r="9" style={{ fill: SNOW }} />
      <path
        d="M14 112C14 104 20 100 28 102L94 118V192L28 176C20 174 14 170 14 162Z"
        style={{ fill: "var(--color-mascot-book)" }}
      />
      <path
        d="M186 112C186 104 180 100 172 102L106 118V192L172 176C180 174 186 170 186 162Z"
        style={{ fill: "var(--color-mascot-book-shade)" }}
      />
    </svg>
  );
}

export function Logo({
  href,
  compact = false,
}: {
  readonly href: string;
  readonly compact?: boolean;
}) {
  return (
    <AppLink
      href={href}
      aria-label={`${APP_NAME}, beranda`}
      className="flex min-h-12 items-center gap-2.5 rounded-chip text-ink"
    >
      <LogoMark size={compact ? 36 : 40} />
      <span
        className={cx(
          "font-display font-semibold max-[419px]:sr-only",
          compact ? "text-[22px]" : "text-[26px]",
        )}
      >
        {APP_NAME}
      </span>
    </AppLink>
  );
}
