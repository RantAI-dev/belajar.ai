import { expect, test } from "bun:test";
import { resultMood } from "./resultMood";

test("two stars or more celebrates, with no extra line", () => {
  expect(resultMood(5, 5)).toEqual({ expression: "celebration", cheer: null });
  expect(resultMood(3, 5)).toEqual({ expression: "celebration", cheer: null });
});

test("below two stars the mascot stays cheerful and encourages", () => {
  for (const [correct, total] of [
    [2, 5],
    [1, 5],
    [0, 5],
    [0, 0],
  ] as const) {
    const mood = resultMood(correct, total);
    expect(mood.expression).toBe("cheerful");
    expect(mood.cheer).not.toBeNull();
  }
});

test("a low score never makes the mascot sad", () => {
  for (let correct = 0; correct <= 10; correct++)
    expect(resultMood(correct, 10).expression).not.toBe("sad");
});
