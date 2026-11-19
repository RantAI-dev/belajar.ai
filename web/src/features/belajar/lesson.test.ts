import { describe, expect, test } from "bun:test";
import { explainPrompt, nextStep, QUIZ_PROMPT, quizHeadline, stepIndex } from "./lesson";
import type { LessonStep } from "./types";

describe("guided lesson", () => {
  test("the first message asks for the chapter at the student's grade", () => {
    expect(explainPrompt("Suhu, Kalor, dan Pemuaian", 7)).toBe(
      "Jelaskan bab Suhu, Kalor, dan Pemuaian untuk kelas 7 dengan sederhana.",
    );
  });

  test("steps move explain → check → quiz, and the quiz step asks for three questions", () => {
    expect(nextStep("explain")?.step).toBe("check");
    expect(nextStep("check")).toEqual({ step: "quiz", prompt: QUIZ_PROMPT });
    expect(nextStep("quiz")).toBeNull();
    expect(nextStep("done")).toBeNull();
    expect(QUIZ_PROMPT).toBe("Buat 3 soal pilihan ganda singkat tentang bab ini.");
  });

  test("stepIndex places each step on the stepper", () => {
    const steps: readonly LessonStep[] = ["explain", "check", "quiz", "done"];
    expect(steps.map(stepIndex)).toEqual([0, 1, 2, 3]);
  });

  test("quizHeadline", () => {
    expect(quizHeadline(3, 3)).toBe("Hebat! Semua benar!");
    expect(quizHeadline(1, 3)).toBe("Kerja bagus! 1 dari 3 benar");
  });
});
