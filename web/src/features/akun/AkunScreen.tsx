import { type FormEvent, useState } from "react";
import type { FormMessage } from "../../contracts/messages";
import type { Result } from "../../contracts/result";
import { Avatar, Button, Card, FormAlert, StarCount, StreakBadge, TextField } from "../../ui";
import { type Grade, GradePicker, gradeLabel, PASSWORD_MIN } from "../auth";
import type { AkunView } from "./types";

/** What a save answers: done, or what to show (one sentence and/or hints by field name). */
export type AkunOutcome = Result<void, FormMessage>;

export type AkunScreenProps = {
  readonly view: AkunView;
  readonly onSaveProfile: (profile: { nickname: string; grade: Grade }) => Promise<AkunOutcome>;
  readonly onChangePassword: (change: { current: string; next: string }) => Promise<AkunOutcome>;
  readonly onLogout: () => Promise<AkunOutcome>;
};

export const PROFILE_SAVED = "Tersimpan! Profilmu sudah diperbarui.";
export const PASSWORD_SAVED = "Tersimpan! Kata sandimu sudah diganti. Ingat baik-baik, ya!";

const errorProp = (message: string | undefined) =>
  message === undefined ? {} : { error: message };

type Shown = { readonly notice?: string; readonly problem?: FormMessage };

const formError = (s: Shown): string | undefined => s.problem?.form ?? undefined;

export function AkunScreen({ view, onSaveProfile, onChangePassword, onLogout }: AkunScreenProps) {
  const [nickname, setNickname] = useState(view.nickname);
  const [grade, setGrade] = useState<Grade>(view.grade);
  const [current, setCurrent] = useState("");
  const [next, setNext] = useState("");
  const [busy, setBusy] = useState<"profile" | "password" | "logout" | null>(null);
  const [profile, setProfile] = useState<Shown>({});
  const [password, setPassword] = useState<Shown>({});
  const [logoutProblem, setLogoutProblem] = useState<string | undefined>(undefined);

  const saveProfile = async (e: FormEvent) => {
    e.preventDefault();
    if (busy !== null) return;
    const trimmed = nickname.trim();
    if (trimmed.length === 0) {
      setProfile({ problem: { form: null, fields: { nickname: "Tulis nama panggilanmu, ya." } } });
      return;
    }
    setBusy("profile");
    const r = await onSaveProfile({ nickname: trimmed, grade });
    setBusy(null);
    setProfile(r.ok ? { notice: PROFILE_SAVED } : { problem: r.error });
  };

  const changePassword = async (e: FormEvent) => {
    e.preventDefault();
    if (busy !== null) return;
    if (next.length < PASSWORD_MIN) {
      setPassword({
        problem: {
          form: null,
          fields: { newPassword: `Kata sandi baru minimal ${PASSWORD_MIN} karakter.` },
        },
      });
      return;
    }
    setBusy("password");
    const r = await onChangePassword({ current, next });
    setBusy(null);
    if (r.ok) {
      setCurrent("");
      setNext("");
      setPassword({ notice: PASSWORD_SAVED });
    } else setPassword({ problem: r.error });
  };

  const logout = async () => {
    setBusy("logout");
    const r = await onLogout();
    setBusy(null);
    setLogoutProblem(r.ok ? undefined : (r.error.form ?? undefined));
  };

  return (
    <div className="mx-auto flex w-full max-w-[760px] flex-col gap-5">
      <section aria-labelledby="akun-title" className="flex flex-wrap items-center gap-4">
        <Avatar name={view.nickname} size={72} />
        <div className="flex min-w-0 flex-1 flex-col gap-1">
          <h1 id="akun-title" className="m-0 font-display text-[28px] font-semibold leading-tight">
            Akunku
          </h1>
          <span className="text-[16px] font-bold text-ink-muted">
            @{view.username} · {gradeLabel(view.grade)}
          </span>
        </div>
        <div className="flex gap-2">
          <StreakBadge days={view.streakDays} />
          <StarCount count={view.stars} />
        </div>
      </section>

      <Card
        as="form"
        onSubmit={saveProfile}
        aria-labelledby="profil-title"
        className="flex flex-col gap-4"
      >
        <h2 id="profil-title" className="m-0 font-display text-[22px] font-semibold">
          Profilku
        </h2>
        {profile.notice !== undefined && <FormAlert tone="success">{profile.notice}</FormAlert>}
        {formError(profile) !== undefined && (
          <FormAlert tone="error">{formError(profile) ?? ""}</FormAlert>
        )}
        <TextField
          label="Nama panggilan"
          value={nickname}
          onValueChange={setNickname}
          autoComplete="nickname"
          maxLength={40}
          {...errorProp(profile.problem?.fields.nickname)}
        />
        <GradePicker
          value={grade}
          onChange={setGrade}
          {...errorProp(profile.problem?.fields.grade)}
        />
        <Button
          type="submit"
          className="self-start"
          loading={busy === "profile"}
          loadingLabel="Menyimpan…"
        >
          Simpan profil
        </Button>
      </Card>

      <Card
        as="form"
        onSubmit={changePassword}
        aria-labelledby="sandi-title"
        className="flex flex-col gap-4"
      >
        <h2 id="sandi-title" className="m-0 font-display text-[22px] font-semibold">
          Ganti kata sandi
        </h2>
        {password.notice !== undefined && <FormAlert tone="success">{password.notice}</FormAlert>}
        {formError(password) !== undefined && (
          <FormAlert tone="error">{formError(password) ?? ""}</FormAlert>
        )}
        <TextField
          label="Kata sandi sekarang"
          type="password"
          value={current}
          onValueChange={setCurrent}
          autoComplete="current-password"
          {...errorProp(password.problem?.fields.currentPassword)}
        />
        <TextField
          label="Kata sandi baru"
          type="password"
          value={next}
          onValueChange={setNext}
          autoComplete="new-password"
          hint={`Minimal ${PASSWORD_MIN} karakter.`}
          {...errorProp(password.problem?.fields.newPassword)}
        />
        <Button
          type="submit"
          variant="secondary"
          className="self-start"
          loading={busy === "password"}
          loadingLabel="Mengganti…"
        >
          Ganti kata sandi
        </Button>
      </Card>

      <Card className="flex flex-col gap-3">
        {logoutProblem !== undefined && <FormAlert tone="error">{logoutProblem}</FormAlert>}
        <div className="flex flex-wrap items-center justify-between gap-3">
          <span className="text-[17px] font-bold">Sudah selesai belajar hari ini?</span>
          <Button
            variant="ghost"
            onClick={() => void logout()}
            loading={busy === "logout"}
            loadingLabel="Keluar…"
          >
            Keluar
          </Button>
        </div>
      </Card>
    </div>
  );
}
