import type { ChapterNode, SessionContent, SubjectEntry } from "./types";

const map = (titles: readonly string[], stars: readonly number[]): readonly ChapterNode[] =>
  titles.map((title, i) => ({
    id: `${i + 1}`,
    number: i + 1,
    title,
    stars: stars[i] ?? 0,
    status: i < stars.length ? "done" : i === stars.length ? "current" : "todo",
  }));

/** Chapter maps per subject; chapter ids are prefixed with the subject id by `chaptersOf`. */
const CHAPTERS: Readonly<Record<string, readonly ChapterNode[]>> = {
  ipa: map(
    [
      "Hakikat ilmu sains",
      "Zat dan perubahannya",
      "Suhu, kalor, dan pemuaian",
      "Gerak dan gaya",
      "Klasifikasi makhluk hidup",
    ],
    [3, 2],
  ),
  mtk: map(
    ["Bilangan bulat", "Bilangan rasional", "Rasio", "Persamaan linear", "Data dan diagram"],
    [2],
  ),
  bing: map(["About me", "Culinary and me", "Home sweet home", "My school activities"], [1]),
  bind: map(["Deskripsi", "Teks prosedur", "Puisi rakyat", "Surat pribadi"], []),
  ips: map(["Kehidupan sosial", "Potensi ekonomi lingkungan", "Kegiatan ekonomi"], []),
  inf: map(["Berpikir komputasional", "Teknologi informasi", "Sistem komputer"], []),
  pp: map(["Sejarah kelahiran Pancasila", "Norma dan aturan", "Bhinneka tunggal ika"], []),
  pai: map(
    ["Al-Qur'an dan hadis", "Akhlak terpuji", "Salat berjamaah", "Sejarah Islam", "Puasa"],
    [1],
  ),
};

export const chaptersOf = (subjectId: string): readonly ChapterNode[] =>
  (CHAPTERS[subjectId] ?? []).map((c) => ({ ...c, id: `${subjectId}-${c.id}` }));

export const BELAJAR_SUBJECTS: readonly SubjectEntry[] = [
  { subjectId: "ipa", progressLabel: "Bab 3 dari 5", progress: 60, href: "/belajar/ipa" },
  { subjectId: "mtk", progressLabel: "Bab 2 dari 5", progress: 35, href: "/belajar/mtk" },
  { subjectId: "bing", progressLabel: "Bab 2 dari 4", progress: 20, href: "/belajar/bing" },
  { subjectId: "bind", progressLabel: "Belum mulai", progress: 0, href: "/belajar/bind" },
  { subjectId: "ips", progressLabel: "Belum mulai", progress: 0, href: "/belajar/ips" },
  { subjectId: "inf", progressLabel: "Belum mulai", progress: 0, href: "/belajar/inf" },
  { subjectId: "pp", progressLabel: "Belum mulai", progress: 0, href: "/belajar/pp" },
  { subjectId: "pai", progressLabel: "Bab 2 dari 5", progress: 15, href: "/belajar/pai" },
];

export const BELAJAR_SESSION: SessionContent = {
  subjectId: "ipa",
  chapterId: "ipa-3",
  chapterTitle: "Suhu, kalor, dan pemuaian",
  explanation: [
    "Suhu menunjukkan seberapa panas atau dingin sebuah benda. Kita mengukurnya dengan termometer, misalnya dalam derajat Celsius (°C).",
    "Kalau benda dipanaskan, partikelnya bergerak makin cepat dan saling menjauh. Akibatnya benda itu bertambah besar sedikit. Peristiwa ini disebut pemuaian.",
    "Itulah sebabnya rel kereta api diberi celah, dan kabel listrik dipasang agak kendor.",
  ],
  source: {
    label: "Buku IPA 7 · hal. 112",
    title: "IPA Kelas VII Semester 1 · Pemuaian · halaman 112",
    fullPageHref: "/demo/buku-ipa-7-hal-112.svg",
  },
  check: {
    id: "check-1",
    prompt: "Coba jawab dulu, ya: apa yang terjadi pada benda saat dipanaskan?",
    options: [
      { id: "a", text: "Bendanya menyusut" },
      { id: "b", text: "Bendanya memuai" },
      { id: "c", text: "Bendanya tidak berubah" },
    ],
    answerId: "b",
    explanation: "Saat dipanaskan, partikel benda saling menjauh, jadi benda memuai.",
  },
  quiz: [
    {
      id: "quiz-1",
      prompt: "Alat untuk mengukur suhu disebut …",
      options: [
        { id: "a", text: "Termometer" },
        { id: "b", text: "Barometer" },
        { id: "c", text: "Penggaris" },
      ],
      answerId: "a",
      explanation: "Termometer dipakai untuk mengukur suhu.",
    },
    {
      id: "quiz-2",
      prompt: "Mengapa kabel listrik dipasang agak kendor di siang hari?",
      options: [
        { id: "a", text: "Supaya terlihat rapi" },
        { id: "b", text: "Karena di malam hari kabel menyusut" },
        { id: "c", text: "Supaya listriknya lebih kuat" },
      ],
      answerId: "b",
      explanation:
        "Malam hari suhunya turun, kabel menyusut. Kalau dipasang kencang, kabel bisa putus.",
    },
    {
      id: "quiz-3",
      prompt: "Zat yang paling besar memuai saat dipanaskan adalah …",
      options: [
        { id: "a", text: "Padat" },
        { id: "b", text: "Cair" },
        { id: "c", text: "Gas" },
      ],
      answerId: "c",
      explanation: "Partikel gas paling renggang, jadi pemuaiannya paling besar.",
    },
  ],
};
