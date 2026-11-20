import type { components } from "../contracts/api.gen";

/**
 * Sistem / Terang / Gelap. `system` follows prefers-color-scheme. The resolved theme is written to
 * `<html data-theme>` (the theme contract in theme.css); web/index.html does the same before the
 * first paint, from the same storage key, so the page never flashes the other theme.
 */
export type ThemePreference = components["schemas"]["ThemePreferenceDto"];
export type ResolvedTheme = "light" | "dark";

export const THEME_PREFERENCES: readonly ThemePreference[] = ["system", "light", "dark"];
/** Read by the inline script in web/index.html too; keep the two the same. */
export const THEME_STORAGE_KEY = "belajar.theme";
export const DARK_QUERY = "(prefers-color-scheme: dark)";

export const isThemePreference = (value: unknown): value is ThemePreference =>
  value === "system" || value === "light" || value === "dark";

export const resolveTheme = (preference: ThemePreference, prefersDark: boolean): ResolvedTheme =>
  preference === "system" ? (prefersDark ? "dark" : "light") : preference;

type ReadStorage = { getItem(key: string): string | null };
type WriteStorage = { setItem(key: string, value: string): void };

/** The choice saved on this device, or `system`. Storage can be missing or throw (private mode). */
export const readStoredTheme = (storage: () => ReadStorage | null): ThemePreference => {
  try {
    const value = storage()?.getItem(THEME_STORAGE_KEY);
    return isThemePreference(value) ? value : "system";
  } catch {
    return "system";
  }
};

/** Saves the choice on this device; a storage that throws only loses the fallback. */
export const storeTheme = (storage: () => WriteStorage | null, preference: ThemePreference) => {
  try {
    storage()?.setItem(THEME_STORAGE_KEY, preference);
  } catch {
    // Without storage the server value (after login) or the system setting still applies.
  }
};

type ThemeRoot = { setAttribute(name: string, value: string): void };

export const applyTheme = (root: ThemeRoot, theme: ResolvedTheme) => {
  root.setAttribute("data-theme", theme);
};
