import { SUBJECTS } from "../../ui";
import type { FollowUp, TanyaView } from "./types";

/** What the quick follow-ups send; "quiz" opens Latihan instead. */
export const FOLLOW_UP_QUESTIONS: Readonly<Record<string, string>> = {
  simpler: "Jelaskan lebih mudah",
  example: "Kasih contoh lain",
};

export const TANYA_FOLLOW_UPS: readonly FollowUp[] = [
  { id: "simpler", label: "Jelaskan lebih mudah" },
  { id: "example", label: "Kasih contoh lain" },
  // Latihan makes sets of 5, 10 or 15, so the button does not promise three.
  { id: "quiz", label: "Buat soal tentang ini" },
];

export const TANYA_SUGGESTIONS: readonly string[] = [
  "Kenapa rel kereta api diberi celah?",
  "Apa beda kalor dan suhu?",
  "Bagaimana cara mengubah °C ke °F?",
];

export const TANYA_EMPTY: TanyaView = {
  filter: "semua",
  filterSubjects: SUBJECTS.map((s) => s.id),
  messages: [],
  followUps: TANYA_FOLLOW_UPS,
  suggestions: TANYA_SUGGESTIONS,
  busy: false,
};
