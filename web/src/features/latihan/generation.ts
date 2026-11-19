import type { Generation } from "./types";

/** What the progress card says at each stage of generation. */
export const generationText = (
  g: Generation | null,
): { readonly headline: string; readonly detail: string } => {
  if (g === null)
    return {
      headline: "Aku siap membuatkan soal!",
      detail: "Pilih mapel, bab dan jenis soal, lalu tekan Buatkan soal.",
    };
  switch (g.stage) {
    case "retrieval":
      return { headline: "Aku cari bahannya di bukumu…", detail: "Sebentar, ya." };
    case "generating":
      return {
        headline: "Sebentar ya, soalmu sedang dibuat…",
        detail: `${g.ready} dari ${g.total} soal sudah jadi.`,
      };
    case "checking":
      return {
        headline: "Aku cek dulu setiap soal…",
        detail: "Supaya kunci jawabannya pasti benar.",
      };
    case "labelling":
      return { headline: "Hampir selesai!", detail: "Aku rapikan soal-soalnya." };
    case "ready":
      return { headline: "Soalmu sudah siap!", detail: "Ayo mulai kerjakan." };
  }
};

/** The numbered slots under the bar: done up to `ready`. Shown only for short sets. */
export const slotsOf = (
  g: Generation,
): readonly { readonly n: number; readonly done: boolean }[] =>
  g.total > 5 ? [] : Array.from({ length: g.total }, (_, i) => ({ n: i + 1, done: i < g.ready }));
