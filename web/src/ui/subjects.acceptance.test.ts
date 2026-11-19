// Acceptance test for the design system's subject palette. Written by the lead before the slice
// started; the slice must make it pass without editing this file.
import { describe, expect, test } from "bun:test";
import { SUBJECTS, subjectById } from "./subjects";

const luminance = (hex: string): number => {
  const channels = [1, 3, 5].map((i) => Number.parseInt(hex.slice(i, i + 2), 16) / 255);
  const [r = 0, g = 0, b = 0] = channels.map((c) =>
    c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4,
  );
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
};
const contrast = (a: string, b: string): number => {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return ((hi ?? 0) + 0.05) / ((lo ?? 0) + 0.05);
};

// The owner's palette: one colour per subject. Ids are shared with the server's /v1/subjects.
const EXPECTED: Record<string, string> = {
  ipa: "#388ca1",
  mtk: "#517fbb",
  bing: "#574399",
  bind: "#bb5153",
  ips: "#bb7851",
  inf: "#32836a",
  pp: "#0069a8",
  pai: "#2f7d5b",
};

describe("subject palette", () => {
  test("has exactly the eight subjects with the owner's colours", () => {
    expect(SUBJECTS.map((s) => s.id).sort()).toEqual(Object.keys(EXPECTED).sort());
    for (const subject of SUBJECTS) {
      expect(subject.color.toLowerCase()).toBe(EXPECTED[subject.id] ?? "");
      expect(subject.name.length).toBeGreaterThan(0);
    }
  });

  test("text in a subject's colour uses its ink shade, readable on white and on the page", () => {
    for (const subject of SUBJECTS) {
      expect(contrast(subject.ink, "#ffffff")).toBeGreaterThanOrEqual(4.5);
      expect(contrast(subject.ink, "#f3f9ff")).toBeGreaterThanOrEqual(4.5);
    }
  });

  test("white on a subject fill is at least large-text contrast", () => {
    for (const subject of SUBJECTS) {
      expect(contrast("#ffffff", subject.color)).toBeGreaterThanOrEqual(3);
    }
  });

  test("lookup by id, and an unknown id has no subject", () => {
    expect(subjectById("ipa")?.color.toLowerCase()).toBe("#388ca1");
    expect(subjectById("nope")).toBeUndefined();
  });
});
