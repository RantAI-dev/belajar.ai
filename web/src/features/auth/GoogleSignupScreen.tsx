import { type FormEvent, useState } from "react";
import { Button, FormAlert, TextField } from "../../ui";
import { AuthFrame } from "./AuthFrame";
import { GradePicker } from "./GradePicker";
import type { Grade } from "./grades";
import { NICKNAME_MAX } from "./validate";

export type GoogleSignupValues = { readonly nickname: string; readonly grade: Grade };

export type GoogleSignupErrors = Partial<Record<"nickname" | "grade", string>>;

/** The same friendly checks as the register form, for the two fields this step has. */
export const validateGoogleSignup = (nickname: string, grade: Grade | null): GoogleSignupErrors => {
  const errors: { nickname?: string; grade?: string } = {};
  const length = nickname.trim().length;
  if (length === 0) errors.nickname = "Tulis nama panggilanmu, ya.";
  else if (length > NICKNAME_MAX)
    errors.nickname = `Nama panggilan paling panjang ${NICKNAME_MAX} huruf.`;
  if (grade === null) errors.grade = "Pilih kelasmu dulu.";
  return errors;
};

export type GoogleSignupScreenProps = {
  /** The Google profile name, as a starting point the student may change. */
  readonly suggestedNickname: string;
  readonly onSubmit: (values: GoogleSignupValues) => void;
  readonly submitting?: boolean;
  readonly serverErrors?: GoogleSignupErrors;
  readonly error?: string;
};

const errorProp = (message: string | undefined) =>
  message === undefined ? {} : { error: message };

/** The first Google sign-in: the account is verified, the student only picks a grade. */
export function GoogleSignupScreen({
  suggestedNickname,
  onSubmit,
  submitting = false,
  serverErrors = {},
  error,
}: GoogleSignupScreenProps) {
  const [nickname, setNickname] = useState(suggestedNickname);
  const [grade, setGrade] = useState<Grade | null>(null);
  const [errors, setErrors] = useState<GoogleSignupErrors>({});

  const submit = (e: FormEvent) => {
    e.preventDefault();
    const found = validateGoogleSignup(nickname, grade);
    setErrors(found);
    if (Object.keys(found).length === 0 && grade !== null) {
      onSubmit({ nickname: nickname.trim(), grade });
    }
  };
  const shown: GoogleSignupErrors = { ...serverErrors, ...errors };

  return (
    <AuthFrame
      title="Satu langkah lagi"
      greeting="Akun Google-mu sudah tersambung. Sekarang pilih kelasmu, yuk!"
    >
      <form onSubmit={submit} className="flex flex-col gap-4" noValidate>
        {error !== undefined && <FormAlert tone="error">{error}</FormAlert>}
        <TextField
          label="Nama panggilan"
          hint="Nama ini yang aku pakai untuk menyapamu."
          value={nickname}
          onValueChange={setNickname}
          autoComplete="nickname"
          maxLength={NICKNAME_MAX}
          {...errorProp(shown.nickname)}
        />
        <GradePicker value={grade} onChange={setGrade} {...errorProp(shown.grade)} />
        <Button
          type="submit"
          size="lg"
          block
          loading={submitting}
          loadingLabel="Menyiapkan akunmu…"
        >
          Mulai belajar
        </Button>
      </form>
    </AuthFrame>
  );
}
