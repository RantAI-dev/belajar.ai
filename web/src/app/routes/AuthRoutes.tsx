import { useState } from "react";
import { useLocation } from "react-router";
import { loginMessage, registerMessage } from "../../contracts/messages";
import {
  celebrateMs,
  GOOGLE_START_HREF,
  googleNotice,
  LoginScreen,
  type RegisterErrors,
  RegisterScreen,
  type StageOutcome,
} from "../../features/auth";
import { useAuth } from "../session";
import { useGoogleEnabled } from "../useGoogleEnabled";

// After a successful sign-in the route guard (guard.ts) moves the student on, to `?ke=` or home.

const REGISTER_FIELDS = ["nickname", "username", "password", "grade"] as const;

const registerErrors = (fields: Partial<Record<string, string>>): RegisterErrors => {
  const out: { -readonly [K in keyof RegisterErrors]: RegisterErrors[K] } = {};
  for (const f of REGISTER_FIELDS) {
    const hint = fields[f];
    if (hint !== undefined) out[f] = hint;
  }
  return out;
};

/** The Google button's prop: only when the server has Google sign-in configured. */
const googleProp = (enabled: boolean) => (enabled ? { googleHref: GOOGLE_START_HREF } : {});

export function LoginRoute() {
  const { search } = useLocation();
  const auth = useAuth();
  const google = useGoogleEnabled();
  const [submitting, setSubmitting] = useState(false);
  // `?google=cancelled|conflict|unavailable|failed` from the server's Google callback.
  const [error, setError] = useState<string | undefined>(() => googleNotice(search));
  const [outcome, setOutcome] = useState<StageOutcome | undefined>(undefined);
  const next = (kind: StageOutcome["kind"]) =>
    setOutcome((o) => ({ kind, key: (o?.key ?? 0) + 1 }));
  return (
    <LoginScreen
      {...googleProp(google)}
      registerHref={`/daftar${search}`}
      submitting={submitting}
      {...(error !== undefined ? { error } : {})}
      {...(outcome !== undefined ? { outcome } : {})}
      onSubmit={async (values) => {
        setSubmitting(true);
        const r = await auth.login(values, {
          // The mascot celebrates for a moment before the guard moves on (at once under reduced motion).
          beforeSignIn: () => {
            next("success");
            return new Promise((done) => window.setTimeout(done, celebrateMs()));
          },
        });
        setSubmitting(false);
        if (!r.ok) {
          const m = loginMessage(r.error);
          // The login form has no per-field hints: a field error reads as one sentence.
          setError(m.form ?? Object.values(m.fields)[0]);
          next("failure");
        }
      }}
    />
  );
}

export function RegisterRoute() {
  const { search } = useLocation();
  const auth = useAuth();
  const google = useGoogleEnabled();
  const [submitting, setSubmitting] = useState(false);
  const [serverErrors, setServerErrors] = useState<RegisterErrors>({});
  const [error, setError] = useState<string | undefined>(undefined);
  return (
    <RegisterScreen
      {...googleProp(google)}
      loginHref={`/masuk${search}`}
      submitting={submitting}
      serverErrors={serverErrors}
      {...(error !== undefined ? { error } : {})}
      onSubmit={async (values) => {
        setSubmitting(true);
        const r = await auth.register(values);
        setSubmitting(false);
        if (r.ok) return;
        const m = registerMessage(r.error);
        setServerErrors(registerErrors(m.fields));
        setError(m.form ?? undefined);
      }}
    />
  );
}
