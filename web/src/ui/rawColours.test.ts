// Every colour comes from a theme token, so the dark theme reaches every screen. This fails on a
// hex, rgb()/hsl() or Tailwind palette colour (bg-white, text-gray-700) anywhere in src except
// theme.css itself, subjects.ts (the light palette theme.css is checked against) and tests.
import { expect, test } from "bun:test";
import { Glob } from "bun";

const SRC = new URL("..", import.meta.url).pathname;
const ALLOWED = new Set(["ui/theme.css", "ui/subjects.ts"]);

const PALETTE =
  "white|black|slate|gray|zinc|neutral|stone|red|orange|amber|yellow|lime|green|emerald|teal|cyan|sky|blue|indigo|violet|purple|fuchsia|pink|rose";
const UTILITY =
  "bg|text|border|border-[trblxy]|ring|ring-offset|fill|stroke|from|via|to|shadow|outline|divide|placeholder|decoration|accent|caret";
const RULES: ReadonlyArray<readonly [string, RegExp]> = [
  ["hex colour", /(?<![\w&])#(?:[0-9a-fA-F]{8}|[0-9a-fA-F]{6}|[0-9a-fA-F]{3,4})(?![\w-])/],
  ["rgb()/hsl() colour", /(?<![a-zA-Z-])(?:rgba?|hsla?|oklch|oklab)\(/],
  [
    "Tailwind palette colour",
    new RegExp(`(?<![\\w-])(?:${UTILITY})-(?:${PALETTE})(?:-\\d{2,3})?(?![\\w-])`),
  ],
];

const rawColours = (text: string): string[] =>
  text
    .split("\n")
    .flatMap((line, i) =>
      RULES.flatMap(([what, rule]) =>
        [...line.matchAll(new RegExp(rule.source, "g"))].map((m) => `${i + 1}: ${what} ${m[0]}`),
      ),
    );

test("the scanner catches each kind of raw colour and passes tokens", () => {
  expect(rawColours('fill="#ffd23f"')).toHaveLength(1);
  expect(rawColours("bg-[#e6f5f7]")).toHaveLength(1);
  expect(rawColours("shadow-[0_6px_20px_rgb(0_105_168/0.08)]")).toHaveLength(1);
  expect(rawColours('className="bg-white text-gray-700"')).toHaveLength(2);
  expect(rawColours("hover:bg-white/60")).toHaveLength(1);
  expect(rawColours("bg-surface text-on-fill border-line-strong bg-transparent")).toEqual([]);
  expect(rawColours('href="#main" id="#1"')).toEqual([]);
  expect(rawColours("var(--color-subject-ipa-tint)")).toEqual([]);
});

test("no raw colour outside theme.css", async () => {
  const found: string[] = [];
  for await (const path of new Glob("**/*.{ts,tsx,css}").scan(SRC)) {
    if (ALLOWED.has(path) || /\.test\.tsx?$/.test(path) || path.endsWith(".gen.ts")) continue;
    const text = await Bun.file(`${SRC}${path}`).text();
    found.push(...rawColours(text).map((hit) => `${path}:${hit}`));
  }
  expect(found).toEqual([]);
});
