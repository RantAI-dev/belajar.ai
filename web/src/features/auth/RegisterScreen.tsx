import { type FormEvent, useState } from "react";
import { AppLink, Button, TextField } from "../../ui";
import { AuthFrame } from "./AuthFrame";
import { GradePicker } from "./GradePicker";
import type { Grade } from "./grades";
import { hasErrors, PASSWORD_MIN, type RegisterErrors, validateRegister } from "./validate";

export type RegisterSubmit = {
  readonly nickname: string;
  readonly username: string;
  readonly password: string;
  readonly grade: Grade;
};

export type RegisterScreenProps = {
  readonly onSubmit: (values: RegisterSubmit) => void;
  readonly loginHref: string;
  readonly submitting?: boolean;
  /** Errors the server sent back, by field (e.g. the username is taken). */
  readonly serverErrors?: RegisterErrors;
};

const errorProp = (message: string | undefined) =>
  message === undefined ? {} : { error: message };

export function RegisterScreen({
  onSubmit,
  loginHref,
  submitting = false,
  serverErrors = {},
}: RegisterScreenProps) {
  const [nickname, setNickname] = useState("");
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [grade, setGrade] = useState<Grade | null>(null);
  const [errors, setErrors] = useState<RegisterErrors>({});

  const submit = (e: FormEvent) => {
    e.preventDefault();
    const values = { nickname: nickname.trim(), username: username.trim(), password, grade };
    const found = validateRegister(values);
    setErrors(found);
    if (!hasErrors(found) && values.grade !== null) onSubmit({ ...values, grade: values.grade });
  };
  const shown: RegisterErrors = { ...serverErrors, ...errors };

  return (
    <AuthFrame title="Daftar" greeting="Hai! Aku teman belajarmu. Kenalan dulu, yuk!">
      <form onSubmit={submit} className="flex flex-col gap-4" noValidate>
        <TextField
          label="Nama panggilan"
          hint="Nama ini yang aku pakai untuk menyapamu."
          value={nickname}
          onValueChange={setNickname}
          autoComplete="nickname"
          maxLength={30}
          {...errorProp(shown.nickname)}
        />
        <TextField
          label="Nama pengguna"
          hint="Untuk masuk. Huruf kecil, angka, titik atau garis bawah."
          value={username}
          onValueChange={setUsername}
          autoComplete="username"
          autoCapitalize="none"
          {...errorProp(shown.username)}
        />
        <TextField
          label="Kata sandi"
          type="password"
          hint={`Minimal ${PASSWORD_MIN} karakter. Jangan beri tahu teman, ya.`}
          value={password}
          onValueChange={setPassword}
          autoComplete="new-password"
          {...errorProp(shown.password)}
        />
        <GradePicker value={grade} onChange={setGrade} {...errorProp(shown.grade)} />
        <Button
          type="submit"
          size="lg"
          block
          loading={submitting}
          loadingLabel="Menyiapkan akunmu…"
        >
          Daftar dan mulai belajar
        </Button>
      </form>
      <p className="m-0 text-center text-[16px] font-bold text-ink-muted">
        Sudah punya akun?{" "}
        <AppLink
          href={loginHref}
          className="inline-flex min-h-12 items-center font-extrabold underline-offset-4 hover:underline"
        >
          Masuk
        </AppLink>
      </p>
    </AuthFrame>
  );
}
