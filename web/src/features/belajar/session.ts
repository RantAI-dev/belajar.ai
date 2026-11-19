import { type Rating, starsForScore } from "../../ui";
import type { SessionContent, SessionQuestion } from "./types";

export type SessionStep = "explain" | "check" | "quiz" | "done";

export type SessionState = {
  readonly step: SessionStep;
  readonly quizIndex: number;
  /** questionId -> chosen optionId. */
  readonly answers: Readonly<Record<string, string>>;
};

export type SessionAction =
  | { readonly type: "understood" }
  | { readonly type: "answer"; readonly questionId: string; readonly optionId: string }
  | { readonly type: "next" };

export const SESSION_STEPS = ["Penjelasan", "Cek paham", "Kuis kecil"] as const;

export const startSession = (): SessionState => ({ step: "explain", quizIndex: 0, answers: {} });

/** The question the student is on, if the step has one. */
export const currentQuestion = (
  content: SessionContent,
  s: SessionState,
): SessionQuestion | undefined =>
  s.step === "check" ? content.check : s.step === "quiz" ? content.quiz[s.quizIndex] : undefined;

export const stepIndex = (s: SessionState): number =>
  s.step === "explain" ? 0 : s.step === "check" ? 1 : s.step === "quiz" ? 2 : 3;

/** Explain, then one check question, then the quiz one question at a time. An answer is final. */
export const sessionReducer =
  (content: SessionContent) =>
  (s: SessionState, action: SessionAction): SessionState => {
    switch (action.type) {
      case "understood":
        return s.step === "explain" ? { ...s, step: "check" } : s;
      case "answer": {
        const q = currentQuestion(content, s);
        if (q === undefined || q.id !== action.questionId || s.answers[q.id] !== undefined)
          return s;
        if (!q.options.some((o) => o.id === action.optionId)) return s;
        return { ...s, answers: { ...s.answers, [q.id]: action.optionId } };
      }
      case "next": {
        const q = currentQuestion(content, s);
        if (q === undefined || s.answers[q.id] === undefined) return s;
        if (s.step === "check")
          return content.quiz.length > 0
            ? { ...s, step: "quiz", quizIndex: 0 }
            : { ...s, step: "done" };
        const nextIndex = s.quizIndex + 1;
        return nextIndex < content.quiz.length
          ? { ...s, quizIndex: nextIndex }
          : { ...s, step: "done" };
      }
    }
  };

export const quizScore = (
  content: SessionContent,
  s: SessionState,
): { correct: number; total: number; stars: Rating } => {
  const correct = content.quiz.filter((q) => s.answers[q.id] === q.answerId).length;
  return {
    correct,
    total: content.quiz.length,
    stars: starsForScore(correct, content.quiz.length),
  };
};
