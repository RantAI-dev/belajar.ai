// The login page's 3D mascot: a procedural three.js port of the owner's prototype
// (masuk-belajar-ai.html), drawn to match the SVG mascot (ui/mascot) and its colour tokens.
// Loaded with import() only, so three.js is its own chunk and never in the first load.
import {
  BoxGeometry,
  BufferGeometry,
  CanvasTexture,
  CircleGeometry,
  Clock,
  Color,
  CylinderGeometry,
  DirectionalLight,
  Float32BufferAttribute,
  Group,
  HemisphereLight,
  type Material,
  Mesh,
  MeshBasicMaterial,
  MeshStandardMaterial,
  type Object3D,
  OctahedronGeometry,
  PerspectiveCamera,
  Raycaster,
  Scene,
  SphereGeometry,
  Sprite,
  SpriteMaterial,
  SRGBColorSpace,
  type Texture,
  Vector2,
  Vector3,
  WebGLRenderer,
} from "three";
import { MASCOT_FACES } from "../../../ui";
import { FACE_W, type FacePaint, paintFace } from "./face";
import { fieldPoint, type Live, poseInput } from "./live";
import { type Arm3d, decidePose, isSleeping, type Outcome, type Reaction } from "./pose";

export type MascotScene = { readonly dispose: () => void };
export type SceneHost = {
  /** The stage the canvas fills; it is sized and observed from here. */
  readonly container: HTMLElement;
  readonly live: Live;
  /** A click, Enter or Space on the mascot. */
  readonly onPoke: () => void;
  readonly label: string;
};

const clamp = (v: number, lo: number, hi: number) => Math.max(lo, Math.min(hi, v));
const token = (name: string): string =>
  getComputedStyle(document.documentElement).getPropertyValue(`--color-${name}`).trim();

// Hand positions [x, y, z] and rotations for the right arm; the left mirrors across x = 0.
const ARMS: Readonly<
  Record<Arm3d, { p: readonly [number, number, number]; r: readonly [number, number, number] }>
> = {
  rest: { p: [1.0, -0.28, 0.12], r: [0, 0, 0.55] },
  low: { p: [0.86, -0.6, 0.25], r: [0, 0, 0.2] },
  up: { p: [1.0, 0.66, 0.1], r: [0, 0, -0.25] },
  wave: { p: [1.1, 0.3, 0.15], r: [0, 0, -0.35] },
  chin: { p: [0.36, -0.42, 0.86], r: [0.3, 0, 0.9] },
  cheek: { p: [0.6, -0.12, 0.82], r: [0.2, 0, 0.5] },
  head: { p: [0.72, 0.72, 0.35], r: [0, 0, -0.9] },
  cover: { p: [0.3, 0.02, 0.97], r: [0.15, 0, 1.4] },
};

function bodyGeometry(top: Color, mid: Color, bottom: Color): SphereGeometry {
  const geo = new SphereGeometry(1, 96, 64);
  const pos = geo.attributes.position;
  const colours: number[] = [];
  const c = new Color();
  if (pos === undefined) return geo;
  for (let i = 0; i < pos.count; i++) {
    const x = pos.getX(i);
    const y = pos.getY(i);
    // A blob a little wider at the bottom, like the SVG body.
    const widen = 1 + Math.max(0, -y) * 0.06;
    pos.setX(i, x * widen);
    pos.setZ(i, pos.getZ(i) * widen);
    const g = clamp(((1 - y) / 2) * 0.85 + ((x + 1) / 2) * 0.15, 0, 1);
    if (g < 0.55) c.copy(top).lerp(mid, g / 0.55);
    else c.copy(mid).lerp(bottom, (g - 0.55) / 0.45);
    colours.push(c.r, c.g, c.b);
  }
  geo.setAttribute("color", new Float32BufferAttribute(colours, 3));
  geo.computeVertexNormals();
  return geo;
}

function glyphTexture(char: string, colour: string): Texture {
  const c = document.createElement("canvas");
  c.width = 64;
  c.height = 64;
  const g = c.getContext("2d");
  if (g !== null) {
    g.font = '800 52px "Plus Jakarta Sans", sans-serif';
    g.fillStyle = colour;
    g.textAlign = "center";
    g.textBaseline = "middle";
    g.fillText(char, 32, 34);
  }
  const t = new CanvasTexture(c);
  t.colorSpace = SRGBColorSpace;
  return t;
}

type Particle = {
  m: Object3D & { material: Material };
  v: Vector3;
  spin: Vector3;
  life: number;
  max: number;
  g: number;
  grow: boolean;
};

