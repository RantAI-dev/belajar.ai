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

test("username follows the server's rule: 3-32 letters, digits, dot, underscore", () => {
  expect(validateRegister({ ...good, username: "rara.ayu_7" }).username).toBeUndefined();
  expect(validateRegister({ ...good, username: "Rara" }).username).toBeUndefined();
  expect(validateRegister({ ...good, username: "r".repeat(32) }).username).toBeUndefined();
  expect(validateRegister({ ...good, username: "r".repeat(33) }).username).toBeDefined();
  expect(validateRegister({ ...good, username: "ra ra" }).username).toBeDefined();
});

test("a nickname may be up to 40 characters", () => {
  expect(validateRegister({ ...good, nickname: "n".repeat(40) }).nickname).toBeUndefined();
  expect(validateRegister({ ...good, nickname: "n".repeat(41) }).nickname).toBeDefined();
});
