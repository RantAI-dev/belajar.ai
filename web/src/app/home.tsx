import {
  createContext,
  type ReactNode,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
} from "react";
import type { components } from "../contracts/api.gen";
import { api } from "../contracts/client";
import type { ApiProblem } from "../contracts/problem";
import { useAuth } from "./session";

export type HomeDto = components["schemas"]["HomeDto"];

export type HomeState = {
  /** Null until the first GET /v1/home answers. */
  readonly home: HomeDto | null;
  readonly problem: ApiProblem | null;
  /** Loads GET /v1/home again: after anything that earns stars or moves a mission. */
  readonly refresh: () => Promise<void>;
};

const HomeContext = createContext<HomeState | null>(null);

/** Holds GET /v1/home for the signed-in student: Beranda, the header's stars and streak, Akun. */
export function HomeProvider({ children }: { readonly children: ReactNode }) {
  const studentId = useAuth().student?.id ?? null;
  const [home, setHome] = useState<HomeDto | null>(null);
  const [problem, setProblem] = useState<ApiProblem | null>(null);

  const refresh = useCallback(async () => {
    const r = await api.get("/v1/home");
    if (r.ok) {
      setHome(r.value);
      setProblem(null);
    } else {
      setProblem(r.error);
    }
  }, []);

  useEffect(() => {
    setHome(null);
    if (studentId !== null) void refresh();
  }, [studentId, refresh]);

  const value = useMemo<HomeState>(() => ({ home, problem, refresh }), [home, problem, refresh]);
  return <HomeContext.Provider value={value}>{children}</HomeContext.Provider>;
}

const OUTSIDE: HomeState = { home: null, problem: null, refresh: async () => {} };

export const useHome = (): HomeState => useContext(HomeContext) ?? OUTSIDE;
