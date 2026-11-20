import { describe, expect, test } from "bun:test";
import {
  type CitationSegment,
  type CiteSource,
  citationSegments,
  markedCite,
  pressCite,
  shortBookName,
} from "./citations";

const IPA = "IPA Buku Siswa Kelas VII";
const SOURCES: readonly CiteSource[] = [
  { id: "s1", number: 1, book: IPA, page: 78 },
  { id: "s2", number: 2, book: IPA, page: 81 },
  { id: "s3", number: 3, book: IPA, page: 78 },
];

const segs = (text: string, sources = SOURCES, streaming = false) =>
  citationSegments([{ text }], sources, streaming);

/** The answer as a child sees it: pills as ‹label›. */
const shown = (segments: readonly CitationSegment[]): string =>
  segments.map((s) => (s.kind === "text" ? s.text : `‹${s.label}›`)).join("");

const cites = (segments: readonly CitationSegment[]) =>
  segments.flatMap((s) => (s.kind === "cite" ? [s] : []));

describe("citationSegments", () => {
  test("a marker becomes a pill right after its word; the period stays after it", () => {
    const s = segs("Rel memuai ketika panas [1]. Kalau tidak ada celah, rel melengkung [2].");
    expect(shown(s)).toBe(
      "Rel memuai ketika panas‹hal. 78›. Kalau tidak ada celah, rel melengkung‹hal. 81›.",
    );
    expect(cites(s)).toEqual([
      {
        kind: "cite",
        id: "cite-0",
        sourceId: "s1",
        label: "hal. 78",
        ariaLabel: `Sumber: ${IPA}, halaman 78`,
      },
      {
        kind: "cite",
        id: "cite-1",
        sourceId: "s2",
        label: "hal. 81",
        ariaLabel: `Sumber: ${IPA}, halaman 81`,
      },
    ]);
  });

  test("each cite knows the sentence it closes", () => {
    const s = segs("Rel memuai ketika panas [1]. Kalau tidak ada celah, rel melengkung [2].");
    const marked = (id: string) =>
      s.flatMap((x) => (x.kind === "text" && x.sentenceOf.includes(id) ? [x.text] : [])).join("");
    expect(marked("cite-0")).toBe("Rel memuai ketika panas");
    expect(marked("cite-1")).toBe("Kalau tidak ada celah, rel melengkung");
  });

  test("a sentence starts after ! ? or a line break, and a period right before the marker ends it", () => {
    const s = segs("Wah! Besi memuai.[1]\nRel juga [2]");
    const marked = (id: string) =>
      s.flatMap((x) => (x.kind === "text" && x.sentenceOf.includes(id) ? [x.text] : [])).join("");
    expect(marked("cite-0")).toBe("Besi memuai.");
    expect(marked("cite-1")).toBe("Rel juga");
  });

  test("a decimal point does not end a sentence", () => {
    const s = segs("Panjangnya 1.5 meter [1].");
    expect(s[0]).toEqual({
      kind: "text",
      text: "Panjangnya 1.5 meter",
      emphasis: false,
      sentenceOf: ["cite-0"],
    });
  });

  test("consecutive markers to the same page collapse into one pill", () => {
    expect(shown(segs("Rel memuai [1][1]."))).toBe("Rel memuai‹hal. 78›.");
    // [1] and [3] are different numbers for the same book and page.
    expect(shown(segs("Rel memuai [1] [3]."))).toBe("Rel memuai‹hal. 78›.");
    expect(shown(segs("Rel memuai [1][2]."))).toBe("Rel memuai‹hal. 78›‹hal. 81›.");
  });

  test("the same page cited in two sentences stays two pills", () => {
    expect(cites(segs("A [1]. B [1].")).map((c) => c.id)).toEqual(["cite-0", "cite-1"]);
  });

  test("a marker with no source is removed with the space before it", () => {
    expect(shown(segs("Rel memuai [9]. Lalu [1][9] melengkung [2]."))).toBe(
      "Rel memuai. Lalu‹hal. 78› melengkung‹hal. 81›.",
    );
  });

  test("looks a marker up among all sources, not only cited ones", () => {
    const uncited: readonly CiteSource[] = [{ id: "x7", number: 7, book: IPA, page: 5 }];
    expect(cites(segs("Lihat [7].", uncited)).map((c) => c.sourceId)).toEqual(["x7"]);
  });

  test("brackets that are not a plain number are left untouched", () => {
    expect(shown(segs("Pilihan [a] dan [12a] dan [ 1] ada [1]."))).toBe(
      "Pilihan [a] dan [12a] dan [ 1] ada‹hal. 78›.",
    );
  });

  test("while streaming a marker without its ] is not shown", () => {
    expect(shown(segs("Rel memuai [", SOURCES, true))).toBe("Rel memuai ");
    expect(shown(segs("Rel memuai [1", SOURCES, true))).toBe("Rel memuai ");
    expect(shown(segs("Rel memuai [1]", SOURCES, true))).toBe("Rel memuai‹hal. 78›");
    // Once done, a stray "[1" is just text.
    expect(shown(segs("Rel memuai [1", SOURCES, false))).toBe("Rel memuai [1");
  });

  test("before the sources are known every marker is hidden", () => {
    expect(shown(segs("Rel memuai [1]. Lalu [2", [], true))).toBe("Rel memuai. Lalu ");
  });

  test("with more than one book each pill names its book", () => {
    const two: readonly CiteSource[] = [
      { id: "a", number: 1, book: IPA, page: 78 },
      { id: "b", number: 2, book: "Matematika Kelas VII", page: 12 },
      { id: "c", number: 3, book: "Atlas Dunia", page: 4 },
    ];
    expect(shown(segs("A [1]. B [2]. C [3].", two))).toBe(
      "A‹IPA hal. 78›. B‹Matematika hal. 12›. C‹Atlas hal. 4›.",
    );
    // Only books the text cites count: one cited book keeps the short label.
    expect(shown(segs("A [1].", two))).toBe("A‹hal. 78›.");
  });

  test("a source without a page shows only the book name", () => {
    const s = segs("A [1].", [{ id: "a", number: 1, book: IPA }]);
    expect(cites(s)).toEqual([
      { kind: "cite", id: "cite-0", sourceId: "a", label: "IPA", ariaLabel: `Sumber: ${IPA}` },
    ]);
  });

  test("keeps the emphasis of each part", () => {
    const s = citationSegments(
      [{ text: "Rel " }, { text: "memuai", emphasis: true }, { text: " [1]." }],
      SOURCES,
      false,
    );
    expect(s.map((x) => (x.kind === "text" ? [x.text, x.emphasis] : x.label))).toEqual([
      ["Rel ", false],
      ["memuai", true],
      "hal. 78",
      [".", false],
    ]);
  });

  test("text without markers is one plain run", () => {
    expect(segs("Halo.")).toEqual([
      { kind: "text", text: "Halo.", emphasis: false, sentenceOf: [] },
    ]);
  });
});

