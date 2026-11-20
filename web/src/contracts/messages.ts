// The Indonesian sentence for each problem, chosen by `code`. The server's English `message` is
// for logs and developers; a student never sees it.
import type { ApiProblem } from "./problem";

export const MESSAGES = {
  wrongCredentials: "Nama pengguna atau kata sandi salah.",
  usernameTaken: "Nama pengguna ini sudah dipakai. Coba nama lain.",
  wrongCurrentPassword: "Kata sandi lama salah.",
  /** Akun shows it instead of the password form, and for `password_not_set` if that ever comes. */
  noPasswordWithGoogle: "Kamu masuk dengan Google, jadi tidak perlu kata sandi.",
  network: "Tidak bisa terhubung. Periksa internetmu, lalu coba lagi.",
  generic: "Ada yang tidak beres di pihak kami. Coba lagi sebentar lagi.",
  checkFields: "Ada isian yang perlu diperbaiki.",
} as const;

/** "Terlalu banyak percobaan. Coba lagi dalam 2 menit." Unknown wait reads as one minute. */
export const rateLimitMessage = (retryAfterSeconds: number | null): string => {
  const minutes = Math.max(1, Math.ceil((retryAfterSeconds ?? 60) / 60));
  return `Terlalu banyak percobaan. Coba lagi dalam ${minutes} menit.`;
};

/** Field names as the server names them (the JSON Pointer without its slash). */
export type FieldName =
  | "username"
  | "password"
  | "nickname"
  | "grade"
  | "currentPassword"
  | "newPassword";

// Lengths mirror rust/crates/server/src/routes/auth.rs and me.rs.
const FIELD_HINTS: Readonly<Record<FieldName, Readonly<Record<string, string>>>> = {
  username: {
    required: "Tulis nama penggunamu.",
    too_short: "Nama pengguna minimal 3 karakter.",
    too_long: "Nama pengguna paling panjang 32 karakter.",
    invalid_format: "Pakai huruf, angka, titik atau garis bawah saja.",
  },
  password: {
    required: "Tulis kata sandimu.",
    too_short: "Kata sandi minimal 8 karakter.",
    too_long: "Kata sandi paling panjang 128 karakter.",
  },
  newPassword: {
    required: "Tulis kata sandi barumu.",
    too_short: "Kata sandi baru minimal 8 karakter.",
    too_long: "Kata sandi baru paling panjang 128 karakter.",
  },
  currentPassword: { required: "Tulis kata sandimu yang sekarang." },
  nickname: {
    required: "Tulis nama panggilanmu, ya.",
    too_short: "Tulis nama panggilanmu, ya.",
    too_long: "Nama panggilan paling panjang 40 huruf.",
  },
  grade: { required: "Pilih kelasmu dulu.", out_of_range: "Pilih kelas 1 sampai 12." },
};

const CODE_HINTS: Readonly<Record<string, string>> = {
  required: "Bagian ini wajib diisi.",
  too_short: "Isiannya terlalu pendek.",
  too_long: "Isiannya terlalu panjang.",
  invalid_format: "Bentuk isiannya belum benar.",
  out_of_range: "Nilainya di luar batas.",
};

const isFieldName = (name: string): name is FieldName => Object.hasOwn(FIELD_HINTS, name);

/** A short hint for one bad field. */
export const fieldHint = (field: string, code: string): string => {
  const name = field.replace(/^\//, "");
  const specific = isFieldName(name) ? FIELD_HINTS[name][code] : undefined;
  return specific ?? CODE_HINTS[code] ?? CODE_HINTS.invalid_format ?? MESSAGES.generic;
};

/** Every bad field's hint, keyed by field name (`username`, not `/username`). */
export const fieldHints = (problem: ApiProblem): Partial<Record<string, string>> => {
  const out: Record<string, string> = {};
  for (const [field, code] of Object.entries(problem.fieldErrors)) {
    out[field.replace(/^\//, "")] = fieldHint(field, code);
  }
  return out;
};

/** The sentence for codes every screen treats the same. */
export const generalMessage = (problem: ApiProblem): string => {
  if (problem.code === "network") return MESSAGES.network;
  if (problem.code === "rate_limited") return rateLimitMessage(problem.retryAfterSeconds);
  if (problem.code === "invalid_request") return MESSAGES.checkFields;
  return MESSAGES.generic;
};

/** What a screen shows: one sentence for the form and/or hints next to fields. */
export type FormMessage = {
  readonly form: string | null;
  readonly fields: Partial<Record<string, string>>;
};

const formOnly = (form: string): FormMessage => ({ form, fields: {} });

/** A form's message: invalid fields go next to their inputs; everything else is one sentence. */
const formMessage = (
  problem: ApiProblem,
  special: Partial<Record<string, string>>,
): FormMessage => {
  const s = special[problem.code];
  if (s !== undefined) return formOnly(s);
  if (problem.code === "invalid_request" && Object.keys(problem.fieldErrors).length > 0) {
    return { form: null, fields: fieldHints(problem) };
  }
  return formOnly(generalMessage(problem));
};

export const loginMessage = (problem: ApiProblem): FormMessage =>
  formMessage(problem, { unauthenticated: MESSAGES.wrongCredentials });

export const registerMessage = (problem: ApiProblem): FormMessage =>
  problem.code === "conflict"
    ? { form: null, fields: { username: MESSAGES.usernameTaken } }
    : formMessage(problem, {});

export const profileMessage = (problem: ApiProblem): FormMessage => formMessage(problem, {});

export const passwordChangeMessage = (problem: ApiProblem): FormMessage =>
  formMessage(problem, {
    permission_denied: MESSAGES.wrongCurrentPassword,
    password_not_set: MESSAGES.noPasswordWithGoogle,
  });
