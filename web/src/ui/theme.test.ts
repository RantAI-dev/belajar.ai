// The contrast gate: every pair in tokenPairs.ts, in both themes, with the values theme.css
// actually defines. A dark token that drops a pair below WCAG AA turns this red.
import { describe, expect, test } from "bun:test";
import { contrastRatio } from "./contrast";
import { AVATAR_COLORS } from "./initials";
import { SUBJECTS, subjectPaint } from "./subjects";
import { darkOverrideNames, parseThemeTokens, type ThemeName } from "./themeTokens";
import { MINIMUM, TOKEN_PAIRS } from "./tokenPairs";

const css = await Bun.file(new URL("./theme.css", import.meta.url)).text();
const tokens = parseThemeTokens(css);
const HEX = /^#[0-9a-f]{6}$/;

describe.each(["light", "dark"] as const satisfies readonly ThemeName[])("%s theme", (theme) => {
  const map = tokens[theme];

  test("every token a pair names is a #rrggbb colour", () => {
    const bad = TOKEN_PAIRS.flatMap((pair) => [pair.fg, pair.bg]).filter(
      (name) => !HEX.test(map[name] ?? ""),
    );
    expect(bad).toEqual([]);
  });

  test("every token pair meets WCAG AA", () => {
    const failures = TOKEN_PAIRS.flatMap((pair) => {
      const ratio = contrastRatio(map[pair.fg] ?? "#000000", map[pair.bg] ?? "#000000");
      const min = MINIMUM[pair.need];
      return ratio >= min
        ? []
        : [
            `${pair.fg} ${map[pair.fg]} on ${pair.bg} ${map[pair.bg]} = ${ratio.toFixed(2)}:1, needs ${min}:1 (${pair.use})`,
          ];
    });
    expect(failures).toEqual([]);
  });
});

describe("theme.css structure", () => {
  test("dark only overrides tokens that light defines", () => {
    const light = new Set(Object.keys(tokens.light));
    expect(darkOverrideNames(css).filter((name) => !light.has(name))).toEqual([]);
  });

  test("the dark theme overrides the surfaces and inks", () => {
    for (const name of ["color-page", "color-surface", "color-ink", "color-ink-muted", "focus"]) {
      expect(tokens.dark[name]).not.toBe(tokens.light[name]);
    }
  });

  test("the light subject tokens equal the palette in subjects.ts", () => {
    for (const s of SUBJECTS) {
      const t = `color-subject-${s.id}`;
      const css = [t, `${t}-ink`, `${t}-tint`, `${t}-edge`, `${t}-shadow`].map(
        (name) => tokens.light[name] ?? "",
      );
      expect([s.color, s.ink, s.tint, s.edge, s.shadow].map((c) => c.toLowerCase())).toEqual(css);
    }
  });

  test("components paint subjects and avatars through tokens that exist", () => {
    const refs = [
      ...SUBJECTS.flatMap((s) => Object.values(subjectPaint(s))),
      ...AVATAR_COLORS.flatMap((c) => [c.bg, c.fg]),
    ];
    const missing = refs
      .map((ref) => ref.match(/^var\(--([\w-]+)\)$/)?.[1] ?? ref)
      .filter((name) => tokens.light[name] === undefined || tokens.dark[name] === undefined);
    expect(missing).toEqual([]);
  });
});

describe("parseThemeTokens", () => {
  test("dark is light plus overrides, and var() resolves within the theme", () => {
    const parsed = parseThemeTokens(`
      @theme static { --color-a: #FFFFFF; --color-b: #000000; }
      :root { --color-c: var(--color-a); }
      [data-theme="dark"] { --color-a: #111111; }
    `);
    expect(parsed.light).toEqual({
      "color-a": "#ffffff",
      "color-b": "#000000",
      "color-c": "#ffffff",
    });
    expect(parsed.dark).toEqual({
      "color-a": "#111111",
      "color-b": "#000000",
      "color-c": "#111111",
    });
  });
});
