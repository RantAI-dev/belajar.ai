import type { ChapterTopic, LessonStep } from "./types";

export const LESSON_STEPS = ["Penjelasan", "Cek paham", "Kuis kecil"] as const;

/** Questions in the quiz step; the student reports how many they got right. */
export const QUIZ_TOTAL = 3;

/**
 * The chapter's own table of contents, sent with every lesson question so the upstream stays
 * inside this chapter instead of drifting to another part of the book.
 */
export const chapterContext = (chapterTitle: string, topics: readonly ChapterTopic[]): string => {
  const lines = topics.map((topic) => `- ${topic.title}`).join("\n");
  return lines.length > 0
    ? `Bab: ${chapterTitle}\nMateri dalam bab ini:\n${lines}\n`
    : `Bab: ${chapterTitle}\n`;
};

/** What the session sends on its first open, so the explanation starts by itself. */
export const explainPrompt = (
  chapterTitle: string,
  topics: readonly ChapterTopic[],
  grade: number,
): string =>
  `${chapterContext(chapterTitle, topics)}Jelaskan bab "${chapterTitle}" untuk kelas ${grade} dengan sederhana. Gunakan hanya materi pada daftar ini dan jangan keluar dari bab.`;

export const CHECK_PROMPT = "Beri aku 1 pertanyaan singkat untuk cek pemahamanku tentang bab ini.";
export const QUIZ_PROMPT = "Buat 3 soal pilihan ganda singkat tentang bab ini.";

/** The step the "next" button moves to, with the question it asks there. */
export const nextStep = (
  step: LessonStep,
  chapterTitle: string,
  topics: readonly ChapterTopic[],
): { readonly step: "check" | "quiz"; readonly prompt: string } | null =>
  step === "explain"
    ? { step: "check", prompt: `${chapterContext(chapterTitle, topics)}${CHECK_PROMPT}` }
    : step === "check"
      ? { step: "quiz", prompt: `${chapterContext(chapterTitle, topics)}${QUIZ_PROMPT}` }
      : null;

/** 0-based position on the stepper; `done` is past the last step. */
export const stepIndex = (step: LessonStep): number =>
  step === "explain" ? 0 : step === "check" ? 1 : step === "quiz" ? 2 : 3;

/** "Hebat! Semua benar!" or "Kerja bagus! 2 dari 3 benar". */
export const quizHeadline = (correct: number, total: number): string =>
  total > 0 && correct >= total
    ? "Hebat! Semua benar!"
    : `Kerja bagus! ${correct} dari ${total} benar`;
