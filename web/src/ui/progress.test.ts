import { expect, test } from "bun:test";
import { progressPercent } from "./progress";

test("percent of max, rounded", () => {
  expect(progressPercent(3, 5)).toBe(60);
  expect(progressPercent(1, 3)).toBe(33);
});

test("clamps to 0..100 and guards a bad max", () => {
  expect(progressPercent(7, 5)).toBe(100);
  expect(progressPercent(-1, 5)).toBe(0);
  expect(progressPercent(2, 0)).toBe(0);
  expect(progressPercent(Number.NaN, 5)).toBe(0);
});
