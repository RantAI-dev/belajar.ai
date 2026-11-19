import type { FollowUp, TanyaMessage, TanyaView } from "./types";

const SOURCES = [
  {
    id: "src-1",
    label: "Buku IPA 7 · hal. 112",
    title: "IPA Kelas VII Semester 1 · Pemuaian · halaman 112",
    imageUrl: "/demo/buku-ipa-7-hal-112.svg",
    fullPageHref: "/demo/buku-ipa-7-hal-112.svg",
  },
  {
    id: "src-2",
    label: "Buku IPA 7 Revisi · hal. 98",
    title: "IPA untuk SMP/MTs Kelas VII · halaman 98",
    fullPageHref: "/demo/buku-ipa-7-hal-112.svg",
  },
] as const;

export const TANYA_FOLLOW_UPS: readonly FollowUp[] = [
  { id: "simpler", label: "Jelaskan lebih mudah" },
  { id: "example", label: "Kasih contoh lain" },
  { id: "quiz", label: "Buat 3 soal tentang ini" },
];

export const TANYA_SUGGESTIONS: readonly string[] = [
  "Kenapa rel kereta api diberi celah?",
  "Apa beda kalor dan suhu?",
  "Bagaimana cara mengubah °C ke °F?",
];

export const TANYA_CONVERSATION: readonly TanyaMessage[] = [
  { id: "q1", kind: "question", text: "Kenapa rel kereta api diberi celah?" },
  {
    id: "a1",
    kind: "answer",
    status: "done",
    parts: [
      { text: "Karena besi rel " },
      { text: "memuai", emphasis: true },
      {
        text: " saat panas. Kalau siang hari rel jadi panas, panjangnya bertambah sedikit. Celah itu memberi ruang supaya rel tidak saling dorong dan melengkung.",
      },
    ],
    example:
      "Contoh lain: tutup botol logam yang susah dibuka bisa dicelup air panas, lalu jadi lebih mudah dibuka.",
    sources: SOURCES,
  },
  { id: "q2", kind: "question", text: "Siapa juara Piala Dunia 2022?" },
  { id: "a2", kind: "not-in-book", gradeLabel: "Kelas 7" },
];

/** The canned reply the prototype streams for any new on-topic question. */
export const TANYA_DEMO_REPLY = {
  text: "Suhu adalah ukuran panas atau dinginnya suatu benda, sedangkan kalor adalah energi panas yang berpindah dari benda yang lebih panas ke benda yang lebih dingin. Jadi suhu bisa diukur dengan termometer, sedangkan kalor diukur dalam satuan joule.",
  example: "Contoh lain: es batu di dalam teh hangat mencair karena menerima kalor dari teh.",
  sources: [SOURCES[0]],
} as const;

/** Words that mark a question the prototype answers with "not in your book". */
export const TANYA_OFF_TOPIC = /piala|artis|game|youtube|sepak ?bola|film/i;

export const TANYA_EMPTY: TanyaView = {
  filter: "semua",
  filterSubjects: ["ipa", "mtk", "bing"],
  messages: [],
  followUps: TANYA_FOLLOW_UPS,
  suggestions: TANYA_SUGGESTIONS,
  busy: false,
};
