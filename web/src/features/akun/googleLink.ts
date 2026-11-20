// "Hubungkan dengan Google" in Akun: where the button goes, and what Akun says when the server's
// callback sends the student back with `?google=<outcome>`.

/** A full-page navigation (not a fetch): the server answers with a redirect to Google. */
export const GOOGLE_LINK_HREF = "/v1/auth/google/link/start";

export type LinkOutcome = "linked" | "conflict" | "cancelled" | "unavailable" | "failed";

export type LinkNotice = { readonly tone: "success" | "error"; readonly text: string };

const NOTICES: Readonly<Record<LinkOutcome, LinkNotice>> = {
  linked: {
    tone: "success",
    text: "Berhasil! Akunmu sudah terhubung dengan Google. Sekarang kamu bisa masuk dengan Google.",
  },
  conflict: {
    tone: "error",
    text: "Akun Google ini sudah dipakai oleh akun belajar.ai lain, jadi tidak bisa dihubungkan.",
  },
  cancelled: {
    tone: "error",
    text: "Menghubungkan dengan Google dibatalkan. Kamu bisa coba lagi kapan saja.",
  },
  unavailable: {
    tone: "error",
    text: "Google sedang tidak bisa dihubungi. Coba lagi sebentar lagi.",
  },
  failed: {
    tone: "error",
    text: "Menghubungkan dengan Google belum berhasil. Coba lagi dari awal, ya.",
  },
};

const isOutcome = (value: string): value is LinkOutcome => Object.hasOwn(NOTICES, value);

/** The notice for `?google=<value>` on Akun, or undefined when there is none (or it is unknown). */
export const linkNotice = (search: string): LinkNotice | undefined => {
  const value = new URLSearchParams(search).get("google");
  return value !== null && isOutcome(value) ? NOTICES[value] : undefined;
};