describe("shortBookName", () => {
  test("uses the subject's short name, else the first word", () => {
    expect(shortBookName(IPA)).toBe("IPA");
    expect(shortBookName("Bahasa Indonesia untuk SMP")).toBe("B. Indonesia");
    expect(shortBookName("Atlas Dunia")).toBe("Atlas");
  });
});

describe("cite focus", () => {
  const s = segs("A [1]. B [2]. C [1].");
  const none = { id: "missing", sourceId: "missing" };
  const [c0 = none, c1 = none, c2 = none] = cites(s);

  test("a chip marks the first pill of its source", () => {
    expect(markedCite(s, { sourceId: "s1", citeId: null })).toBe("cite-0");
    expect(markedCite(s, { sourceId: null, citeId: null })).toBeNull();
  });

  test("pressing a pill marks that pill, even when its source is already open", () => {
    expect(pressCite(s, { sourceId: "s1", citeId: null }, c2)).toEqual({
      sourceId: "s1",
      citeId: "cite-2",
    });
  });

  test("pressing the marked pill closes the preview", () => {
    expect(pressCite(s, { sourceId: "s1", citeId: null }, c0)).toEqual({
      sourceId: null,
      citeId: null,
    });
    expect(pressCite(s, { sourceId: "s1", citeId: "cite-0" }, c1)).toEqual({
      sourceId: "s2",
      citeId: "cite-1",
    });
  });
});
