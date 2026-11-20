import { type FormEvent, useState } from "react";
import { AppLink, Button, FormAlert, PasswordField, TextField } from "../../ui";
import { AuthFrame } from "./AuthFrame";
import { GoogleButton } from "./GoogleButton";
import type { StageOutcome } from "./MascotStage";

export type LoginValues = { readonly username: string; readonly password: string };

export type LoginScreenProps = {
  readonly onSubmit: (values: LoginValues) => void;
  readonly registerHref: string;
  readonly submitting?: boolean;
  /** A message for the whole form, e.g. wrong username or password. */
  readonly error?: string;
  /** The latest sign-in result, for the mascot (celebrates, or is sad and then confused). */
  readonly outcome?: StageOutcome;
  /** "Masuk dengan Google" goes here; no button when Google sign-in is off. */
  readonly googleHref?: string;
};

export function LoginScreen({
  onSubmit,
  registerHref,
  submitting = false,
  error,
  outcome,
  googleHref,
}: LoginScreenProps) {
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const submit = (e: FormEvent) => {
    e.preventDefault();
    onSubmit({ username: username.trim(), password });
  };
  const welcomed = outcome?.kind === "success";
  return (
    <AuthFrame
      title="Masuk"
      subtitle="Lanjutkan perjalanan belajarmu, dari SD hingga SMA."
      outcome={outcome}
    >
      <form onSubmit={submit} className="flex flex-col gap-4" noValidate>
        {/* Keyed by the attempt, so a second wrong password is read out again. */}
        {error !== undefined && !welcomed && (
          <FormAlert key={outcome?.key ?? 0} tone="error">
            {error}
          </FormAlert>
        )}
        {welcomed && <FormAlert tone="success">Berhasil masuk. Selamat datang kembali!</FormAlert>}
        <TextField
          label="Nama pengguna"
          value={username}
          onValueChange={setUsername}
          autoComplete="username"
          autoCapitalize="none"
          spellCheck={false}
          required
        />
        <PasswordField
          label="Kata sandi"
          value={password}
          onValueChange={setPassword}
          autoComplete="current-password"
          required
        />
        <Button
          type="submit"
          size="lg"
          block
          loading={submitting || welcomed}
          loadingLabel="Sebentar…"
          className="mt-1"
        >
          Masuk
        </Button>
      </form>
      {googleHref !== undefined && <GoogleButton href={googleHref} label="Masuk dengan Google" />}
      <p className="m-0 text-center text-[16px] font-semibold text-ink-muted">
        Belum punya akun?{" "}
        <AppLink
          href={registerHref}
          className="inline-flex min-h-12 items-center font-extrabold underline-offset-4 hover:underline"
        >
          Daftar
        </AppLink>
      </p>
    </AuthFrame>
  );
}
