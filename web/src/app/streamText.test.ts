import { expect, test } from "bun:test";
import { takeWords, wordCount } from "./streamText";

test("takes whole words from the start", () => {
  expect(takeWords("Karena besi rel memuai.", 2)).toBe("Karena besi");
  expect(takeWords("Karena besi", 9)).toBe("Karena besi");
  expect(takeWords("Karena besi", 0)).toBe("");
});

test("counts words", () => {
  expect(wordCount("  satu dua  tiga ")).toBe(3);
});
