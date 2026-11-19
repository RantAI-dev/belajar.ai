import { expect, test } from "bun:test";
import { contrastRatio } from "./contrast";
import { cx } from "./cx";
import { AVATAR_COLORS, avatarColorIndex, initialsOf } from "./initials";

test("initials from up to two words", () => {
  expect(initialsOf("Siswa A")).toBe("SA");
  expect(initialsOf("  budi  ")).toBe("B");
  expect(initialsOf("Rara Ayu Lestari")).toBe("RA");
  expect(initialsOf("")).toBe("");
});

test("a name keeps its colour and every colour pair is readable", () => {
  expect(avatarColorIndex("Siswa A")).toBe(avatarColorIndex("Siswa A"));
  expect(avatarColorIndex("Siswa A")).toBeLessThan(AVATAR_COLORS.length);
  for (const c of AVATAR_COLORS) expect(contrastRatio(c.fg, c.bg)).toBeGreaterThanOrEqual(4.5);
});

test("cx drops empty parts", () => {
  expect(cx("a", false, undefined, "", "b", null)).toBe("a b");
});
