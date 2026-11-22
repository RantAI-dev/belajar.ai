// The marks around the mascot that tell expressions apart at a glance (bulb, zzz, ?, confetti...).
// `inverse` paints the page-coloured marks white for coloured panels.
import { type Accent, fill, stroke } from "./face";

const sparkle = (x: number, y: number, r: number) =>
  `M${x} ${y - r}Q${x + r * 0.2} ${y - r * 0.2} ${x + r} ${y}Q${x + r * 0.2} ${y + r * 0.2} ${x} ${y + r}Q${x - r * 0.2} ${y + r * 0.2} ${x - r} ${y}Q${x - r * 0.2} ${y - r * 0.2} ${x} ${y - r}Z`;

function Bang({
  x,
  y,
  scale,
  token,
}: {
  readonly x: number;
  readonly y: number;
  readonly scale: number;
  readonly token: string;
}) {
  return (
    <g transform={`translate(${x} ${y}) rotate(14) scale(${scale})`} style={fill(token)}>
      <rect x="-3.5" y="-12" width="7" height="17" rx="3.5" />
      <circle cx="0" cy="11" r="3.6" />
    </g>
  );
}

function Question({
  x,
  y,
  scale,
  token,
}: {
  readonly x: number;
  readonly y: number;
  readonly scale: number;
  readonly token: string;
}) {
  return (
    <g transform={`translate(${x} ${y}) rotate(10) scale(${scale})`}>
      <path
        d="M-6-6C-6-14 6-14 6-6C6-1 0 0 0 5"
        style={stroke(token)}
        strokeWidth="4"
        strokeLinecap="round"
      />
      <circle cx="0" cy="12" r="2.6" style={fill(token)} />
    </g>
  );
}

const CONFETTI: ReadonlyArray<readonly [number, number, number, string]> = [
  [22, 30, 20, "brand-teal"],
  [44, 12, -30, "brand-pink"],
  [158, 10, 40, "brand-purple"],
  [186, 36, -15, "brand-sparkle"],
  [12, 66, 60, "brand-blue"],
  [190, 70, 30, "brand-pink"],
  [10, 120, -40, "brand-sparkle"],
  [192, 126, 15, "brand-teal"],
  [30, 170, 50, "brand-purple"],
  [174, 172, -25, "brand-blue"],
];

export function AccentMarks({
  kind,
  inverse,
}: {
  readonly kind: Accent;
  readonly inverse: boolean;
}) {
  const mark = inverse ? "snow" : "mascot-accent";
  switch (kind) {
    case "lines":
      return (
        <path
          d="M24 72l-11-7M20 88H8M176 64l11-7M182 80h12"
          style={stroke("brand-sparkle")}
          strokeWidth="4.5"
          strokeLinecap="round"
        />
      );
    case "bangs":
      return (
        <g>
          <Bang x={170} y={24} scale={1} token="brand-sparkle" />
          <Bang x={188} y={40} scale={0.75} token="brand-sparkle" />
          <path
            d="M22 74l-11-8M18 90H6"
            style={stroke("brand-sparkle")}
            strokeWidth="4.5"
            strokeLinecap="round"
          />
        </g>
      );
    case "surprise":
      return (
        <g>
          <Bang x={176} y={30} scale={1.1} token="brand-sparkle" />
          <Bang x={24} y={34} scale={0.9} token="brand-sparkle" />
        </g>
      );
    case "thought":
      return (
        <g style={fill(mark)}>
          <circle cx="164" cy="48" r="3.5" />
          <circle cx="173" cy="35" r="5.5" />
          <ellipse cx="184" cy="17" rx="13" ry="10" />
          <g style={fill(inverse ? "hero" : "page")}>
            <circle cx="178" cy="17" r="2" />
            <circle cx="184" cy="17" r="2" />
            <circle cx="190" cy="17" r="2" />
          </g>
        </g>
      );
    case "bulb":
      return (
        <g>
          <path
            d="M180 2v6M162 10l5 4M198 10l-5 4M158 26h6M196 26h6"
            style={stroke("brand-sparkle")}
            strokeWidth="3"
            strokeLinecap="round"
          />
          <circle
            cx="180"
            cy="25"
            r="12"
            style={{ ...fill("brand-sparkle"), stroke: "var(--color-sun-edge)" }}
            strokeWidth="1.5"
          />
          <circle cx="176" cy="21" r="3.5" style={fill("snow")} opacity="0.8" />
          <rect x="174" y="36" width="12" height="9" rx="2.5" style={fill(mark)} />
        </g>
      );
    case "stars":
      return (
        <g>
          <path d={sparkle(172, 28, 10)} style={fill("brand-pink")} />
          <path d={sparkle(188, 54, 6)} style={fill("brand-purple")} />
          <path d={sparkle(26, 42, 8)} style={fill("brand-sparkle")} />
          <circle cx="20" cy="70" r="3.5" style={fill("brand-purple")} />
          <circle cx="160" cy="10" r="3" style={fill("brand-teal")} />
        </g>
      );
    case "tear":
      return (
        <g>
          <path
            d="M131 124c3 5 5 8 5 10.5a5 5 0 0 1-10 0c0-2.5 2-5.5 5-10.5Z"
            style={fill("snow")}
            opacity="0.9"
          />
          <path
            d="M166 30c4-12 22-12 24-1c2 10-17 10-15 0c2-8 15-6 13 3"
            style={stroke(mark)}
            strokeWidth="3"
            strokeLinecap="round"
          />
        </g>
      );
    case "zzz":
      return (
        <path
          d="M158 44h8l-8 8h8M168 26h11l-11 11h11M182 4h14l-14 14h14"
          style={stroke(mark)}
          strokeWidth="3.5"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
      );
    case "question":
      return (
        <g>
          <Question x={180} y={22} scale={1.4} token={mark} />
          <Question x={160} y={36} scale={0.8} token={mark} />
        </g>
      );
    case "confetti":
      return (
        <g>
          {CONFETTI.map(([x, y, angle, token]) => (
            <rect
              key={`${x}-${y}`}
              x={x - 3}
              y={y - 5}
              width="6"
              height="10"
              rx="1.5"
              transform={`rotate(${angle} ${x} ${y})`}
              style={fill(token)}
            />
          ))}
          <path d={sparkle(184, 98, 7)} style={fill("brand-sparkle")} />
          <path d={sparkle(16, 96, 6)} style={fill("brand-pink")} />
        </g>
      );
  }
}
