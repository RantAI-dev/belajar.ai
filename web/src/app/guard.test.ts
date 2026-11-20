import { describe, expect, test } from "bun:test";
import { guardRedirect, safeReturnPath } from "./guard";

const at = (pathname: string, search = "") => ({ pathname, search });

describe("guardRedirect", () => {
  test("signed out: private routes go to /masuk and remember where they were going", () => {
    expect(guardRedirect(at("/"), false)).toBe("/masuk");
    expect(guardRedirect(at("/latihan/set-1"), false)).toBe("/masuk?ke=%2Flatihan%2Fset-1");
    expect(guardRedirect(at("/tanya", "?q=1"), false)).toBe("/masuk?ke=%2Ftanya%3Fq%3D1");
    expect(guardRedirect(at("/tidak-ada"), false)).toBe("/masuk?ke=%2Ftidak-ada");
  });

  test("after logging out: /masuk without remembering the page", () => {
    expect(guardRedirect(at("/akun"), false, { remember: false })).toBe("/masuk");
  });

  test("signed out: /masuk and /daftar show", () => {
    expect(guardRedirect(at("/masuk"), false)).toBeNull();
    expect(guardRedirect(at("/daftar", "?ke=%2Fakun"), false)).toBeNull();
    expect(guardRedirect(at("/masuk/"), false)).toBeNull();
  });

  test("signed out: the Google grade step shows; signed in, it goes home", () => {
    expect(guardRedirect(at("/daftar/google"), false)).toBeNull();
    expect(guardRedirect(at("/daftar/google"), true)).toBe("/");
  });

  test("signed in: /masuk and /daftar go back to where they were going, else home", () => {
    expect(guardRedirect(at("/masuk"), true)).toBe("/");
    expect(guardRedirect(at("/daftar"), true)).toBe("/");
    expect(guardRedirect(at("/masuk", "?ke=%2Flatihan%2Fset-1"), true)).toBe("/latihan/set-1");
  });

  test("signed in: private routes show", () => {
    expect(guardRedirect(at("/"), true)).toBeNull();
    expect(guardRedirect(at("/akun"), true)).toBeNull();
  });
});

describe("safeReturnPath", () => {
  test("only same-origin paths that are not the auth pages", () => {
    expect(safeReturnPath("/belajar/ipa")).toBe("/belajar/ipa");
    expect(safeReturnPath(null)).toBe("/");
    expect(safeReturnPath("https://evil.example")).toBe("/");
    expect(safeReturnPath("//evil.example")).toBe("/");
    expect(safeReturnPath("/\\evil.example")).toBe("/");
    expect(safeReturnPath("/masuk?ke=/masuk")).toBe("/");
  });
});
