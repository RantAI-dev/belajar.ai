import type { Answers, QuizQuestion, QuizSet } from "./types";

/** Choosing in a "pick one" question replaces; in "pick more than one" it toggles. */
export const toggleChoice = (
  q: QuizQuestion,
  chosen: readonly string[],
  optionId: string,
): readonly string[] => {
  if (!q.options.some((o) => o.id === optionId)) return chosen;
  if (q.kind !== "multiple") return [optionId];
  return chosen.includes(optionId) ? chosen.filter((id) => id !== optionId) : [...chosen, optionId];
};

export const answeredCount = (set: QuizSet, answers: Answers): number =>
  set.questions.filter((q) => (answers[q.id] ?? []).length > 0).length;

/** The headline on the results screen. */
export const resultHeadline = (correct: number, total: number): string => {
  if (total > 0 && correct === total) return `Hebat! Semua ${total} benar`;
  if (correct * 2 >= total) return `Hampir benar! ${correct} dari ${total} benar`;
  return `Ayo coba lagi! ${correct} dari ${total} benar`;
};
