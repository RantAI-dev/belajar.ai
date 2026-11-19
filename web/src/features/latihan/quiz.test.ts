import { describe, expect, test } from "bun:test";
import { canSubmit, type QuizAction, quizReducer, startQuiz } from "./quiz";
import type { QuizSet } from "./types";

const SET: QuizSet = {
  id: "s1",
  subjectId: "ipa",
  topic: "Suhu",
  submitted: false,
  questions: [
    {
      id: "q1",
      kind: "single",
      prompt: "1",
      options: [
        { id: "A", text: "a" },
        { id: "B", text: "b" },
      ],
    },
    {
      id: "q2",
      kind: "multiple",
      prompt: "2",
      options: [
        { id: "A", text: "a" },
        { id: "B", text: "b" },
      ],
    },
  ],
};

const run = (...actions: QuizAction[]) => actions.reduce(quizReducer(SET), startQuiz());

describe("quizReducer", () => {
  test("choosing answers the open question", () => {
    expect(run({ type: "choose", optionId: "B" }).answers).toEqual({ q1: ["B"] });
  });

  test("next waits for a choice, then moves on and stops at the last question", () => {
    expect(run({ type: "next" }).index).toBe(0);
    const s = run({ type: "choose", optionId: "A" }, { type: "next" });
    expect(s.index).toBe(1);
    const end = run(
      { type: "choose", optionId: "A" },
      { type: "next" },
      { type: "choose", optionId: "A" },
      { type: "next" },
    );
    expect(end.index).toBe(1);
  });

  test("a pick-many question toggles; prev goes back and keeps the answers", () => {
    const s = run(
      { type: "choose", optionId: "A" },
      { type: "next" },
      { type: "choose", optionId: "A" },
      { type: "choose", optionId: "B" },
      { type: "prev" },
    );
    expect(s).toEqual({ index: 0, answers: { q1: ["A"], q2: ["A", "B"] } });
    expect(run({ type: "prev" }).index).toBe(0);
  });

  test("canSubmit once every question has a choice", () => {
    expect(canSubmit(SET, run({ type: "choose", optionId: "A" }))).toBe(false);
    expect(
      canSubmit(
        SET,
        run({ type: "choose", optionId: "A" }, { type: "next" }, { type: "choose", optionId: "B" }),
      ),
    ).toBe(true);
  });
});
