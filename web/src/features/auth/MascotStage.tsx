import { useCallback, useEffect, useRef, useState } from "react";
import { cx, LogoMark, Mascot, type MascotExpression, useReducedMotion, Wordmark } from "../../ui";
import { hasWebGL, type Live, newLive, poseInput } from "./mascot3d/live";
import { decidePose, GREETING, isSleeping, pokeReaction, WAKE } from "./mascot3d/pose";

/** The latest sign-in result; a new `key` replays the reaction (a second wrong password). */
export type StageOutcome = { readonly kind: "success" | "failure"; readonly key: number };

export const TAGLINE = "Belajar Lebih Mudah, Lebih Pintar";
const LABEL = "Maskot belajar.ai. Tekan Enter untuk berinteraksi.";

/**
 * `?maskot=2d` keeps the 3D chunk from loading, to measure the page with and without it. Reduced
 * motion and a browser without WebGL get the SVG mascot too.
 */
const wants3d = (reduced: boolean): boolean => {
  if (reduced) return false;
  try {
    if (new URLSearchParams(window.location.search).get("maskot") === "2d") return false;
  } catch {
    return false;
  }
  return hasWebGL();
};

/**
 * The left half of the sign-in pages: the brand, and the mascot that follows the pointer, covers
 * its eyes for the password and reacts to the sign-in result. The SVG mascot shows at once; the
 * 3D one (three.js, its own chunk) replaces it when it has loaded.
 */
export function MascotStage({ outcome }: { readonly outcome?: StageOutcome | undefined }) {
  const reduced = useReducedMotion();
  const live = useRef<Live>(newLive(0));
  const host = useRef<HTMLDivElement>(null);
  const [in3d, setIn3d] = useState(false);
  const [expression, setExpression] = useState<MascotExpression>("happy");

  const poke = useCallback(() => {
    const l = live.current;
    const r = pokeReaction(poseInput(l, performance.now()), l.pokes);
    if (r === null) return;
    l.pokes += 1;
    l.reaction = r;
  }, []);

  // Greet once, and track activity (for the nap) and the pointer (for the eyes).
  useEffect(() => {
    const l = live.current;
    const now = performance.now();
    l.lastActivityAt = now;
    l.pointer = { x: window.innerWidth * 0.3, y: window.innerHeight * 0.45 };
    l.reaction = { pose: GREETING, at: now, ms: 2000 };
    const active = (e: Event) => {
      const t = performance.now();
      if (isSleeping(poseInput(l, t))) l.reaction = { pose: WAKE, at: t, ms: 900 };
      l.lastActivityAt = t;
      if (e instanceof PointerEvent) l.pointer = { x: e.clientX, y: e.clientY };
    };
    const kinds = ["pointermove", "pointerdown", "keydown", "wheel", "focusin"] as const;
    for (const k of kinds) window.addEventListener(k, active, { passive: true });
    return () => {
      for (const k of kinds) window.removeEventListener(k, active);
    };
  }, []);

  useEffect(() => {
    if (outcome === undefined) return;
    live.current.outcome = { kind: outcome.kind, at: performance.now() };
  }, [outcome]);

  // The 3D mascot, when wanted: fetched after first paint, disposed on unmount.
  useEffect(() => {
    const el = host.current;
    if (el === null || !wants3d(reduced)) return;
    let gone = false;
    let dispose: (() => void) | null = null;
    void import("./mascot3d/scene")
      .then(({ mountMascotScene }) => {
        if (gone) return;
        const scene = mountMascotScene({
          container: el,
          live: live.current,
          onPoke: poke,
          label: LABEL,
        });
        if (scene === null) return;
        dispose = scene.dispose;
        setIn3d(true);
      })
      .catch(() => {});
    return () => {
      gone = true;
      dispose?.();
      setIn3d(false);
    };
  }, [reduced, poke]);

  // The SVG mascot asks the same pose function a few times a second.
  useEffect(() => {
    if (in3d) return;
    const tick = () =>
      setExpression(decidePose(poseInput(live.current, performance.now())).expression);
    tick();
    const id = window.setInterval(tick, 250);
    return () => window.clearInterval(id);
  }, [in3d]);

  return (
    <section
      aria-label="Maskot belajar.ai"
      className="relative order-first flex h-[clamp(320px,50vh,480px)] items-center justify-center overflow-hidden border-b-2 border-line bg-surface-alt min-[861px]:sticky min-[861px]:top-0 min-[861px]:h-screen min-[861px]:border-r-2 min-[861px]:border-b-0"
    >
      <div className="absolute top-[clamp(18px,3vw,36px)] left-[clamp(18px,3vw,40px)] z-10 flex items-center gap-3">
        <LogoMark size={48} />
        <div className="flex flex-col">
          <Wordmark className="text-[24px] leading-none min-[861px]:text-[30px]" />
          <span className="text-[14px] font-semibold text-ink-muted">{TAGLINE}</span>
        </div>
      </div>
      {/* A calm flat shape behind the mascot (no glow): the owner's colour update. */}
      <div
        aria-hidden="true"
        className="absolute top-1/2 left-1/2 size-[clamp(250px,34vw,460px)] -translate-x-1/2 -translate-y-[42%] rounded-full bg-(--color-mascot-backdrop)"
      />
      {!in3d && (
        <Mascot
          expression={expression}
          size={320}
          animate
          interactive
          onActivate={poke}
          title={LABEL}
          className="relative h-auto w-[clamp(200px,26vw,320px)]"
        />
      )}
      <div
        ref={host}
        className={cx(
          "absolute inset-0",
          !in3d && "pointer-events-none",
          "[&>canvas]:block [&>canvas]:size-full [&>canvas]:touch-none [&>canvas]:outline-none [&>canvas:focus-visible]:shadow-[inset_0_0_0_3px_var(--focus)]",
        )}
      />
      <p className="pointer-events-none absolute right-0 bottom-[10px] left-0 m-0 px-4 text-center text-[13px] font-semibold text-ink-muted min-[861px]:bottom-[18px] min-[861px]:text-[14px]">
        Gerakkan kursor, klik maskot, atau isi formulir.
      </p>
    </section>
  );
}
