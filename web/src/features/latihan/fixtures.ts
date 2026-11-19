// Demo content the API does not provide.
import { SUBJECTS } from "../../ui";
import type { Choice } from "./types";

export const LATIHAN_SUBJECTS: readonly Choice[] = SUBJECTS.map((s) => ({
  id: s.id,
  label: s.name,
}));

/** The set sizes the server makes. */
export const PRACTICE_COUNTS: readonly number[] = [5, 10, 15];

/** "Tahukah kamu?" while a set is being made, one per subject. */
export const FUN_FACTS: Readonly<Record<string, string>> = {
  ipa: "Kabel listrik dipasang agak kendor di siang hari, karena di malam hari kabel menyusut saat suhunya turun.",
  mtk: "Angka nol baru dipakai sebagai bilangan sekitar 1.500 tahun lalu. Tanpa nol, menulis angka 2026 jauh lebih repot!",
  bing: "Kata bahasa Inggris yang paling sering dipakai adalah “the”.",
  bind: "Bahasa Indonesia diikrarkan sebagai bahasa persatuan pada Sumpah Pemuda, 28 Oktober 1928.",
  ips: "Indonesia punya lebih dari 17.000 pulau, jadi negara kepulauan terbesar di dunia.",
  inf: "Komputer hanya mengenal dua angka, 0 dan 1. Semua gambar dan lagu di HP-mu tersimpan sebagai deretan 0 dan 1.",
  pp: "Lambang Garuda Pancasila punya 17 helai bulu di sayapnya, sesuai tanggal kemerdekaan Indonesia.",
  pai: "Kata “salat” berasal dari bahasa Arab yang berarti doa.",
};

export const funFactFor = (subjectId: string): string =>
  FUN_FACTS[subjectId] ?? FUN_FACTS.ipa ?? "";
