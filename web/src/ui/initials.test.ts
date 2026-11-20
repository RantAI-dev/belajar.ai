import { expect, test } from "bun:test";
import { cx } from "./cx";
import { AVATAR_COLORS, avatarColorIndex, initialsOf } from "./initials";

test("initials from up to two words", () => {
  expect(initialsOf("Siswa A")).toBe("SA");
  expect(initialsOf("  budi  ")).toBe("B");
  expect(initialsOf("Rara Ayu Lestari")).toBe("RA");
  expect(initialsOf("")).toBe("");
});

// Contrast of each avatar pair, in both themes, is in theme.test.ts (tokenPairs.ts).
test("a name keeps its colour, and every colour is a theme token", () => {
  expect(avatarColorIndex("Siswa A")).toBe(avatarColorIndex("Siswa A"));
  expect(avatarColorIndex("Siswa A")).toBeLessThan(AVATAR_COLORS.length);
  for (const c of AVATAR_COLORS) {
    expect(c.bg).toMatch(/^var\(--color-avatar-\d-bg\)$/);
    expect(c.fg).toMatch(/^var\(--color-avatar-\d-fg\)$/);
  }
});

test("cx drops empty parts", () => {
  expect(cx("a", false, undefined, "", "b", null)).toBe("a b");
});
