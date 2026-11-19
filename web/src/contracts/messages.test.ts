import { describe, expect, test } from "bun:test";
import {
  fieldHint,
  loginMessage,
  MESSAGES,
  passwordChangeMessage,
  rateLimitMessage,
  registerMessage,
} from "./messages";
import { type ApiProblem, networkProblem } from "./problem";

const problem = (code: ApiProblem["code"], extra: Partial<ApiProblem> = {}): ApiProblem => ({
  status: 400,
  code,
  fieldErrors: {},
  retryAfterSeconds: null,
  requestId: "r",
  ...extra,
});

describe("messages", () => {
  test("login: wrong credentials, rate limit rounded up, network, anything else", () => {
    expect(loginMessage(problem("unauthenticated")).form).toBe(MESSAGES.wrongCredentials);
    expect(loginMessage(problem("rate_limited", { retryAfterSeconds: 61 })).form).toBe(
      "Terlalu banyak percobaan. Coba lagi dalam 2 menit.",
    );
    expect(loginMessage(networkProblem()).form).toBe(MESSAGES.network);
    expect(loginMessage(problem("internal")).form).toBe(MESSAGES.generic);
    expect(loginMessage(problem("upstream_failed")).form).toBe(MESSAGES.generic);
  });

  test("rate limit minutes: round up, at least one", () => {
    expect(rateLimitMessage(1)).toContain("dalam 1 menit");
    expect(rateLimitMessage(60)).toContain("dalam 1 menit");
    expect(rateLimitMessage(900)).toContain("dalam 15 menit");
    expect(rateLimitMessage(null)).toContain("dalam 1 menit");
  });

  test("register: a taken username goes next to the username field", () => {
    expect(registerMessage(problem("conflict"))).toEqual({
      form: null,
      fields: { username: MESSAGES.usernameTaken },
    });
  });

  test("register: field codes become per-field hints keyed without the slash", () => {
    const m = registerMessage(
      problem("invalid_request", {
        fieldErrors: {
          "/username": "invalid_format",
          "/password": "too_short",
          "/grade": "out_of_range",
        },
      }),
    );
    expect(m.form).toBeNull();
    expect(Object.keys(m.fields).sort()).toEqual(["grade", "password", "username"]);
    expect(m.fields.password).toBe("Kata sandi minimal 8 karakter.");
  });

  test("every field code has a hint, never the server's text", () => {
    for (const code of ["required", "too_short", "too_long", "invalid_format", "out_of_range"]) {
      for (const field of [
        "/username",
        "/password",
        "/nickname",
        "/grade",
        "/newPassword",
        "/other",
      ]) {
        expect(fieldHint(field, code).length).toBeGreaterThan(0);
      }
    }
    expect(fieldHint("/nickname", "too_long")).toBe("Nama panggilan paling panjang 40 huruf.");
    expect(fieldHint("/whatever", "made_up")).toBe("Bentuk isiannya belum benar.");
  });

  test("change password: wrong current password, short new password", () => {
    expect(passwordChangeMessage(problem("permission_denied")).form).toBe(
      MESSAGES.wrongCurrentPassword,
    );
    expect(
      passwordChangeMessage(
        problem("invalid_request", { fieldErrors: { "/newPassword": "too_short" } }),
      ).fields,
    ).toEqual({ newPassword: "Kata sandi baru minimal 8 karakter." });
  });

  test("invalid_request without field errors is one sentence", () => {
    expect(registerMessage(problem("invalid_request")).form).toBe(MESSAGES.checkFields);
  });
});
