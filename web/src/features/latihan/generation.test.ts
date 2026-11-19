import { describe, expect, test } from "bun:test";
import { generationText, slotsOf } from "./generation";

describe("generation progress", () => {
  test("each stage has its own headline; generating counts the questions", () => {
    expect(generationText(null).headline).toBe("Aku siap membuatkan soal!");
    expect(generationText({ stage: "generating", ready: 2, total: 5 })).toEqual({
      headline: "Sebentar ya, soalmu sedang dibuat…",
      detail: "2 dari 5 soal sudah jadi.",
    });
    expect(generationText({ stage: "checking", ready: 5, total: 5 }).headline).toBe(
      "Aku cek dulu setiap soal…",
    );
    expect(generationText({ stage: "ready", ready: 5, total: 5 }).headline).toBe(
      "Soalmu sudah siap!",
    );
  });

  test("slots fill as questions arrive, and only for sets of five or fewer", () => {
    expect(slotsOf({ stage: "generating", ready: 2, total: 5 })).toEqual([
      { n: 1, done: true },
      { n: 2, done: true },
      { n: 3, done: false },
      { n: 4, done: false },
      { n: 5, done: false },
    ]);
    expect(slotsOf({ stage: "generating", ready: 2, total: 10 })).toEqual([]);
  });
});
