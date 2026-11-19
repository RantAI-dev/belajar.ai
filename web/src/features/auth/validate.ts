import type { Grade } from "./grades";

export type RegisterValues = {
  readonly nickname: string;
  readonly username: string;
  readonly password: string;
  readonly grade: Grade | null;
};

export type RegisterErrors = Partial<Record<keyof RegisterValues, string>>;

export const USERNAME_PATTERN = /^[a-z0-9_.]{3,20}$/;
export const PASSWORD_MIN = 8;

/** Friendly checks before the form is sent; the server has the final word. */
export const validateRegister = (v: RegisterValues): RegisterErrors => {
  const errors: { -readonly [K in keyof RegisterErrors]: RegisterErrors[K] } = {};
  if (v.nickname.trim().length === 0) errors.nickname = "Tulis nama panggilanmu, ya.";
  else if (v.nickname.trim().length > 30)
    errors.nickname = "Nama panggilan paling panjang 30 huruf.";
  if (!USERNAME_PATTERN.test(v.username))
    errors.username = "Pakai 3–20 huruf kecil, angka, titik atau garis bawah.";
  if (v.password.length < PASSWORD_MIN)
    errors.password = `Kata sandi minimal ${PASSWORD_MIN} karakter.`;
  if (v.grade === null) errors.grade = "Pilih kelasmu dulu.";
  return errors;
};

export const hasErrors = (e: RegisterErrors): boolean => Object.keys(e).length > 0;