/** Mounts the scene in `host.container`. Null when WebGL fails to start. */
export function mountMascotScene(host: SceneHost): MascotScene | null {
  const { container, live } = host;
  const canvas = document.createElement("canvas");
  let renderer: WebGLRenderer;
  try {
    renderer = new WebGLRenderer({ canvas, antialias: true, alpha: true });
  } catch {
    return null;
  }
  canvas.tabIndex = 0;
  canvas.setAttribute("role", "button");
  canvas.setAttribute("aria-label", host.label);
  canvas.className = "mascot3d-canvas";
  container.append(canvas);

  const paint = {
    teal: new Color(token("brand-teal")),
    blue: new Color(token("brand-blue")),
    purple: new Color(token("brand-purple")),
    snow: token("snow"),
    arm: token("mascot-arm"),
    cap: token("mascot-cap"),
    band: token("mascot-cap-band"),
    accent: token("mascot-accent"),
    shadow: token("mascot-shadow"),
    confetti: ["brand-sparkle", "brand-pink", "brand-teal", "brand-purple", "brand-blue"].map(
      token,
    ),
  };
  const facePaint: FacePaint = {
    ink: token("mascot-face"),
    mouth: token("mascot-mouth"),
    tongue: token("mascot-tongue"),
    blush: token("mascot-blush"),
    snow: paint.snow,
  };

  const scene = new Scene();
  const camera = new PerspectiveCamera(30, 1, 0.1, 60);
  scene.add(new HemisphereLight(paint.snow, paint.purple, 2.2));
  const key = new DirectionalLight(paint.snow, 1.9);
  key.position.set(-2.5, 3.5, 4);
  const rim = new DirectionalLight(paint.teal, 1.4);
  rim.position.set(3, 1.5, -3);
  scene.add(key, rim);

  const root = new Group();
  const char = new Group();
  scene.add(root);
  root.add(char);
  const head = new Group();
  head.scale.set(1, 0.95, 1);
  char.add(head);
  const body = new Mesh(
    bodyGeometry(paint.teal, paint.blue, paint.purple),
    new MeshStandardMaterial({ vertexColors: true, roughness: 0.55 }),
  );
  head.add(body);

  // The face: a canvas texture on a patch of a slightly larger sphere, in front.
  const faceCanvas = document.createElement("canvas");
  faceCanvas.width = 768;
  faceCanvas.height = 576;
  const faceCtx = faceCanvas.getContext("2d");
  const faceTex = new CanvasTexture(faceCanvas);
  faceTex.colorSpace = SRGBColorSpace;
  faceTex.anisotropy = 4;
  const PL = 1.4;
  const TL = 1.05;
  const face = new Mesh(
    new SphereGeometry(1.006, 64, 48, Math.PI / 2 - PL / 2, PL, Math.PI / 2 - TL / 2, TL),
    new MeshStandardMaterial({
      map: faceTex,
      transparent: true,
      roughness: 0.6,
      depthWrite: false,
    }),
  );
  face.scale.set(1.03, 1, 1.03);
  head.add(face);

  // The mortarboard and its tassel, as in the SVG: a dark band, a lighter board.
  const capMat = new MeshStandardMaterial({ color: paint.cap, roughness: 0.6 });
  const bandMat = new MeshStandardMaterial({ color: paint.band, roughness: 0.6 });
  const cap = new Group();
  cap.position.set(0, 0.74, -0.04);
  cap.rotation.x = -0.12;
  char.add(cap);
  const skull = new Mesh(new CylinderGeometry(0.6, 0.67, 0.42, 48), bandMat);
  skull.position.y = 0.12;
  const board = new Mesh(new BoxGeometry(1.5, 0.08, 1.5), capMat);
  board.position.y = 0.36;
  board.rotation.y = Math.PI / 4;
  const button = new Mesh(new SphereGeometry(0.07, 16, 12), bandMat);
  button.position.y = 0.42;
  const cord = new Mesh(new CylinderGeometry(0.018, 0.018, 1, 8), bandMat);
  cord.rotation.z = Math.PI / 2;
  cord.position.set(0.5, 0.415, 0);
  cap.add(skull, board, button, cord);
  const tassel = new Group();
  tassel.position.set(1.0, 0.4, 0);
  cap.add(tassel);
  const hang = new Mesh(new CylinderGeometry(0.018, 0.018, 0.38, 8), bandMat);
  hang.position.y = -0.19;
  const tip = new Mesh(new CylinderGeometry(0.06, 0.08, 0.3, 16), capMat);
  tip.position.y = -0.5;
  tassel.add(hang, tip);

  // Two stubby arms.
  const armMat = new MeshStandardMaterial({ color: paint.arm, roughness: 0.5 });
  const handGeo = new SphereGeometry(1, 32, 24);
  const mkArm = (side: number) => {
    const m = new Mesh(handGeo, armMat);
    m.scale.set(0.26, 0.34, 0.24);
    m.position.set(ARMS.rest.p[0] * side, ARMS.rest.p[1], ARMS.rest.p[2]);
    m.rotation.z = ARMS.rest.r[2] * side;
    char.add(m);
    return m;
  };
  const armL = mkArm(-1);
  const armR = mkArm(1);
  const poseArm = (m: Mesh, name: Arm3d, side: number, k: number, t: number) => {
    const P = ARMS[name];
    let rz = P.r[2] * side;
    if (name === "wave") rz += Math.sin(t * 10) * 0.4 * side;
    if (name === "up") rz += Math.sin(t * 8) * 0.15 * side;
    m.position.x += (P.p[0] * side - m.position.x) * k;
    m.position.y += (P.p[1] - m.position.y) * k;
    m.position.z += (P.p[2] - m.position.z) * k;
    m.rotation.x += (P.r[0] - m.rotation.x) * k;
    m.rotation.z += (rz - m.rotation.z) * k;
  };

  // A flat ground shadow, like the SVG's ellipse.
  const shadow = new Mesh(
    new CircleGeometry(1, 48),
    new MeshBasicMaterial({ color: paint.shadow, transparent: true, depthWrite: false }),
  );
  shadow.rotation.x = -Math.PI / 2;
  shadow.scale.set(1.05, 0.5, 1);
  shadow.position.y = -1.3;
  scene.add(shadow);

  // Confetti, sparkles and the floating z / ? marks.
  const parts: Particle[] = [];
  const sparkGeo = new OctahedronGeometry(0.07);
  const confGeo = new BoxGeometry(0.07, 0.12, 0.01);
  const zTex = glyphTexture("z", paint.accent);
  const qTex = glyphTexture("?", paint.accent);
  const burst = (n: number, confetti: boolean) => {
    for (let i = 0; i < n; i++) {
      const m = new Mesh(
        confetti ? confGeo : sparkGeo,
        new MeshBasicMaterial({ color: paint.confetti[i % 5] ?? paint.snow, transparent: true }),
      );
      m.position.set(0, root.position.y + 0.7, 0.3);
      const a = Math.random() * Math.PI * 2;
      const sp = (confetti ? 2.2 : 1.6) + Math.random() * 1.6;
      const life = confetti ? 2.2 : 1.1;
      parts.push({
        m,
        v: new Vector3(Math.cos(a) * sp, 2 + Math.random() * 2.5, Math.sin(a) * sp * 0.4 + 0.4),
        spin: new Vector3(Math.random() * 8, Math.random() * 8, Math.random() * 8),
        life,
        max: life,
        g: confetti ? 4 : 6,
        grow: false,
      });
      scene.add(m);
    }
  };
  const glyph = (tex: Texture) => {
    const m = new Sprite(new SpriteMaterial({ map: tex, transparent: true, depthWrite: false }));
    m.position.set(0.75, root.position.y + 1.15, 0.3);
    scene.add(m);
    parts.push({
      m,
      v: new Vector3(0.25, 0.35, 0),
      spin: new Vector3(),
      life: 2.6,
      max: 2.6,
      g: 0,
      grow: true,
    });
  };

  // Hover and click by ray casting onto the mascot.
  const ray = new Raycaster();
  const ndc = new Vector2();
  const hitables = [body, armL, armR, board, skull];
  const hit = (e: PointerEvent) => {
    const r = canvas.getBoundingClientRect();
    ndc.set(((e.clientX - r.left) / r.width) * 2 - 1, -((e.clientY - r.top) / r.height) * 2 + 1);
    ray.setFromCamera(ndc, camera);
    return ray.intersectObjects(hitables, false).length > 0;
  };
  const onMove = (e: PointerEvent) => {
    const h = hit(e);
    if (h && !live.hover) sqV -= 1.2;
    live.hover = h;
    canvas.style.cursor = h ? "pointer" : "default";
  };
  const onLeave = () => {
    live.hover = false;
  };
  const onDown = (e: PointerEvent) => {
    if (!hit(e)) return;
    live.hover = e.pointerType === "mouse";
    host.onPoke();
  };
  const onKey = (e: KeyboardEvent) => {
    if (e.key !== "Enter" && e.key !== " ") return;
    e.preventDefault();
    host.onPoke();
  };
  canvas.addEventListener("pointermove", onMove);
  canvas.addEventListener("pointerleave", onLeave);
  canvas.addEventListener("pointerdown", onDown);
  canvas.addEventListener("keydown", onKey);

  const small = () => window.innerWidth <= 860;
  const resize = () => {
    const w = container.clientWidth;
    const h = container.clientHeight;
    if (w === 0 || h === 0) return;
    renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, small() ? 1.5 : 2));
    renderer.setSize(w, h, false);
    camera.aspect = w / h;
    const tanH = Math.tan((15 * Math.PI) / 180);
    const d = Math.max((small() ? 2.25 : 1.9) / tanH, 1.75 / camera.aspect / tanH);
    const ly = small() ? 0.3 : 0.1;
    camera.position.set(0, ly + 0.35, d);
    camera.lookAt(0, ly, 0);
    camera.updateProjectionMatrix();
  };
  const sizes = new ResizeObserver(resize);
  sizes.observe(container);
  resize();

  // Motion state, as in the prototype.
  let jumpY = 0;
  let jumpV = 0;
  let sq = 0;
  let sqV = 0;
  let ta = 0;
  let tav = 0;
  let prevRY = 0;
  let droop = 0;
  let blinkT = 0;
  let nextBlink = 1.5;
  let blush = 0.7;
  let lastFace = "";
  let glyphAt = 0;
  let seenOutcome: Outcome | null = null;
  let seenReaction: Reaction | null = null;
  const look = { x: 0, y: 0 };
  const kick = (v: number) => {
    sqV -= 2.6;
    if (v > 0 && jumpY < 0.05) jumpV = v;
  };

  const update = (dt: number, t: number) => {
    const now = performance.now();
    const input = poseInput(live, now);
    const st = decidePose(input);
    if (live.outcome !== seenOutcome) {
      seenOutcome = live.outcome;
      if (seenOutcome?.kind === "success") {
        burst(70, true);
        kick(3.6);
      } else if (seenOutcome?.kind === "failure") sqV += 2;
    }
    if (live.reaction !== seenReaction) {
      seenReaction = live.reaction;
      if (seenReaction !== null) {
        burst(14, false);
        kick(3.2);
      }
    }

    let target = live.pointer;
    if (st.look === "field") target = fieldPoint() ?? target;
    const cr = canvas.getBoundingClientRect();
    let dx = clamp((target.x - (cr.left + cr.width / 2)) / (window.innerWidth * 0.45), -1, 1);
    let dy = clamp((target.y - (cr.top + cr.height * 0.45)) / (window.innerHeight * 0.5), -1, 1);
    if (st.look === "none") {
      dx = 0;
      dy = 0;
    }
    const k = 1 - Math.exp(-dt * 7);
    look.x += (dx - look.x) * k;
    look.y += (dy - look.y) * k;
    const sleeping = isSleeping(input);
    droop += ((sleeping ? 0.3 : 0) - droop) * (1 - Math.exp(-dt * 2));
    const failAge = live.outcome?.kind === "failure" ? (now - live.outcome.at) / 1000 : 9;
    const shake = failAge < 0.7 ? Math.sin(t * 28) * 0.32 * (1 - failAge / 0.7) : 0;
    char.rotation.y = look.x * 0.55 + shake;
    char.rotation.x = look.y * 0.3 + droop;
    char.rotation.z = -look.x * 0.06;

    if (live.outcome?.kind === "success" && jumpY === 0 && jumpV === 0) jumpV = 3.2;
    jumpV -= 14 * dt;
    jumpY += jumpV * dt;
    if (jumpY <= 0) {
      if (jumpV < -1.5) sqV += jumpV * 0.7;
      jumpY = 0;
      jumpV = 0;
    }
    sqV += (-sq * 180 - sqV * 11) * dt;
    sq = clamp(sq + sqV * dt, -0.3, 0.3);
    root.scale.set(1 + sq * 0.5, 1 - sq, 1 + sq * 0.5);
    const fl = Math.sin(t * (sleeping ? 1 : 1.8)) * (sleeping ? 0.03 : 0.07);
    root.position.y = fl + jumpY - Math.max(0, sq) * 0.4;
    shadow.scale.set(1.05 * (1 - jumpY * 0.3 - fl * 0.8), 0.5 * (1 - jumpY * 0.3 - fl * 0.8), 1);

    const ka = 1 - Math.exp(-dt * 10);
    poseArm(armL, st.arms[0], -1, ka, t);
    poseArm(armR, st.arms[1], 1, ka, t);

    const row = MASCOT_FACES[st.expression];
    cap.rotation.z += ((-row.tilt * Math.PI) / 180 - cap.rotation.z) * k;
    const drive = (-(char.rotation.y - prevRY) / Math.max(dt, 1e-3)) * 1.6 - jumpV * 0.25;
    prevRY = char.rotation.y;
    tav += (-ta * 38 - tav * 3.2 + drive) * dt;
    ta = clamp(ta + tav * dt, -0.9, 0.9);
    tassel.rotation.z = ta + 0.1;
    tassel.rotation.x = Math.sin(t * 1.3) * 0.06 + ta * 0.3;

    if (t > nextBlink) {
      blinkT = 0.16;
      nextBlink = t + 2.2 + Math.random() * 3;
    }
    blinkT = Math.max(0, blinkT - dt);
    const open = blinkT > 0 ? Math.abs(Math.cos((1 - blinkT / 0.16) * Math.PI)) : 1;
    const warm = st.expression === "celebration" || st.mouth === "grin" || st.arms[0] === "cover";
    blush += ((warm ? 1 : 0.7) - blush) * k;
    const frame = {
      eyes: st.eyes ?? row.eyes,
      mouth: st.mouth ?? row.mouth,
      brow: st.brow ?? row.brow,
      dx: look.x * 3,
      dy: look.y * 2.2,
      open,
      blush,
    };
    const faceKey = `${frame.eyes.join()}|${frame.mouth}|${frame.brow}|${frame.dx.toFixed(1)}|${frame.dy.toFixed(1)}|${open.toFixed(2)}|${blush.toFixed(2)}`;
    if (faceKey !== lastFace && faceCtx !== null) {
      lastFace = faceKey;
      paintFace(faceCtx, facePaint, frame, faceCanvas.width / FACE_W);
      faceTex.needsUpdate = true;
    }

    for (let i = parts.length - 1; i >= 0; i--) {
      const p = parts[i];
      if (p === undefined) continue;
      p.life -= dt;
      p.v.y -= p.g * dt;
      p.m.position.addScaledVector(p.v, dt);
      p.m.rotation.x += p.spin.x * dt;
      p.m.rotation.y += p.spin.y * dt;
      p.m.material.opacity = clamp((p.life / p.max) * 1.5, 0, 1);
      if (p.grow) p.m.scale.setScalar(0.25 + (1 - p.life / p.max) * 0.3);
      if (p.life <= 0) {
        scene.remove(p.m);
        p.m.material.dispose();
        parts.splice(i, 1);
      }
    }
    const mark = sleeping ? zTex : st.expression === "confused" ? qTex : null;
    if (mark !== null && t > glyphAt) {
      glyphAt = t + 1.3;
      glyph(mark);
    }
  };

  // The loop runs only while the page is visible and the stage on screen.
  const clock = new Clock();
  let time = 0;
  let frameId = 0;
  let onScreen = true;
  const loop = () => {
    const dt = Math.min(clock.getDelta(), 0.05);
    time += dt;
    update(dt, time);
    renderer.render(scene, camera);
    frameId = requestAnimationFrame(loop);
  };
  const run = () => {
    cancelAnimationFrame(frameId);
    frameId = 0;
    if (!document.hidden && onScreen) {
      clock.getDelta();
      frameId = requestAnimationFrame(loop);
    }
  };
  const seen = new IntersectionObserver((entries) => {
    onScreen = entries.some((e) => e.isIntersecting);
    run();
  });
  seen.observe(container);
  document.addEventListener("visibilitychange", run);
  run();

  return {
    dispose: () => {
      cancelAnimationFrame(frameId);
      sizes.disconnect();
      seen.disconnect();
      document.removeEventListener("visibilitychange", run);
      canvas.removeEventListener("pointermove", onMove);
      canvas.removeEventListener("pointerleave", onLeave);
      canvas.removeEventListener("pointerdown", onDown);
      canvas.removeEventListener("keydown", onKey);
      const textures = new Set<Texture>([faceTex, zTex, qTex]);
      scene.traverse((o) => {
        if (o instanceof Mesh || o instanceof Sprite) {
          const geo: unknown = o.geometry;
          if (geo instanceof BufferGeometry) geo.dispose();
          const mats: Material[] = Array.isArray(o.material) ? o.material : [o.material];
          for (const m of mats) m.dispose();
        }
      });
      sparkGeo.dispose();
      confGeo.dispose();
      for (const t of textures) t.dispose();
      renderer.dispose();
      renderer.forceContextLoss();
      canvas.remove();
    },
  };
}
