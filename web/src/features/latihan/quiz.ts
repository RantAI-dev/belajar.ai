import { toggleChoice } from "./scoring";
import type { Answers, QuizSet } from "./types";

/** Answering a set: which question is open and what is chosen so far. */
export type QuizState = { readonly index: number; readonly answers: Answers };

export type QuizAction =
  | { readonly type: "choose"; readonly optionId: string }
  | { readonly type: "prev" }
  | { readonly type: "next" };

export const startQuiz = (): QuizState => ({ index: 0, answers: {} });

/** "Next" waits for a choice; the last question has no next. */
export const quizReducer =
  (set: QuizSet) =>
  (s: QuizState, a: QuizAction): QuizState => {
    const q = set.questions[s.index];
    if (q === undefined) return s;
    const chosen = s.answers[q.id] ?? [];
    switch (a.type) {
      case "choose":
        return { ...s, answers: { ...s.answers, [q.id]: toggleChoice(q, chosen, a.optionId) } };
      case "prev":
        return { ...s, index: Math.max(0, s.index - 1) };
      case "next":
        if (chosen.length === 0) return s;
        return { ...s, index: Math.min(set.questions.length - 1, s.index + 1) };
    }
  };

/** Every question has a choice, so the set can be sent for grading. */
export const canSubmit = (set: QuizSet, s: QuizState): boolean =>
  set.questions.length > 0 && set.questions.every((q) => (s.answers[q.id] ?? []).length > 0);
