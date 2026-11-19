import { useState } from "react";
import { useLocation } from "react-router";
import { loginMessage, registerMessage } from "../../contracts/messages";
import { LoginScreen, type RegisterErrors, RegisterScreen } from "../../features/auth";
import { useAuth } from "../session";

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

export function LoginRoute() {
  const { search } = useLocation();
  const auth = useAuth();
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | undefined>(undefined);
  return (
    <LoginScreen
      registerHref={`/daftar${search}`}
      submitting={submitting}
      {...(error !== undefined ? { error } : {})}
      onSubmit={async (values) => {
        setSubmitting(true);
        const r = await auth.login(values);
        setSubmitting(false);
        if (!r.ok) {
          const m = loginMessage(r.error);
          // The login form has no per-field hints: a field error reads as one sentence.
          setError(m.form ?? Object.values(m.fields)[0]);
        }
      }}
    />
  );
}

export function RegisterRoute() {
  const { search } = useLocation();
  const auth = useAuth();
  const [submitting, setSubmitting] = useState(false);
  const [serverErrors, setServerErrors] = useState<RegisterErrors>({});
  const [error, setError] = useState<string | undefined>(undefined);
  return (
    <RegisterScreen
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
