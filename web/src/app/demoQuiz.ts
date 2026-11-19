import type { QuizSet } from "../features/latihan";

/** Stretches the demo set to `count` questions by repeating it with fresh ids. */
export const demoSet = (base: QuizSet, count: number, id: string): QuizSet => ({
  ...base,
  id,
  questions: Array.from({ length: count }, (_, i) => {
    const q = base.questions[i % base.questions.length];
    return q === undefined ? undefined : { ...q, id: `${q.id}-${i + 1}` };
  }).filter((q) => q !== undefined),
});
