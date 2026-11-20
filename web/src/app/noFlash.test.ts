// Runs the inline script in web/index.html against fake browser globals: it must pick the stored
// choice, fall back to the system setting, survive a storage that throws, and agree with
// resolveTheme for every case.
import { expect, test } from "bun:test";
import { resolveTheme, type ThemePreference } from "../ui";

const html = await Bun.file(new URL("../../index.html", import.meta.url)).text();
const script = html.match(/<script id="theme-boot">([\s\S]*?)<\/script>/)?.[1] ?? "";

const run = (stored: string | null | Error, prefersDark: boolean): string | undefined => {
  const attrs: Record<string, string> = {};
  const window = {
    localStorage: {
      getItem: (key: string) => {
        if (stored instanceof Error) throw stored;
        return key === "belajar.theme" ? stored : null;
      },
    },
    matchMedia: (query: string) => ({
      matches: query === "(prefers-color-scheme: dark)" && prefersDark,
    }),
  };
  const document = {
    documentElement: {
      setAttribute: (n: string, v: string) => {
        attrs[n] = v;
      },
    },
  };
  new Function("window", "document", script)(window, document);
  return attrs["data-theme"];
};

test("index.html has the boot script", () => {
  expect(script).toContain("data-theme");
});

test("the boot script resolves like the app does", () => {
  for (const preference of ["system", "light", "dark"] as ThemePreference[]) {
    for (const prefersDark of [false, true]) {
      expect(run(preference, prefersDark)).toBe(resolveTheme(preference, prefersDark));
    }
  }
});

test("nothing stored, junk, or a storage that throws: follow the system", () => {
  expect(run(null, true)).toBe("dark");
  expect(run("purple", false)).toBe("light");
  expect(run(new Error("SecurityError"), true)).toBe("dark");
});
