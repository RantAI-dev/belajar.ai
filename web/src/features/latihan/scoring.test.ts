import { describe, expect, test } from "bun:test";
import { LATIHAN_SAMPLE_ANSWERS, LATIHAN_SET } from "./fixtures";
import {
  answeredCount,
  isCorrect,
  pointsFor,
  resultHeadline,
  scoreSet,
  toggleChoice,
} from "./scoring";
import type { QuizQuestion } from "./types";

const byId = (id: string): QuizQuestion => {
  const q = LATIHAN_SET.questions.find((x) => x.id === id);
  if (q === undefined) return LATIHAN_SET.questions[0] as QuizQuestion;
  return q;
};

describe("answers", () => {
  test("pick-one replaces the choice", () => {
    expect(toggleChoice(byId("q1"), ["a"], "c")).toEqual(["c"]);
  });

  test("pick-many toggles each option", () => {
    const q = byId("q3");
    expect(toggleChoice(q, ["a"], "c")).toEqual(["a", "c"]);
    expect(toggleChoice(q, ["a", "c"], "a")).toEqual(["c"]);
  });

  test("an unknown option changes nothing", () => {
    expect(toggleChoice(byId("q1"), ["a"], "zzz")).toEqual(["a"]);
  });

  test("pick-many is right only with exactly the right set, in any order", () => {
    const q = byId("q3");
    expect(isCorrect(q, ["c", "a"])).toBe(true);
    expect(isCorrect(q, ["a"])).toBe(false);
    expect(isCorrect(q, ["a", "c", "d"])).toBe(false);
  });
});

describe("score", () => {
  test("the sample answers get 4 of 5, worth 20 stars", () => {
    const s = scoreSet(LATIHAN_SET, LATIHAN_SAMPLE_ANSWERS);
    expect(s).toEqual({ correct: 4, total: 5 });
    expect(pointsFor(s.correct)).toBe(20);
    expect(answeredCount(LATIHAN_SET, { q1: ["a"], q2: [] })).toBe(1);
  });

  test("headlines stay encouraging", () => {
    expect(resultHeadline(5, 5)).toBe("Hebat! Semua 5 benar");
    expect(resultHeadline(4, 5)).toBe("Hebat! 4 dari 5 benar");
    expect(resultHeadline(1, 5)).toBe("Ayo semangat! 1 dari 5 benar");
  });
});
