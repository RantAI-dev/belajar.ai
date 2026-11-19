import { expect, test } from "bun:test";
import { choiceState, letterAt } from "./ChoiceButton";

test("before the reveal only the chosen option shows", () => {
  expect(choiceState("a", ["a"], null)).toBe("selected");
  expect(choiceState("b", ["a"], null)).toBe("idle");
});

test("after the reveal: correct, wrong, missed, idle", () => {
  expect(choiceState("a", ["a"], ["a", "c"])).toBe("correct");
  expect(choiceState("b", ["a", "b"], ["a", "c"])).toBe("wrong");
  expect(choiceState("c", ["a"], ["a", "c"])).toBe("missed");
  expect(choiceState("d", ["a"], ["a", "c"])).toBe("idle");
});

test("letters", () => {
  expect(letterAt(0)).toBe("A");
  expect(letterAt(3)).toBe("D");
  expect(letterAt(9)).toBe("10");
});
