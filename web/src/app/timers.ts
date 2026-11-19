import { useEffect, useRef } from "react";

/** Calls `tick` every `ms` while `ms` is not null; the latest `tick` is always used. */
export function useInterval(tick: () => void, ms: number | null): void {
  const saved = useRef(tick);
  saved.current = tick;
  useEffect(() => {
    if (ms === null) return;
    const id = window.setInterval(() => saved.current(), ms);
    return () => window.clearInterval(id);
  }, [ms]);
}
