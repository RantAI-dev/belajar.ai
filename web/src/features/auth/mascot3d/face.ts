// The 3D mascot's face, painted on a canvas that is wrapped on the front of the body. It is the
// SVG face of ui/mascot/face.tsx redrawn on a 2D canvas: the same parts, coordinates (the SVG's
// 200x200 viewBox) and colours, so the 3D mascot and the SVG one look alike.
import type { MascotBrow, MascotEye, MascotMouth } from "../../../ui";

export type FacePaint = {
  readonly ink: string;
  readonly mouth: string;
  readonly tongue: string;
  readonly blush: string;
  readonly snow: string;
};

export type FaceFrame = {
  readonly eyes: readonly [MascotEye, MascotEye];
  readonly mouth: MascotMouth;
  readonly brow: MascotBrow;
  /** Look offset in viewBox units. */
  readonly dx: number;
  readonly dy: number;
  /** 1 open, 0 shut (a blink). */
  readonly open: number;
  readonly blush: number;
};

// The face centre in the viewBox, where the texture centre goes, and the texture pixels per unit.
const CX = 100;
const CY = 120;
const SCALE = 10;
export const FACE_W = 1024;
export const FACE_H = 768;
const EYE_Y = 116;
const EYE_X = [78, 122] as const;

const BROWS: Readonly<Record<Exclude<MascotBrow, "none">, string>> = {
  sad: "M68 103L86 97M114 97L132 103",
  raised: "M70 99Q78 92 86 99M114 99Q122 92 130 99",
  one: "M70 101L86 101M114 96L131 91",
};
const MOUTH_STROKES: Partial<Record<MascotMouth, readonly [string, number]>> = {
  smile: ["M90 133Q100 143 110 133", 4],
  side: ["M98 138Q105 141 112 136", 3.5],
  frown: ["M91 142Q100 133 109 142", 4],
  wavy: ["M88 138q3-5 6 0t6 0t6 0t6 0", 3.5],
};

const ellipse = (g: CanvasRenderingContext2D, x: number, y: number, rx: number, ry: number) => {
  g.beginPath();
  g.ellipse(x, y, rx, Math.max(ry, 0.3), 0, 0, Math.PI * 2);
  g.fill();
};

const strokePath = (g: CanvasRenderingContext2D, d: string, width: number) => {
  g.lineWidth = width;
  g.stroke(new Path2D(d));
};

function eye(g: CanvasRenderingContext2D, p: FacePaint, kind: MascotEye, x: number, open: number) {
  const y = EYE_Y;
  g.fillStyle = p.ink;
  switch (kind) {
    case "open":
    case "big":
    case "sad":
    case "up": {
      const [rx, ry, shine] =
        kind === "big" ? [8.5, 11.5, 3.2] : kind === "sad" ? [7.5, 9, 3.6] : [7, 9.5, 2.6];
      const [cx, cy] = kind === "up" ? [x + 2, y - 3] : [x, y];
      const look = kind === "up" ? 1.5 : 0;
      ellipse(g, cx, cy, rx, ry * Math.max(open, 0.08));
      if (open > 0.5) {
        g.fillStyle = p.snow;
        ellipse(g, cx - 2.2 + look, cy - 3.5 - look, shine, shine);
        ellipse(g, cx + 2.2, cy + 3.2, 1.2, 1.2);
      }
      return;
    }
    case "happy":
      return strokePath(g, `M${x - 8} ${y + 3}Q${x} ${y - 8} ${x + 8} ${y + 3}`, 4);
    case "sleep":
      return strokePath(g, `M${x - 8} ${y - 1}Q${x} ${y + 7} ${x + 8} ${y - 1}`, 4);
    case "gt":
    case "lt": {
      const s = kind === "gt" ? 1 : -1;
      return strokePath(g, `M${x - 6 * s} ${y - 7}L${x + 6 * s} ${y}L${x - 6 * s} ${y + 7}`, 4);
    }
  }
}

function mouth(g: CanvasRenderingContext2D, p: FacePaint, kind: MascotMouth) {
  const line = MOUTH_STROKES[kind];
  if (line !== undefined) return strokePath(g, line[0], line[1]);
  if (kind === "grin") {
    g.fillStyle = p.mouth;
    g.fill(new Path2D("M88 131Q100 129 112 131Q111 149 100 150Q89 149 88 131Z"));
    g.fillStyle = p.tongue;
    return ellipse(g, 100, 144, 6.5, 3.8);
  }
  g.fillStyle = kind === "o" ? p.mouth : p.ink;
  if (kind === "o") return ellipse(g, 100, 139, 6, 8);
  return ellipse(g, 100, 137, 4, 3);
}

/** Paints one frame of the face onto `g`, a FACE_W x FACE_H canvas (or scaled to one). */
export function paintFace(g: CanvasRenderingContext2D, p: FacePaint, f: FaceFrame, scale = 1) {
  g.setTransform(scale, 0, 0, scale, 0, 0);
  g.clearRect(0, 0, FACE_W, FACE_H);
  g.translate(FACE_W / 2, FACE_H / 2);
  g.scale(SCALE, SCALE);
  g.translate(-CX + f.dx, -CY + f.dy);
  g.lineCap = "round";
  g.lineJoin = "round";
  g.strokeStyle = p.ink;
  g.globalAlpha = 0.6 * f.blush;
  g.fillStyle = p.blush;
  ellipse(g, 64, 134, 10, 6);
  ellipse(g, 136, 134, 10, 6);
  g.globalAlpha = 1;
  eye(g, p, f.eyes[0], EYE_X[0], f.open);
  eye(g, p, f.eyes[1], EYE_X[1], f.open);
  if (f.brow !== "none") strokePath(g, BROWS[f.brow], 3.5);
  mouth(g, p, f.mouth);
}
