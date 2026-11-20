import { useEffect, useState } from "react";

/**
 * A number that changes whenever the colours may have changed: `data-theme` on <html> (set by the
 * theme switcher) or the system's light/dark preference. Works before either exists.
 */
export function useThemeVersion(): number {
  const [version, setVersion] = useState(0);
  useEffect(() => {
    const bump = () => setVersion((n) => n + 1);
    const observer = new MutationObserver(bump);
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["data-theme"],
    });
    const query =
      typeof window.matchMedia === "function"
        ? window.matchMedia("(prefers-color-scheme: dark)")
        : null;
    query?.addEventListener("change", bump);
    return () => {
      observer.disconnect();
      query?.removeEventListener("change", bump);
    };
  }, []);
  return version;
}
