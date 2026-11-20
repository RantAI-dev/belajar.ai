import { describe, expect, test } from "bun:test";
import {
  applyTheme,
  isThemePreference,
  readStoredTheme,
  resolveTheme,
  storeTheme,
  THEME_STORAGE_KEY,
} from "./themePreference";

const memory = (initial: Record<string, string> = {}) => {
  const data = { ...initial };
  return {
    data,
    getItem: (key: string) => data[key] ?? null,
    setItem: (key: string, value: string) => {
      data[key] = value;
    },
  };
};

const throwing = {
  getItem: (): string | null => {
    throw new Error("SecurityError");
  },
  setItem: (): void => {
    throw new Error("QuotaExceededError");
  },
};

describe("theme preference", () => {
  test("system follows the device; light and dark are fixed", () => {
    expect(resolveTheme("system", true)).toBe("dark");
    expect(resolveTheme("system", false)).toBe("light");
    expect(resolveTheme("light", true)).toBe("light");
    expect(resolveTheme("dark", false)).toBe("dark");
  });

  test("only the three values are preferences", () => {
    expect(["system", "light", "dark"].every(isThemePreference)).toBe(true);
    expect([null, "", "Dark", "auto", 1].some(isThemePreference)).toBe(false);
  });

  test("a stored choice is read back; anything else is system", () => {
    expect(readStoredTheme(() => memory({ [THEME_STORAGE_KEY]: "dark" }))).toBe("dark");
    expect(readStoredTheme(() => memory({ [THEME_STORAGE_KEY]: "purple" }))).toBe("system");
    expect(readStoredTheme(() => memory())).toBe("system");
    expect(readStoredTheme(() => null)).toBe("system");
  });

  test("storage that throws never breaks the app", () => {
    expect(readStoredTheme(() => throwing)).toBe("system");
    expect(() => storeTheme(() => throwing, "dark")).not.toThrow();
    expect(
      readStoredTheme(() => {
        throw new Error("localStorage is not available");
      }),
    ).toBe("system");
  });

  test("a choice is stored under the key the inline script reads", () => {
    const storage = memory();
    storeTheme(() => storage, "light");
    expect(storage.data).toEqual({ [THEME_STORAGE_KEY]: "light" });
  });

  test("the resolved theme goes on data-theme", () => {
    const attrs: Record<string, string> = {};
    const root = {
      setAttribute: (n: string, v: string) => {
        attrs[n] = v;
      },
    };
    applyTheme(root, "dark");
    expect(attrs).toEqual({ "data-theme": "dark" });
  });
});
