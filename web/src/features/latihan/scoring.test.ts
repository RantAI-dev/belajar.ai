import { describe, expect, test } from "bun:test";
import { answeredCount, resultHeadline, toggleChoice } from "./scoring";
import type { QuizQuestion, QuizSet } from "./types";

const q = (kind: QuizQuestion["kind"]): QuizQuestion => ({
  id: "q1",
  kind,
  prompt: "?",
  options: [
    { id: "A", text: "a" },
    { id: "B", text: "b" },
  ],
});

describe("toggleChoice", () => {
  test("pick-one questions replace the choice", () => {
    expect(toggleChoice(q("single"), ["A"], "B")).toEqual(["B"]);
    expect(toggleChoice(q("story"), [], "A")).toEqual(["A"]);
  });

  test("pick-many questions toggle", () => {
    expect(toggleChoice(q("multiple"), ["A"], "B")).toEqual(["A", "B"]);
    expect(toggleChoice(q("multiple"), ["A", "B"], "A")).toEqual(["B"]);
  });

  test("an unknown option changes nothing", () => {
    expect(toggleChoice(q("single"), ["A"], "Z")).toEqual(["A"]);
  });
});

test("answeredCount counts questions with a choice", () => {
  const set: QuizSet = {
    id: "s",
    subjectId: "ipa",
    topic: "t",
    submitted: false,
    questions: [q("single"), { ...q("single"), id: "q2" }],
  };
  expect(answeredCount(set, { q1: ["A"], q2: [] })).toBe(1);
});

test("resultHeadline cheers every score", () => {
  expect(resultHeadline(5, 5)).toBe("Hebat! Semua 5 benar");
  expect(resultHeadline(3, 5)).toBe("Hebat! 3 dari 5 benar");
  expect(resultHeadline(1, 5)).toBe("Hampir benar! 1 dari 5 benar");
});
