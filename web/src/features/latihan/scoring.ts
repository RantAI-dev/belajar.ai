import type { Answers, QuizQuestion, QuizSet } from "./types";

/** Right only when exactly the correct options are chosen. */
export const isCorrect = (q: QuizQuestion, chosen: readonly string[]): boolean =>
  chosen.length === q.answerIds.length && q.answerIds.every((id) => chosen.includes(id));

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

export const scoreSet = (set: QuizSet, answers: Answers): { correct: number; total: number } => ({
  correct: set.questions.filter((q) => isCorrect(q, answers[q.id] ?? [])).length,
  total: set.questions.length,
});

export const STARS_PER_CORRECT = 5;
export const pointsFor = (correct: number): number => Math.max(0, correct) * STARS_PER_CORRECT;

export const answeredCount = (set: QuizSet, answers: Answers): number =>
  set.questions.filter((q) => (answers[q.id] ?? []).length > 0).length;

/** The headline on the results screen. */
export const resultHeadline = (correct: number, total: number): string => {
  if (total > 0 && correct === total) return `Hebat! Semua ${total} benar`;
  if (correct * 2 >= total) return `Hebat! ${correct} dari ${total} benar`;
  return `Ayo semangat! ${correct} dari ${total} benar`;
};
