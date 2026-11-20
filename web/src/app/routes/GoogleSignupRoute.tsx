import { useEffect, useState } from "react";
import { Navigate } from "react-router";
import { api } from "../../contracts/client";
import { profileMessage } from "../../contracts/messages";
import {
  type GoogleOutcome,
  type GoogleSignupErrors,
  GoogleSignupScreen,
  loginWithNotice,
} from "../../features/auth";
import { LoadingState } from "../../ui";
import { useAuth } from "../session";

type Pending =
  | { readonly status: "loading" }
  | { readonly status: "ready"; readonly suggestedNickname: string }
  | { readonly status: "gone"; readonly outcome: GoogleOutcome };

// After the grade is picked the session is signed in, and the route guard moves on to home.

export function GoogleSignupRoute() {
  const auth = useAuth();
  const [pending, setPending] = useState<Pending>({ status: "loading" });
  const [submitting, setSubmitting] = useState(false);
  const [serverErrors, setServerErrors] = useState<GoogleSignupErrors>({});
  const [error, setError] = useState<string | undefined>(undefined);

  useEffect(() => {
    const controller = new AbortController();
    void api.get("/v1/auth/google/signup", { signal: controller.signal }).then((r) => {
      if (controller.signal.aborted) return;
      if (r.ok) setPending({ status: "ready", suggestedNickname: r.value.suggestedNickname });
      // Expired (10 minutes) or opened in another browser: start again from the login page.
      else if (r.error.status === 404) setPending({ status: "gone", outcome: "failed" });
      else setPending({ status: "gone", outcome: "unavailable" });
    });
    return () => controller.abort();
  }, []);

  if (pending.status === "loading") return <LoadingState />;
  if (pending.status === "gone") return <Navigate to={loginWithNotice(pending.outcome)} replace />;
  return (
    <GoogleSignupScreen
      suggestedNickname={pending.suggestedNickname}
      submitting={submitting}
      serverErrors={serverErrors}
      {...(error !== undefined ? { error } : {})}
      onSubmit={async (values) => {
        setSubmitting(true);
        const r = await auth.googleSignup(values);
        setSubmitting(false);
        if (r.ok) return;
        if (r.error.code === "conflict") {
          setPending({ status: "gone", outcome: "conflict" });
          return;
        }
        if (r.error.status === 404) {
          setPending({ status: "gone", outcome: "failed" });
          return;
        }
        const m = profileMessage(r.error);
        setServerErrors({
          ...(m.fields.nickname !== undefined ? { nickname: m.fields.nickname } : {}),
          ...(m.fields.grade !== undefined ? { grade: m.fields.grade } : {}),
        });
        setError(m.form ?? undefined);
      }}
    />
  );
}
