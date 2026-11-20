import { type ReactNode, useCallback, useEffect, useRef, useState } from "react";
import { Mascot, type MascotExpression } from "../../ui";
import { type HeroMood, REACTION_MS, reactionFrames, reactionLine } from "./mascotMood";

const prefersReducedMotion = (): boolean => {
  try {
    return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  } catch {
    return false;
  }
};

type Reaction = { readonly expression: MascotExpression; readonly line: string };

/** A tap (or Enter/Space) plays a short reaction with a one-liner, then the mood comes back. */
function useReaction() {
  const [reaction, setReaction] = useState<Reaction | null>(null);
  const taps = useRef(0);
  const timers = useRef<number[]>([]);
  const clear = useCallback(() => {
    for (const t of timers.current) window.clearTimeout(t);
    timers.current = [];
  }, []);
  useEffect(() => clear, [clear]);
  const react = () => {
    clear();
    const line = reactionLine(taps.current);
    taps.current += 1;
    for (const { expression, at } of reactionFrames(prefersReducedMotion())) {
      if (at === 0) setReaction({ expression, line });
      else timers.current.push(window.setTimeout(() => setReaction({ expression, line }), at));
    }
    timers.current.push(window.setTimeout(() => setReaction(null), REACTION_MS));
  };
  return { reaction, react };
}

/**
 * The big mascot, the greeting and its speech bubble at the top of Beranda. `children` (the quick
 * question) sits under the greeting on wide screens and full width under the mascot on phones,
 * so the hero stays short there.
 */
export function MascotHero({
  name,
  mood,
  children,
}: {
  readonly name: string;
  readonly mood: HeroMood;
  readonly children: ReactNode;
}) {
  const { reaction, react } = useReaction();
  return (
    <div className="grid grid-cols-[140px_minmax(0,1fr)] items-center gap-x-3 gap-y-4 sm:grid-cols-[224px_minmax(0,1fr)] sm:gap-x-8">
      <Mascot
        expression={reaction?.expression ?? mood.expression}
        size={224}
        backdrop="light"
        animate
        interactive
        onActivate={react}
        title="Sapa maskot belajar.ai"
        className="max-sm:size-[140px] sm:row-span-2"
      />
      <div className="flex min-w-0 flex-col gap-3 sm:self-end">
        <h1 className="m-0 font-display text-[26px] font-semibold leading-tight sm:text-[40px]">
          Halo, {name}!
        </h1>
        <p
          aria-live="polite"
          className="m-0 self-start rounded-[16px] rounded-tl-[4px] bg-surface px-3.5 py-2.5 text-[16px] font-bold text-ink sm:px-4 sm:text-[19px]"
        >
          {reaction?.line ?? mood.line}
        </p>
      </div>
      <div className="col-span-2 flex min-w-0 flex-col gap-3 sm:col-span-1 sm:col-start-2 sm:self-start">
        {children}
      </div>
    </div>
  );
}
