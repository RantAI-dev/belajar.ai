import type { Grade } from "./grades";

export type RegisterValues = {
  readonly nickname: string;
  readonly username: string;
  readonly password: string;
  readonly grade: Grade | null;
};

export type RegisterErrors = Partial<Record<keyof RegisterValues, string>>;

// Same rule as the server: 3–32 of A-Z a-z 0-9 _ . (the server stores it lowercased).
export const USERNAME_PATTERN = /^[A-Za-z0-9_.]{3,32}$/;
export const NICKNAME_MAX = 40;
export const PASSWORD_MIN = 8;

/** Friendly checks before the form is sent; the server has the final word. */
export const validateRegister = (v: RegisterValues): RegisterErrors => {
  const errors: { -readonly [K in keyof RegisterErrors]: RegisterErrors[K] } = {};
  if (v.nickname.trim().length === 0) errors.nickname = "Tulis nama panggilanmu, ya.";
  else if (v.nickname.trim().length > NICKNAME_MAX)
    errors.nickname = `Nama panggilan paling panjang ${NICKNAME_MAX} huruf.`;
  if (!USERNAME_PATTERN.test(v.username))
    errors.username = "Pakai 3–32 huruf, angka, titik atau garis bawah, tanpa spasi.";
  if (v.password.length < PASSWORD_MIN)
    errors.password = `Kata sandi minimal ${PASSWORD_MIN} karakter.`;
  if (v.grade === null) errors.grade = "Pilih kelasmu dulu.";
  return errors;
};

export const hasErrors = (e: RegisterErrors): boolean => Object.keys(e).length > 0;
