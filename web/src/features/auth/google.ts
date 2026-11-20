// "Masuk dengan Google": where the button goes, and what the login page says when the server's
// callback sends the student back with `?google=<outcome>`.

/** A full-page navigation (not a fetch): the server answers with a redirect to Google. */
export const GOOGLE_START_HREF = "/v1/auth/google/start";

/** The query parameter the server's callback sets on `/masuk`. */
export const GOOGLE_PARAM = "google";

export type GoogleOutcome = "cancelled" | "conflict" | "unavailable" | "failed";

const NOTICES: Readonly<Record<GoogleOutcome, string>> = {
  cancelled: "Masuk dengan Google dibatalkan. Kamu bisa coba lagi kapan saja.",
  conflict: "Akun Google ini sudah tersambung ke akun lain. Coba masuk dengan Google sekali lagi.",
  unavailable: "Google sedang tidak bisa dihubungi. Coba lagi sebentar lagi.",
  failed: "Masuk dengan Google belum berhasil. Coba lagi dari awal, ya.",
};

const isOutcome = (value: string): value is GoogleOutcome => Object.hasOwn(NOTICES, value);

/** The sentence for `?google=<value>`, or undefined when there is none (or it is unknown). */
export const googleNotice = (search: string): string | undefined => {
  const value = new URLSearchParams(search).get(GOOGLE_PARAM);
  return value !== null && isOutcome(value) ? NOTICES[value] : undefined;
};

/** `/masuk?google=<outcome>`: where the onboarding step sends the student when it cannot go on. */
export const loginWithNotice = (outcome: GoogleOutcome): string =>
  `/masuk?${GOOGLE_PARAM}=${outcome}`;
