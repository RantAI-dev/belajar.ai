import { describe, expect, test } from "bun:test";
import {
  chapterContext,
  explainPrompt,
  nextStep,
  QUIZ_PROMPT,
  quizHeadline,
  stepIndex,
} from "./lesson";
import type { ChapterTopic, LessonStep } from "./types";

const topics: readonly ChapterTopic[] = [
  { title: "Sel sebagai Unit Struktural dan Fungsional Kehidupan", page: 9 },
  { title: "Jaringan-Jaringan pada Hewan dan Tumbuhan", page: 17 },
];

describe("guided lesson", () => {
  test("the first message asks for the chapter at the student's grade", () => {
    expect(explainPrompt("Suhu, Kalor, dan Pemuaian", [], 7)).toBe(
      'Bab: Suhu, Kalor, dan Pemuaian\nJelaskan bab "Suhu, Kalor, dan Pemuaian" untuk kelas 7 dengan sederhana. Gunakan hanya materi pada daftar ini dan jangan keluar dari bab.',
    );
  });

  test("the chapter title and its materi go to the upstream as context", () => {
    const prompt = explainPrompt("Sistem Organisasi Kehidupan Makhluk Hidup", topics, 7);
    expect(prompt).toContain("Bab: Sistem Organisasi Kehidupan Makhluk Hidup");
    expect(prompt).toContain("Materi dalam bab ini:");
    expect(prompt).toContain("- Sel sebagai Unit Struktural dan Fungsional Kehidupan");
    expect(prompt).toContain("- Jaringan-Jaringan pada Hewan dan Tumbuhan");
    expect(chapterContext("Bab tanpa materi", [])).toBe("Bab: Bab tanpa materi\n");
  });

  test("steps move explain → check → quiz, keeping the chapter context", () => {
    expect(nextStep("explain", "Pecahan", topics)?.step).toBe("check");
    expect(nextStep("check", "Pecahan", topics)).toEqual({
      step: "quiz",
      prompt: `${chapterContext("Pecahan", topics)}${QUIZ_PROMPT}`,
    });
    expect(nextStep("check", "Pecahan", topics)?.prompt).toContain("- Sel sebagai Unit");
    expect(nextStep("quiz", "Pecahan", topics)).toBeNull();
    expect(nextStep("done", "Pecahan", topics)).toBeNull();
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
