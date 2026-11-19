import { expect, test } from "bun:test";
import { GRADE_GROUPS, gradeLabel, isGrade, levelOf } from "./grades";

test("groups cover grades 1 to 12 once, in order", () => {
  expect(GRADE_GROUPS.flatMap((g) => g.grades)).toEqual([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
  expect(GRADE_GROUPS.map((g) => g.level)).toEqual(["SD", "SMP", "SMA"]);
});

test("each grade sits in the group of its level", () => {
  for (const group of GRADE_GROUPS) {
    for (const grade of group.grades) expect(levelOf(grade)).toBe(group.level);
  }
});

test("labels and the grade guard", () => {
  expect(gradeLabel(7)).toBe("Kelas 7 SMP");
  expect(gradeLabel(12)).toBe("Kelas 12 SMA");
  expect(isGrade(6)).toBe(true);
  expect(isGrade(0)).toBe(false);
  expect(isGrade(13)).toBe(false);
  expect(isGrade(2.5)).toBe(false);
});
