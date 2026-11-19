import { expect, test } from "bun:test";
import { LATIHAN_SET } from "../features/latihan";
import { demoSet } from "./demoQuiz";

test("a demo set has the asked number of questions, each with its own id", () => {
  const set = demoSet(LATIHAN_SET, 12, "s-12");
  expect(set.id).toBe("s-12");
  expect(set.questions).toHaveLength(12);
  expect(new Set(set.questions.map((q) => q.id)).size).toBe(12);
  expect(set.questions[5]?.prompt).toBe(LATIHAN_SET.questions[0]?.prompt);
});
