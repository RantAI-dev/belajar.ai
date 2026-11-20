import { beforeAll, describe, expect, test } from "bun:test";
import { loadMath, renderMath } from "./math";

beforeAll(async () => {
  expect(await loadMath()).toBe(true);
});

describe("renderMath", () => {
  test("a fraction is KaTeX HTML", () => {
    const r = renderMath("\\frac{1}{4} + \\frac{3}{4} = 1", false);
    expect(r.kind).toBe("html");
    expect(r.kind === "html" ? r.html : "").toContain('class="katex"');
  });

  test("display mode marks the formula as a display", () => {
    const r = renderMath("\\sum_{i=1}^{n} i", true);
    expect(r.kind === "html" ? r.html : "").toContain("katex-display");
  });

  test("invalid LaTeX is invalid, not KaTeX's red error", () => {
    expect(renderMath("\\frac{1", false)).toEqual({ kind: "invalid" });
    expect(renderMath("\\notacommand{x}", false)).toEqual({ kind: "invalid" });
  });

  test("links and raw HTML commands are refused", () => {
    expect(renderMath("\\href{https://example.com}{klik}", false)).toEqual({ kind: "invalid" });
    expect(renderMath("\\url{https://example.com}", false)).toEqual({ kind: "invalid" });
    expect(renderMath("\\htmlClass{x}{y}", false)).toEqual({ kind: "invalid" });
  });
});
