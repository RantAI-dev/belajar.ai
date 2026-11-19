import { useState } from "react";
import { useNavigate } from "react-router";
import {
  LOGIN_ERROR,
  LoginScreen,
  PASSWORD_MIN,
  type RegisterErrors,
  RegisterScreen,
  TAKEN_USERNAMES,
} from "../../features/auth";
import { useDemo } from "../demo";

const FAKE_WAIT_MS = 700;

export function LoginRoute() {
  const navigate = useNavigate();
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | undefined>(undefined);
  return (
    <LoginScreen
      registerHref="/daftar"
      submitting={submitting}
      {...(error !== undefined ? { error } : {})}
      onSubmit={({ username, password }) => {
        setSubmitting(true);
        window.setTimeout(() => {
          setSubmitting(false);
          // The prototype accepts any username with a plausible password.
          if (username.length === 0 || password.length < PASSWORD_MIN) setError(LOGIN_ERROR);
          else navigate("/");
        }, FAKE_WAIT_MS);
      }}
    />
  );
}

export function RegisterRoute() {
  const navigate = useNavigate();
  const demo = useDemo();
  const [submitting, setSubmitting] = useState(false);
  const [serverErrors, setServerErrors] = useState<RegisterErrors>({});
  return (
    <RegisterScreen
      loginHref="/masuk"
      submitting={submitting}
      serverErrors={serverErrors}
      onSubmit={(values) => {
        setSubmitting(true);
        window.setTimeout(() => {
          setSubmitting(false);
          if (TAKEN_USERNAMES.includes(values.username)) {
            setServerErrors({ username: "Nama pengguna ini sudah dipakai. Coba yang lain, ya." });
            return;
          }
          demo.setProfile({
            nickname: values.nickname,
            username: values.username,
            grade: values.grade,
          });
          navigate("/");
        }, FAKE_WAIT_MS);
      }}
    />
  );
}
