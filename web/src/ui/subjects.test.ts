import { describe, expect, test } from "bun:test";
import { contrastRatio } from "./contrast";
import { isSubjectId, SUBJECTS } from "./subjects";

describe("subject shades", () => {
  test("ink text stays readable on the subject's own tint", () => {
    for (const s of SUBJECTS) {
      expect(contrastRatio(s.ink, s.tint)).toBeGreaterThanOrEqual(4.5);
    }
  });

  test("body ink is readable on every tint", () => {
    for (const s of SUBJECTS) {
      expect(contrastRatio("#10233a", s.tint)).toBeGreaterThanOrEqual(4.5);
    }
  });

  test("isSubjectId narrows only known ids", () => {
    expect(isSubjectId("mtk")).toBe(true);
    expect(isSubjectId("MTK")).toBe(false);
  });
});

// Text/background pairs the components use. Small text needs 4.5:1.
const SMALL_TEXT_PAIRS: ReadonlyArray<readonly [string, string, string]> = [
  ["muted ink on white", "#3d5470", "#ffffff"],
  ["muted ink on page", "#3d5470", "#f3f9ff"],
  ["faint ink (placeholders) on white", "#55697f", "#ffffff"],
  ["white on primary", "#ffffff", "#0069a8"],
  ["primary on white", "#0069a8", "#ffffff"],
  ["primary-strong on primary-soft", "#00598a", "#e0f0fd"],
  ["sky text on primary hero", "#cfe7fb", "#0069a8"],
  ["warm ink on warm card", "#5c4400", "#fff8d6"],
  ["star ink on warm", "#7a5a00", "#fff8d6"],
  ["streak ink on streak pill", "#8a4b0f", "#fff4e5"],
  ["oops ink on white", "#a3501a", "#ffffff"],
  ["oops ink on oops soft", "#a3501a", "#fff4ea"],
  ["success edge on success soft", "#157a50", "#eafaf2"],
  ["sun ink on sun button", "#3b2a00", "#ffd23f"],
];

describe("token contrast", () => {
  for (const [name, fg, bg] of SMALL_TEXT_PAIRS) {
    test(name, () => {
      expect(contrastRatio(fg, bg)).toBeGreaterThanOrEqual(4.5);
    });
  }

  test("white on success is only used for large text (3:1)", () => {
    expect(contrastRatio("#ffffff", "#22a06b")).toBeGreaterThanOrEqual(3);
  });
});
