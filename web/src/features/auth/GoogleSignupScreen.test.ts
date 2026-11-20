import { describe, expect, test } from "bun:test";
import { validateGoogleSignup } from "./GoogleSignupScreen";

describe("validateGoogleSignup", () => {
  test("a nickname and a grade is enough", () => {
    expect(validateGoogleSignup("  Budi ", 5)).toEqual({});
  });

  test("every missing field is named at once", () => {
    const errors = validateGoogleSignup("   ", null);
    expect(Object.keys(errors).sort()).toEqual(["grade", "nickname"]);
  });

  test("a nickname over the limit is refused", () => {
    expect(validateGoogleSignup("x".repeat(41), 7).nickname).toBeString();
  });
});
