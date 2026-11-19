import { SUBJECTS } from "../../ui";
import type { TanyaMessage } from "./types";

/** "Bab 5: Suhu, Kalor, dan Pemuaian" → "Suhu, Kalor, dan Pemuaian". */
export const topicOfSection = (section: string): string =>
  section.replace(/^\s*(bab|chapter|unit)\s+\d+\s*[:.\-–]\s*/i, "").trim();

// Longest names first, so "Bahasa Indonesia" is not read as some shorter name.
const BY_NAME = [...SUBJECTS].sort((a, b) => b.name.length - a.name.length);

/** Our subject id for a book title such as "IPA Buku Siswa Kelas VII". */
export const subjectOfBook = (book: string): string | null => {
  const lower = book.toLowerCase();
  return BY_NAME.find((s) => lower.startsWith(s.name.toLowerCase()))?.id ?? null;
};

const TOPIC_MAX = 100;

/**
 * Where "Buat 3 soal tentang ini" goes: the latest answer's chapter and book subject, else the
 * conversation's subject and the last question as the topic.
 */
export const practiceTopicOf = (
  messages: readonly TanyaMessage[],
  conversationSubjectId: string | null,
): { readonly subjectId: string | null; readonly topic: string } => {
  let topic = "";
  let subjectId = conversationSubjectId;
  for (let i = messages.length - 1; i >= 0 && topic === ""; i--) {
    const m = messages[i];
    if (m?.kind === "answer") {
      const source = m.sources[0];
      if (source?.section !== undefined) topic = topicOfSection(source.section);
      if (source !== undefined) subjectId = subjectId ?? subjectOfBook(source.book);
    } else if (m?.kind === "question") {
      topic = m.text.replace(/\?+\s*$/, "").trim();
    }
  }
  return { subjectId, topic: topic.slice(0, TOPIC_MAX) };
};
