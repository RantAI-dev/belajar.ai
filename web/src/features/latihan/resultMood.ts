import { type MascotExpression, starsForScore } from "../../ui";

export type ResultMood = {
  readonly expression: MascotExpression;
  /** A line of encouragement under the headline; null when the score speaks for itself. */
  readonly cheer: string | null;
};

/**
 * The mascot on the results screen. Two stars or more (the same 60% line as starsForScore)
 * celebrates; below it the mascot stays cheerful and encourages. Never sad: these are children.
 */
export const resultMood = (correct: number, total: number): ResultMood =>
  starsForScore(correct, total) >= 2
    ? { expression: "celebration", cheer: null }
    : {
        expression: "cheerful",
        cheer:
          "Tidak apa-apa, salah itu bagian dari belajar. Lihat pembahasannya, lalu coba lagi, ya!",
      };
