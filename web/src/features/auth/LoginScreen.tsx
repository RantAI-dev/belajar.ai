import { type FormEvent, useState } from "react";
import { AppLink, Button, FormAlert, TextField } from "../../ui";
import { AuthFrame } from "./AuthFrame";
import { GoogleButton } from "./GoogleButton";

export type LoginValues = { readonly username: string; readonly password: string };

export type LoginScreenProps = {
  readonly onSubmit: (values: LoginValues) => void;
  readonly registerHref: string;
  readonly submitting?: boolean;
  /** A message for the whole form, e.g. wrong username or password. */
  readonly error?: string;
  /** "Masuk dengan Google" goes here; no button when Google sign-in is off. */
  readonly googleHref?: string;
};

export function LoginScreen({
  onSubmit,
  registerHref,
  submitting = false,
  error,
  googleHref,
}: LoginScreenProps) {
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const submit = (e: FormEvent) => {
    e.preventDefault();
    onSubmit({ username: username.trim(), password });
  };
  return (
    <AuthFrame title="Masuk" greeting="Halo lagi! Masuk dulu, yuk, lalu kita lanjut belajar.">
      <form onSubmit={submit} className="flex flex-col gap-4" noValidate>
        {error !== undefined && <FormAlert tone="error">{error}</FormAlert>}
        <TextField
          label="Nama pengguna"
          value={username}
          onValueChange={setUsername}
          autoComplete="username"
          autoCapitalize="none"
          required
        />
        <TextField
          label="Kata sandi"
          type="password"
          value={password}
          onValueChange={setPassword}
          autoComplete="current-password"
          required
        />
        <Button type="submit" size="lg" block loading={submitting} loadingLabel="Sebentar…">
          Masuk
        </Button>
      </form>
      {googleHref !== undefined && <GoogleButton href={googleHref} label="Masuk dengan Google" />}
      <p className="m-0 text-center text-[16px] font-bold text-ink-muted">
        Belum punya akun?{" "}
        <AppLink
          href={registerHref}
          className="inline-flex min-h-12 items-center font-extrabold underline-offset-4 hover:underline"
        >
          Daftar di sini
        </AppLink>
      </p>
    </AuthFrame>
  );
}
