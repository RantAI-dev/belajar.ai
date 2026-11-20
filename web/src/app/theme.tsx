import {
  createContext,
  type ReactNode,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
} from "react";
import type { ApiResult } from "../contracts/client";
import { ok } from "../contracts/result";
import {
  applyTheme,
  DARK_QUERY,
  type ResolvedTheme,
  readStoredTheme,
  resolveTheme,
  storeTheme,
  type ThemePreference,
} from "../ui";
import { useAuth } from "./session";

export type Theme = {
  readonly preference: ThemePreference;
  readonly resolved: ResolvedTheme;
  /** Applies at once and saves on this device; for a signed-in student also on the profile. */
  readonly setPreference: (preference: ThemePreference) => Promise<ApiResult<undefined>>;
};

const storage = () => window.localStorage;

const systemPrefersDark = (): boolean => {
  try {
    return window.matchMedia(DARK_QUERY).matches;
  } catch {
    return false;
  }
};

const ThemeContext = createContext<Theme | null>(null);

/**
 * Holds the theme for the whole app. Before login the device's saved choice (or the system)
 * applies, as the boot script in index.html already did; once the student is known their profile's
 * choice wins and is saved on the device. "Sistem" follows live changes of the system setting.
 */
export function ThemeProvider({ children }: { readonly children: ReactNode }) {
  const auth = useAuth();
  const [preference, setLocal] = useState<ThemePreference>(() => readStoredTheme(storage));
  const [prefersDark, setPrefersDark] = useState(systemPrefersDark);

  useEffect(() => {
    let query: MediaQueryList;
    try {
      query = window.matchMedia(DARK_QUERY);
    } catch {
      return;
    }
    const onChange = (e: MediaQueryListEvent) => setPrefersDark(e.matches);
    query.addEventListener("change", onChange);
    return () => query.removeEventListener("change", onChange);
  }, []);

  const serverTheme = auth.student?.theme;
  useEffect(() => {
    if (serverTheme === undefined) return;
    setLocal(serverTheme);
    storeTheme(storage, serverTheme);
  }, [serverTheme]);

  const resolved = resolveTheme(preference, prefersDark);
  useEffect(() => applyTheme(document.documentElement, resolved), [resolved]);

  const { student, updateProfile } = auth;
  const setPreference = useCallback<Theme["setPreference"]>(
    async (next) => {
      setLocal(next);
      storeTheme(storage, next);
      if (student === null) return ok(undefined);
      const r = await updateProfile({ theme: next });
      return r.ok ? ok(undefined) : r;
    },
    [student, updateProfile],
  );

  const value = useMemo<Theme>(
    () => ({ preference, resolved, setPreference }),
    [preference, resolved, setPreference],
  );
  return <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>;
}

const OUTSIDE: Theme = {
  preference: "system",
  resolved: "light",
  setPreference: async () => ok(undefined),
};

export const useTheme = (): Theme => useContext(ThemeContext) ?? OUTSIDE;
