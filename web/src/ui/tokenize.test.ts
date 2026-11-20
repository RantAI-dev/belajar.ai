import { describe, expect, test } from "bun:test";
import { cutOutsideMaths, type Segment, tokenize } from "./tokenize";

const kinds = (segs: readonly Segment[]) => segs.map((s) => s.kind);
const maths = (segs: readonly Segment[]) =>
  segs.flatMap((s) => (s.kind === "math" ? [{ tex: s.tex, display: s.display }] : []));
const textOf = (segs: readonly Segment[]) =>
  segs.map((s) => (s.kind === "text" ? s.text : `<${s.kind}>`)).join("");

describe("inline and display maths", () => {
  test("the upstream's spaced inline formula is maths", () => {
    const segs = tokenize("Jadi $ \\frac{1}{4} + \\frac{3}{4} = 1 $, utuh.");
    expect(kinds(segs)).toEqual(["text", "math", "text"]);
    expect(maths(segs)).toEqual([{ tex: "\\frac{1}{4} + \\frac{3}{4} = 1", display: false }]);
    expect(segs[1]).toMatchObject({ source: "$ \\frac{1}{4} + \\frac{3}{4} = 1 $" });
  });

  test("a formula that starts with a number is maths when it has a maths sign", () => {
    expect(maths(tokenize("Selesaikan $3x + 2 = 11$."))).toEqual([
      { tex: "3x + 2 = 11", display: false },
    ]);
  });

  test("$$…$$ is a display formula and may span lines", () => {
    const segs = tokenize("Rumusnya:\n$$\na^2 + b^2\n= c^2\n$$\nSelesai.");
    expect(maths(segs)).toEqual([{ tex: "a^2 + b^2\n= c^2", display: true }]);
    expect(textOf(segs)).toBe("Rumusnya:\n<math>\nSelesai.");
  });

  test("an inline formula does not cross a blank line", () => {
    const segs = tokenize("Ada $x\n\ndan y$ di sini.");
    expect(maths(segs)).toEqual([]);
    expect(textOf(segs)).toBe("Ada $x\n\ndan y$ di sini.");
  });
});

describe("currency is not maths", () => {
  test("rupiah and a lone dollar stay text", () => {
    const segs = tokenize("Harganya Rp5.000 dan $3 saja.");
    expect(kinds(segs)).toEqual(["text"]);
    expect(textOf(segs)).toBe("Harganya Rp5.000 dan $3 saja.");
  });

  test("two prices in one sentence stay text", () => {
    const segs = tokenize("Buku $5 dan pensil $3.");
    expect(maths(segs)).toEqual([]);
    expect(textOf(segs)).toBe("Buku $5 dan pensil $3.");
  });

  test("a price before a formula leaves the formula alone", () => {
    const segs = tokenize("Uangnya $10, jadi $x + 5 = 15$.");
    expect(maths(segs)).toEqual([{ tex: "x + 5 = 15", display: false }]);
    expect(textOf(segs)).toBe("Uangnya $10, jadi <math>.");
  });

  test("a dollar after a letter (US$5) is money", () => {
    expect(maths(tokenize("Kurs US$5 dan US$7 beda."))).toEqual([]);
  });

  test("an escaped dollar is a literal dollar", () => {
    const segs = tokenize("Harga \\$5 dan \\$7.");
    expect(kinds(segs)).toEqual(["text"]);
    expect(textOf(segs)).toBe("Harga $5 dan $7.");
  });
});

describe("text around maths", () => {
  test("[n] is left to Tanya's citation parser, even beside a formula", () => {
    expect(textOf(tokenize("Jumlahnya $\\frac{1}{2}$[1]."))).toBe("Jumlahnya <math>[1].");
  });

  test("brackets inside a formula belong to the formula", () => {
    expect(kinds(tokenize("$[1] + [2]$"))).toEqual(["math"]);
  });

  test("tags and other markup pass through as text", () => {
    const text = "Lihat <gambar 3> dan [Gambar 1.2] di bukumu.";
    expect(textOf(tokenize(text))).toBe(text);
  });
});

describe("cutOutsideMaths", () => {
  test("a cut inside a formula moves to the formula's start", () => {
    const text = "Jadi $ \\frac{1}{4} = 1 $";
    expect(cutOutsideMaths(text, text.length - 1)).toBe(5);
    expect(cutOutsideMaths(text, 4)).toBe(4);
  });

  test("a cut inside a fenced block moves to the fence", () => {
    const text = "A:\n```mermaid\ngraph TD\nA-->B\n```\nB";
    expect(cutOutsideMaths(text, 12)).toBe(3);
    expect(cutOutsideMaths(text, text.length - 1)).toBe(text.length - 1);
  });

  test("money is not a formula, so a cut there stays", () => {
    expect(cutOutsideMaths("Rp5.000 dan $3", 13)).toBe(13);
  });
});

describe("a stream that stops mid-way", () => {
  test("an unclosed $ is plain text until it closes", () => {
    const partial = tokenize("Hasilnya $\\frac{1}{4", { streaming: true });
    expect(kinds(partial)).toEqual(["text"]);
    expect(textOf(partial)).toBe("Hasilnya $\\frac{1}{4");
    expect(kinds(tokenize("Hasilnya $\\frac{1}{4}$", { streaming: true }))).toEqual([
      "text",
      "math",
    ]);
  });

  test("an unclosed $$ is plain text", () => {
    expect(kinds(tokenize("Rumus:\n$$a^2 + ", { streaming: true }))).toEqual(["text"]);
    expect(kinds(tokenize("Rumus: $$", { streaming: true }))).toEqual(["text"]);
  });

  test("every prefix of an answer tokenizes without throwing", () => {
    const answer =
      "Pecahan $\\frac{1}{4}$ dan $$\\sum_{i=1}^{n} i$$ lalu\n```mermaid\ngraph TD\nA-->B\n```\nselesai $5.";
    for (let i = 0; i <= answer.length; i++) {
      expect(() => tokenize(answer.slice(0, i), { streaming: true })).not.toThrow();
    }
  });
});

describe("mermaid fences", () => {
  const diagram = "Alurnya:\n```mermaid\ngraph TD\n  A[Air] --> B[Uap]\n```\nItu dia.";

  test("a closed fence is a complete diagram", () => {
    const segs = tokenize(diagram);
    expect(kinds(segs)).toEqual(["text", "mermaid", "text"]);
    expect(segs[1]).toEqual({
      kind: "mermaid",
      code: "graph TD\n  A[Air] --> B[Uap]",
      complete: true,
    });
  });

  test("while streaming, an unclosed fence is a placeholder", () => {
    const segs = tokenize("Alurnya:\n```mermaid\ngraph TD\n  A --", { streaming: true });
    expect(segs[1]).toEqual({ kind: "mermaid", code: "graph TD\n  A --", complete: false });
  });

  test("while streaming, a half-written opening line is plain text", () => {
    expect(kinds(tokenize("Alurnya:\n```merm", { streaming: true }))).toEqual(["text"]);
  });

  test("an unclosed fence in a finished answer is shown as code", () => {
    expect(kinds(tokenize("```mermaid\ngraph TD"))).toEqual(["code"]);
  });

  test("maths inside a code fence is not maths", () => {
    expect(kinds(tokenize("```\nharga $5 dan $x = 2$\n```"))).toEqual(["code"]);
  });
});
