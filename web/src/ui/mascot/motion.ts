import { useSyncExternalStore } from "react";

const QUERY = "(prefers-reduced-motion: reduce)";

const media = (): MediaQueryList | null => {
  try {
    return typeof window === "undefined" ? null : window.matchMedia(QUERY);
  } catch {
    return null;
  }
};

const subscribe = (onChange: () => void) => {
  const query = media();
  query?.addEventListener("change", onChange);
  return () => query?.removeEventListener("change", onChange);
};

/** True when the student asked the system for less motion (and while rendering on the server). */
export const useReducedMotion = (): boolean =>
  useSyncExternalStore(
    subscribe,
    () => media()?.matches ?? false,
    () => true,
  );
