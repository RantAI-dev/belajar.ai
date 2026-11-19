import type { Choice, QuizSet, SetupView } from "./types";

const SRC = { label: "Lihat di buku IPA Kelas 7 · hal. 112", href: "/demo/buku-ipa-7-hal-112.svg" };

export const LATIHAN_SET: QuizSet = {
  id: "set-demo",
  subjectId: "ipa",
  chapterTitle: "Suhu, kalor, dan pemuaian",
  questions: [
    {
      id: "q1",
      kind: "single",
      prompt: "Pada termometer Celsius, titik didih air ditetapkan pada suhu …",
      options: [
        { id: "a", text: "0 °C" },
        { id: "b", text: "32 °C" },
        { id: "c", text: "100 °C" },
        { id: "d", text: "212 °C" },
      ],
      answerIds: ["c"],
      explanation: "Skala Celsius memakai titik beku air 0 °C dan titik didih air 100 °C.",
      source: {
        label: "Lihat di buku IPA Kelas 7 · hal. 104",
        href: "/demo/buku-ipa-7-hal-112.svg",
      },
    },
    {
      id: "q2",
      kind: "single",
      prompt: "Mengapa rel kereta api diberi celah di setiap sambungannya?",
      options: [
        { id: "a", text: "Supaya ringan" },
        { id: "b", text: "Memberi ruang saat rel memuai" },
        { id: "c", text: "Supaya mudah dipasang" },
        { id: "d", text: "Mengurangi bunyi" },
      ],
      answerIds: ["b"],
      explanation: "Saat panas, besi rel memuai. Celah mencegah rel melengkung.",
      source: SRC,
    },
    {
      id: "q3",
      kind: "multiple",
      prompt: "Mana saja yang termasuk contoh pemuaian dalam kehidupan sehari-hari?",
      options: [
        { id: "a", text: "Kabel listrik kendor di siang hari" },
        { id: "b", text: "Es batu mencair" },
        { id: "c", text: "Tutup botol logam lebih mudah dibuka setelah dicelup air panas" },
        { id: "d", text: "Air hujan jatuh ke tanah" },
      ],
      answerIds: ["a", "c"],
      explanation:
        "Kabel dan tutup logam bertambah panjang saat panas. Es mencair itu perubahan wujud, bukan pemuaian.",
      source: SRC,
    },
    {
      id: "q4",
      kind: "story",
      story:
        "Dina menuang teh panas ke dalam gelas kaca yang tebal dan dingin. Tiba-tiba gelasnya retak. Ibu bilang, lain kali Dina harus menaruh sendok logam di dalam gelas dulu.",
      prompt: "Mengapa gelas Dina retak?",
      options: [
        { id: "a", text: "Gelasnya terlalu berat" },
        { id: "b", text: "Bagian dalam gelas memuai lebih cepat daripada bagian luar" },
        { id: "c", text: "Tehnya terlalu manis" },
        { id: "d", text: "Gelasnya menyusut karena panas" },
      ],
      answerIds: ["b"],
      explanation:
        "Kaca tebal menghantarkan panas pelan. Bagian dalam memuai duluan, bagian luar belum, jadi gelas retak.",
      source: SRC,
    },
    {
      id: "q5",
      kind: "single",
      prompt: "Suhu 50 °C jika diubah ke Fahrenheit adalah …",
      options: [
        { id: "a", text: "82 °F" },
        { id: "b", text: "90 °F" },
        { id: "c", text: "122 °F" },
        { id: "d", text: "132 °F" },
      ],
      answerIds: ["c"],
      explanation:
        "Rumusnya °F = (9/5 × °C) + 32. Jadi 9/5 × 50 = 90, lalu ditambah 32 menjadi 122 °F. Kalau kamu dapat 90, tinggal tambah 32!",
      source: {
        label: "Lihat di buku IPA Kelas 7 · hal. 106",
        href: "/demo/buku-ipa-7-hal-112.svg",
      },
    },
  ],
};

/** The demo answers used when the results page is opened directly. */
export const LATIHAN_SAMPLE_ANSWERS = {
  q1: ["c"],
  q2: ["b"],
  q3: ["a", "c"],
  q4: ["b"],
  q5: ["b"],
} as const;

export const LATIHAN_SUBJECTS: readonly Choice[] = [
  { id: "ipa", label: "IPA" },
  { id: "mtk", label: "Matematika" },
  { id: "bing", label: "Bahasa Inggris" },
];

export const LATIHAN_CHAPTERS: Readonly<Record<string, readonly Choice[]>> = {
  ipa: [
    { id: "ipa-3", label: "Bab 3: Suhu, kalor, dan pemuaian" },
    { id: "ipa-2", label: "Bab 2: Zat dan perubahannya" },
    { id: "ipa-1", label: "Bab 1: Hakikat ilmu sains" },
  ],
  mtk: [
    { id: "mtk-2", label: "Bab 2: Bilangan rasional" },
    { id: "mtk-1", label: "Bab 1: Bilangan bulat" },
  ],
  bing: [{ id: "bing-1", label: "Chapter 1: About me" }],
};

export const LATIHAN_SETUP: SetupView = {
  subjects: LATIHAN_SUBJECTS,
  chapters: LATIHAN_CHAPTERS.ipa ?? [],
  subjectId: "ipa",
  chapterId: "ipa-3",
  kind: "single",
  count: 5,
  counts: [5, 10, 15],
  generation: null,
  funFact: {
    text: "Kabel listrik dipasang agak kendor di siang hari, karena di malam hari kabel menyusut saat suhunya turun.",
    source: "Dari buku IPA Kelas 7 · hal. 113",
  },
};
