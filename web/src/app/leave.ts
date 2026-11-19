import { useCallback, useEffect, useRef } from "react";

/**
 * Hands out AbortControllers that are aborted when the component leaves the page. The abort waits
 * a tick so StrictMode's rehearsal unmount (immediately followed by a remount) does not cut off a
 * stream that was started once on purpose.
 */
export function useAbortOnLeave(): () => AbortController {
  const live = useRef(new Set<AbortController>());
  const mounted = useRef(true);
  useEffect(() => {
    mounted.current = true;
    const all = live.current;
    return () => {
      mounted.current = false;
      window.setTimeout(() => {
        if (mounted.current) return;
        for (const c of all) c.abort();
        all.clear();
      }, 0);
    };
  }, []);
  return useCallback(() => {
    const c = new AbortController();
    live.current.add(c);
    c.signal.addEventListener("abort", () => live.current.delete(c));
    return c;
  }, []);
}
