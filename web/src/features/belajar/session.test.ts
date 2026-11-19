import { describe, expect, test } from "bun:test";
import { BELAJAR_SESSION as content } from "./fixtures";
import {
  currentQuestion,
  quizScore,
  type SessionAction,
  sessionReducer,
  startSession,
  stepIndex,
} from "./session";

const reduce = sessionReducer(content);
const run = (...actions: SessionAction[]) => actions.reduce(reduce, startSession());
const answer = (questionId: string, optionId: string): SessionAction => ({
  type: "answer",
  questionId,
  optionId,
});
const next: SessionAction = { type: "next" };

describe("guided session", () => {
  test("explain, then check, then the quiz one question at a time, then done", () => {
    let s = run({ type: "understood" });
    expect(s.step).toBe("check");
    expect(stepIndex(s)).toBe(1);
    s = [answer("check-1", "b"), next].reduce(reduce, s);
    expect(s.step).toBe("quiz");
    expect(currentQuestion(content, s)?.id).toBe("quiz-1");
    s = [
      answer("quiz-1", "a"),
      next,
      answer("quiz-2", "b"),
      next,
      answer("quiz-3", "a"),
      next,
    ].reduce(reduce, s);
    expect(s.step).toBe("done");
    expect(stepIndex(s)).toBe(3);
    expect(quizScore(content, s)).toEqual({ correct: 2, total: 3, stars: 2 });
  });

  test("cannot move on before answering", () => {
    const s = run({ type: "understood" }, next);
    expect(s.step).toBe("check");
  });

  test("an answer is final and must belong to the current question", () => {
    let s = run({ type: "understood" }, answer("check-1", "a"), answer("check-1", "b"));
    expect(s.answers["check-1"]).toBe("a");
    s = reduce(s, answer("quiz-1", "a"));
    expect(s.answers["quiz-1"]).toBeUndefined();
    s = run({ type: "understood" }, answer("check-1", "zzz"));
    expect(s.answers["check-1"]).toBeUndefined();
  });

  test("all quiz answers right earns three stars", () => {
    const s = run(
      { type: "understood" },
      answer("check-1", "b"),
      next,
      answer("quiz-1", "a"),
      next,
      answer("quiz-2", "b"),
      next,
      answer("quiz-3", "c"),
      next,
    );
    expect(quizScore(content, s).stars).toBe(3);
  });
});
