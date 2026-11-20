import { beforeAll, describe, expect, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import { loadMath } from "./math";
import { RichText } from "./RichText";

// Server rendering shows where a formula goes (`data-math`); KaTeX's own markup is put in on the
// client, and math.test.ts checks that markup.
const html = (text: string, streaming = false) =>
  renderToStaticMarkup(<RichText text={text} streaming={streaming} />);

beforeAll(async () => {
  expect(await loadMath()).toBe(true);
});

describe("RichText", () => {
  test("draws an inline fraction with KaTeX", () => {
    const out = html("Jadi $ \\frac{1}{4} + \\frac{3}{4} = 1 $.");
    expect(out).toContain('data-math="inline"');
    expect(out).not.toContain("$");
  });

  test("a long display equation scrolls in its own box", () => {
    const out = html("$$x = \\frac{-b \\pm \\sqrt{b^2 - 4ac}}{2a}$$");
    expect(out).toContain("overflow-x-auto");
    expect(out).toContain('data-math="display"');
  });

  test("invalid LaTeX shows its source as code, with no error box", () => {
    const out = html("Coba $\\frac{1$ ya.");
    expect(out).toContain("<code");
    expect(out).toContain("$\\frac{1$");
    expect(out).not.toContain("katex-error");
    expect(out).not.toContain("ParseError");
  });

  test("money stays text", () => {
    const out = html("Harganya Rp5.000 dan $3 saja.");
    expect(out).not.toContain("data-math");
    expect(out).toContain("Rp5.000 dan $3 saja.");
  });

  test("citations stay readable beside maths", () => {
    const out = html("Hasilnya $\\frac{1}{2}$[1].");
    expect(out).toContain("data-math");
    expect(out).toMatch(/<span class="sr-only">sumber <\/span>\[1\]/);
  });

  test("model text is escaped, never markup", () => {
    const out = html("<img src=x onerror=alert(1)> $x$");
    expect(out).not.toContain("<img");
    expect(out).toContain("&lt;img");
  });

  test("a mermaid block waits quietly, also while still streaming", () => {
    expect(html("```mermaid\ngraph TD\nA-->B\n```")).toContain("aku gambar diagramnya");
    expect(html("```mermaid\ngraph TD\nA--", true)).toContain("aku gambar diagramnya");
    expect(html("```mermaid\ngraph TD\nA--", true)).not.toContain("<code");
  });
});
