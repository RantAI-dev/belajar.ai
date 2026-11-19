import { expect, test } from "bun:test";
import { hasErrors, validateRegister } from "./validate";

const good = { nickname: "Rara", username: "rara_7", password: "rahasia123", grade: 7 } as const;

test("a complete form has no errors", () => {
  expect(hasErrors(validateRegister(good))).toBe(false);
});

test("each missing or bad field gets its own message", () => {
  const e = validateRegister({ nickname: " ", username: "Ra", password: "123", grade: null });
  expect(Object.keys(e).sort()).toEqual(["grade", "nickname", "password", "username"]);
});

test("username allows lowercase, digits, dot and underscore only", () => {
  expect(validateRegister({ ...good, username: "rara.ayu_7" }).username).toBeUndefined();
  expect(validateRegister({ ...good, username: "Rara" }).username).toBeDefined();
  expect(validateRegister({ ...good, username: "ra ra" }).username).toBeDefined();
});
