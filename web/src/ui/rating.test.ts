import { expect, test } from "bun:test";
import { clampRating, starsForScore } from "./rating";

test("all correct earns 3 stars, 60% earns 2, anything right earns 1", () => {
  expect(starsForScore(5, 5)).toBe(3);
  expect(starsForScore(4, 5)).toBe(2);
  expect(starsForScore(3, 5)).toBe(2);
  expect(starsForScore(2, 5)).toBe(1);
  expect(starsForScore(0, 5)).toBe(0);
});

test("an empty set earns nothing", () => {
  expect(starsForScore(0, 0)).toBe(0);
});

test("ratings clamp to whole 0..3", () => {
  expect(clampRating(-2)).toBe(0);
  expect(clampRating(1.2)).toBe(1);
  expect(clampRating(2.6)).toBe(3);
  expect(clampRating(9)).toBe(3);
  expect(clampRating(Number.NaN)).toBe(0);
});
