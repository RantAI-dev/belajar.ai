import type { ReactNode } from "react";
import { cx } from "./cx";
import { subjectById } from "./subjects";

const SUN = "#ffd23f";

// Two-tone line icons on a 24px grid: strokes in the subject colour, fills in its light shade.
function glyph(id: string, c: string, tint: string): ReactNode {
  switch (id) {
    case "ipa":
      return (
        <>
          <path
            d="M7.4 14.5h9.2l2.3 4.3A1.5 1.5 0 0 1 17.6 21H6.4a1.5 1.5 0 0 1-1.3-2.2z"
            fill={tint}
            stroke="none"
          />
          <path d="M9 3h6" />
          <path d="M10 3v6.2L5.1 18.8A1.5 1.5 0 0 0 6.4 21h11.2a1.5 1.5 0 0 0 1.3-2.2L14 9.2V3" />
          <circle cx="10.4" cy="17.2" r="1" fill={c} />
          <circle cx="13.6" cy="15.9" r="0.8" fill={c} />
          <circle cx="12.6" cy="11.8" r="0.6" fill={c} />
        </>
      );
    case "mtk":
      return (
        <>
          <rect x="3" y="3" width="8" height="8" rx="2" fill={tint} />
          <path d="M7 5.3v3.4M5.3 7h3.4" />
          <circle cx="17" cy="7" r="4" fill={SUN} />
          <rect x="3" y="13" width="8" height="8" rx="2" fill={tint} />
          <path d="m5.4 15.4 3.2 3.2M8.6 15.4l-3.2 3.2" />
          <rect x="13" y="13" width="8" height="8" rx="2" fill={tint} />
          <path d="M15.3 17h3.4" />
        </>
      );
    case "bing":
      return (
        <>
          <path
            d="M4 4h16a1 1 0 0 1 1 1v10a1 1 0 0 1-1 1h-9l-5 4v-4H4a1 1 0 0 1-1-1V5a1 1 0 0 1 1-1z"
            fill={tint}
          />
          <text
            x="12"
            y="12.6"
            textAnchor="middle"
            fontFamily="Fredoka, Nunito, sans-serif"
            fontSize="7"
            fontWeight="600"
            fill={c}
            stroke="none"
          >
            Hi!
          </text>
        </>
      );
    case "bind":
      return (
        <>
          <path d="M2 5h7a3 3 0 0 1 3 3v12a2.5 2.5 0 0 0-2.5-2.5H2z" fill={tint} />
          <path d="M22 5h-7a3 3 0 0 0-3 3v12a2.5 2.5 0 0 1 2.5-2.5H22z" fill="#ffffff" />
          <text
            x="17"
            y="13.6"
            textAnchor="middle"
            fontFamily="Fredoka, Nunito, sans-serif"
            fontSize="6"
            fontWeight="600"
            fill={c}
            stroke="none"
          >
            Aa
          </text>
          <path d="M5 9h3.5M5 12h3.5" />
        </>
      );
    case "ips":
      return (
        <>
          <circle cx="12" cy="12" r="9" fill={tint} />
          <path d="M3 12h18" />
          <path d="M12 3a14 14 0 0 1 0 18a14 14 0 0 1 0-18z" />
          <path d="M6.5 7.2c1.8.6 2.8 2.2 2 4.2M15 15.5c1.4-.6 2.9-.2 3.9.8" />
        </>
      );
    case "inf":
      return (
        <>
          <rect x="3" y="4" width="18" height="12" rx="2" fill={tint} />
          <path d="M2 20h20" />
          <path d="M9.5 8 7.5 10l2 2M14.5 8l2 2-2 2" />
        </>
      );
    case "pp":
      return (
        <>
          <path d="M12 2.5 4 5.5v6c0 5 3.4 8.6 8 10 4.6-1.4 8-5 8-10v-6z" fill={tint} />
          <path
            d="m12 7.5 1.4 2.9 3.1.4-2.3 2.1.6 3.1L12 14.5l-2.8 1.5.6-3.1-2.3-2.1 3.1-.4z"
            fill={SUN}
          />
        </>
      );
    case "pai":
      return (
        <>
          <path d="M14 3.5a8.5 8.5 0 1 0 6.5 13.8A7 7 0 1 1 14 3.5z" fill={tint} />
          <path d="m18 5.5.9 1.9 2 .3-1.5 1.4.4 2-1.8-1-1.8 1 .4-2-1.5-1.4 2-.3z" fill={SUN} />
        </>
      );
    default:
      return <circle cx="12" cy="12" r="8" fill={tint} />;
  }
}

export type SubjectIconProps = {
  readonly subjectId: string;
  readonly size?: number;
  /** Draw the icon on a white rounded tile, as on subject cards. */
  readonly framed?: boolean;
  readonly className?: string;
};

export function SubjectIcon({ subjectId, size = 42, framed = false, className }: SubjectIconProps) {
  const subject = subjectById(subjectId);
  const color = subject?.color ?? "#0069a8";
  const tint = subject?.edge ?? "#e0f0fd";
  const svg = (
    <svg
      aria-hidden="true"
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke={color}
      strokeWidth="1.8"
      strokeLinecap="round"
      strokeLinejoin="round"
      data-subject={subjectId}
    >
      {glyph(subjectId, color, tint)}
    </svg>
  );
  if (!framed) return svg;
  const box = Math.round(size * 1.5);
  return (
    <span
      aria-hidden="true"
      className={cx("wiggle flex shrink-0 items-center justify-center bg-white", className)}
      style={{ width: box, height: box, borderRadius: Math.round(box * 0.28) }}
    >
      {svg}
    </span>
  );
}
