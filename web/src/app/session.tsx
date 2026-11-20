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
import { type ApiResult, api } from "../contracts/client";
import { type ApiProblem, setSessionExpiredListener } from "../contracts/problem";
import { err, ok } from "../contracts/result";
import { type Grade, isGrade } from "../features/auth";
import type { ThemePreference } from "../ui";

type StudentDto = components["schemas"]["StudentDto"];

export type Student = {
  readonly id: string;
  readonly username: string;
  readonly nickname: string;
  readonly grade: Grade;
  readonly theme: ThemePreference;
  readonly createdAt: string;
};

/** The server keeps grades in 1–12; anything else is a broken answer, not a student. */
export const toStudent = (dto: StudentDto): Student | null =>
  isGrade(dto.grade) ? { ...dto, grade: dto.grade } : null;

const brokenAnswer: ApiProblem = {
  status: 200,
  code: "internal",
  fieldErrors: {},
  retryAfterSeconds: null,
  requestId: "",
};

const asStudent = (r: ApiResult<StudentDto>): ApiResult<Student> => {
  if (!r.ok) return r;
  const s = toStudent(r.value);
  return s === null ? err(brokenAnswer) : ok(s);
};

export type SessionState =
  | { readonly status: "loading" }
  | { readonly status: "failed"; readonly problem: ApiProblem }
  /** `loggedOut`: the student pressed "Keluar" (the guard then forgets the page they were on). */
  | { readonly status: "signedOut"; readonly loggedOut: boolean }
  | { readonly status: "signedIn"; readonly student: Student };

export type Auth = {
  readonly state: SessionState;
  readonly student: Student | null;
  readonly login: (v: { username: string; password: string }) => Promise<ApiResult<Student>>;
  readonly register: (v: {
    username: string;
    password: string;
    nickname: string;
    grade: Grade;
  }) => Promise<ApiResult<Student>>;
  readonly logout: () => Promise<ApiResult<undefined>>;
  /** Loads the student again (on start, and after "coba lagi" on the failed screen). */
  readonly refresh: () => Promise<void>;
  readonly updateProfile: (p: {
    nickname?: string;
    grade?: Grade;
    theme?: ThemePreference;
  }) => Promise<ApiResult<Student>>;
};

const AuthContext = createContext<Auth | null>(null);

/** Loads GET /v1/me once on start and holds the signed-in student for every route. */
export function AuthProvider({ children }: { readonly children: ReactNode }) {
  const [state, setState] = useState<SessionState>({ status: "loading" });

  const refresh = useCallback(async () => {
    const r = asStudent(await api.get("/v1/me"));
    if (r.ok) setState({ status: "signedIn", student: r.value });
    else if (r.error.status === 401) setState({ status: "signedOut", loggedOut: false });
    else setState({ status: "failed", problem: r.error });
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  // The route guard sends the student to /masuk and remembers where they were.
  useEffect(() => {
    setSessionExpiredListener(() => setState({ status: "signedOut", loggedOut: false }));
    return () => setSessionExpiredListener(null);
  }, []);

  const signedIn = useCallback((r: ApiResult<Student>): ApiResult<Student> => {
    if (r.ok) setState({ status: "signedIn", student: r.value });
    return r;
  }, []);

  const login = useCallback<Auth["login"]>(
    async (v) => signedIn(asStudent(await api.post("/v1/auth/login", v))),
    [signedIn],
  );
  const register = useCallback<Auth["register"]>(
    async (v) => signedIn(asStudent(await api.post("/v1/auth/register", v))),
    [signedIn],
  );
  const updateProfile = useCallback<Auth["updateProfile"]>(
    async (p) => signedIn(asStudent(await api.patch("/v1/me", p))),
    [signedIn],
  );

  const logout = useCallback(async () => {
    const r = await api.post("/v1/auth/logout", {});
    // A 401 means the session was already gone: signed out either way.
    if (r.ok || r.error.status === 401) {
      setState({ status: "signedOut", loggedOut: true });
      return ok(undefined);
    }
    return r;
  }, []);

  const value = useMemo<Auth>(
    () => ({
      state,
      student: state.status === "signedIn" ? state.student : null,
      login,
      register,
      logout,
      refresh,
      updateProfile,
    }),
    [state, login, register, logout, refresh, updateProfile],
  );
  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

const OUTSIDE: Auth = {
  state: { status: "loading" },
  student: null,
  login: async () => err(brokenAnswer),
  register: async () => err(brokenAnswer),
  logout: async () => err(brokenAnswer),
  refresh: async () => {},
  updateProfile: async () => err(brokenAnswer),
};

export const useAuth = (): Auth => useContext(AuthContext) ?? OUTSIDE;
