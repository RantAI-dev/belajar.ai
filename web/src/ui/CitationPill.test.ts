import { expect, test } from "bun:test";
import { CITE } from "./CitationPill";
import { contrastRatio } from "./contrast";
import { parseThemeTokens } from "./themeTokens";

const tokens = parseThemeTokens(await Bun.file(new URL("./theme.css", import.meta.url)).text());
const name = (ref: string) => ref.match(/^var\(--([\w-]+)\)$/)?.[1] ?? ref;

test("pill and marker text stay readable (>= 4.5:1), in both themes", () => {
  for (const theme of [tokens.light, tokens.dark]) {
    const of = (ref: string) => theme[name(ref)] ?? "#000000";
    const ink = theme["color-ink"] ?? "#000000";
    expect(
      contrastRatio(theme["color-on-fill"] ?? "#000000", of(CITE.strong)),
    ).toBeGreaterThanOrEqual(4.5);
    expect(contrastRatio(ink, of(CITE.soft))).toBeGreaterThanOrEqual(4.5);
    expect(contrastRatio(ink, of(CITE.marker))).toBeGreaterThanOrEqual(4.5);
  }
});
