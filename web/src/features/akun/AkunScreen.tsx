import { type FormEvent, useState } from "react";
import { Avatar, Button, Card, StarCount, StreakBadge, TextField } from "../../ui";
import { type Grade, GradePicker, gradeLabel, PASSWORD_MIN } from "../auth";
import type { AkunView } from "./types";

export type AkunScreenProps = {
  readonly view: AkunView;
  readonly onSaveProfile: (profile: { nickname: string; grade: Grade }) => void;
  readonly onChangePassword: (change: { current: string; next: string }) => void;
  readonly onLogout: () => void;
  /** A short confirmation after a save, e.g. "Tersimpan!". */
  readonly notice?: string;
};

export function AkunScreen({
  view,
  onSaveProfile,
  onChangePassword,
  onLogout,
  notice,
}: AkunScreenProps) {
  const [nickname, setNickname] = useState(view.nickname);
  const [grade, setGrade] = useState<Grade>(view.grade);
  const [current, setCurrent] = useState("");
  const [next, setNext] = useState("");
  const [pwError, setPwError] = useState<string | undefined>(undefined);

  const saveProfile = (e: FormEvent) => {
    e.preventDefault();
    if (nickname.trim().length > 0) onSaveProfile({ nickname: nickname.trim(), grade });
  };
  const changePassword = (e: FormEvent) => {
    e.preventDefault();
    if (next.length < PASSWORD_MIN) {
      setPwError(`Kata sandi baru minimal ${PASSWORD_MIN} karakter.`);
      return;
    }
    setPwError(undefined);
    onChangePassword({ current, next });
    setCurrent("");
    setNext("");
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

      {notice !== undefined && (
        <p
          role="status"
          className="m-0 rounded-[14px] bg-success-soft px-4 py-3 text-[16px] font-extrabold text-success-edge"
        >
          {notice}
        </p>
      )}

      <Card
        as="form"
        onSubmit={saveProfile}
        aria-labelledby="profil-title"
        className="flex flex-col gap-4"
      >
        <h2 id="profil-title" className="m-0 font-display text-[22px] font-semibold">
          Profilku
        </h2>
        <TextField
          label="Nama panggilan"
          value={nickname}
          onValueChange={setNickname}
          autoComplete="nickname"
          maxLength={30}
        />
        <GradePicker value={grade} onChange={setGrade} />
        <Button type="submit" className="self-start">
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
        <TextField
          label="Kata sandi sekarang"
          type="password"
          value={current}
          onValueChange={setCurrent}
          autoComplete="current-password"
        />
        <TextField
          label="Kata sandi baru"
          type="password"
          value={next}
          onValueChange={setNext}
          autoComplete="new-password"
          hint={`Minimal ${PASSWORD_MIN} karakter.`}
          {...(pwError !== undefined ? { error: pwError } : {})}
        />
        <Button type="submit" variant="secondary" className="self-start">
          Ganti kata sandi
        </Button>
      </Card>

      <Card className="flex flex-wrap items-center justify-between gap-3">
        <span className="text-[17px] font-bold">Sudah selesai belajar hari ini?</span>
        <Button variant="ghost" onClick={onLogout}>
          Keluar
        </Button>
      </Card>
    </div>
  );
}
