import { cutOutsideMaths, subjectById } from "../../ui";
import { subjectOfBook } from "./practiceLink";
import type { AnswerPart, BookSource } from "./types";

export type CiteSource = Pick<BookSource, "id" | "number" | "book" | "page">;

export type CitationSegment =
  | {
      readonly kind: "text";
      readonly text: string;
      readonly emphasis: boolean;
      /** Ids of the cites whose sentence this text belongs to. */
      readonly sentenceOf: readonly string[];
    }
  | {
      readonly kind: "cite";
      /** Unique within one answer, e.g. "cite-0". */
      readonly id: string;
      readonly sourceId: string;
      readonly label: string;
      readonly ariaLabel: string;
    };

type Cite = Extract<CitationSegment, { kind: "cite" }>;

/** A run of adjacent `[n]` markers, with the whitespace before them. */
type Group = {
  readonly start: number;
  readonly end: number;
  readonly sentenceStart: number;
  readonly sources: readonly CiteSource[];
};

// Leading whitespace is part of the match: the pill sits right after its word.
const MARKER = /\s*\[(\d+)\]/g;
const PARTIAL_MARKER = /\[\d*$/;
const TERMINATOR = /[.!?]/;

/** "IPA" for "IPA Buku Siswa Kelas VII"; else the title's first word. */
export const shortBookName = (book: string): string => {
  const subject = subjectById(subjectOfBook(book) ?? "");
  return subject?.shortName ?? book.trim().split(/\s+/)[0] ?? book;
};

/** Where the sentence that ends at `end` starts: after the last `.`, `!`, `?` or line break. */
const sentenceStart = (text: string, end: number): number => {
  let i = end - 1;
  // A terminator right before the marker ends this very sentence: "panas.[1]".
  while (i >= 0 && (/\s/.test(text.charAt(i)) || TERMINATOR.test(text.charAt(i)))) i--;
  for (; i >= 0; i--) {
    const c = text.charAt(i);
    if (c === "\n") break;
    if (TERMINATOR.test(c) && /\s/.test(text.charAt(i + 1))) break;
  }
  let start = i + 1;
  while (start < end && /\s/.test(text.charAt(start))) start++;
  return start;
};

const findGroups = (text: string, sources: readonly CiteSource[]): readonly Group[] => {
  const groups: Group[] = [];
  for (const m of text.matchAll(MARKER)) {
    const start = m.index;
    const end = start + m[0].length;
    const found = sources.find((s) => s.number === Number(m[1]));
    const last = groups.at(-1);
    const resolved = found === undefined ? [] : [found];
    if (last !== undefined && last.end === start) {
      groups[groups.length - 1] = { ...last, end, sources: [...last.sources, ...resolved] };
    } else {
      groups.push({ start, end, sentenceStart: sentenceStart(text, start), sources: resolved });
    }
  }
  return groups;
};

const pageKey = (s: CiteSource): string => `${s.book}\u0000${s.page ?? ""}`;

/**
 * An answer's text as plain runs and "book + page" pills, in order.
 * A `[n]` is looked up by number among all `sources`; one without a source disappears, and so
 * does every marker until the sources are known. Adjacent markers to the same page make one pill.
 * While `streaming`, a marker whose `]` has not arrived yet is held back.
 */
export function citationSegments(
  parts: readonly AnswerPart[],
  sources: readonly CiteSource[],
  streaming: boolean,
): readonly CitationSegment[] {
  let text = parts.map((p) => p.text).join("");
  if (streaming) text = text.replace(PARTIAL_MARKER, "");
  const groups = findGroups(text, sources);

  const books = new Set(groups.flatMap((g) => g.sources.map((s) => s.book)));
  const named = books.size > 1;
  let next = 0;
  const citesOf = groups.map((g): readonly Cite[] => {
    const seen = new Set<string>();
    return g.sources.flatMap((s) => {
      if (seen.has(pageKey(s))) return [];
      seen.add(pageKey(s));
      const short = shortBookName(s.book);
      const label =
        s.page === undefined ? short : named ? `${short} hal. ${s.page}` : `hal. ${s.page}`;
      const ariaLabel =
        s.page === undefined ? `Sumber: ${s.book}` : `Sumber: ${s.book}, halaman ${s.page}`;
      return [{ kind: "cite", id: `cite-${next++}`, sourceId: s.id, label, ariaLabel }];
    });
  });

  // Every place a run of text may change: part edges, marker edges and sentence starts.
  const cuts = new Set<number>([0, text.length]);
  let offset = 0;
  const partEnds = parts.map((p) => {
    offset += p.text.length;
    cuts.add(Math.min(offset, text.length));
    return offset;
  });
  for (const g of groups) {
    cuts.add(g.start);
    cuts.add(g.end);
    cuts.add(g.sentenceStart);
  }
  const points = [...cuts].sort((a, b) => a - b);

  const out: CitationSegment[] = [];
  points.forEach((a, i) => {
    const b = points[i + 1];
    if (b === undefined || b <= a) return;
    const gi = groups.findIndex((g) => g.start <= a && b <= g.end);
    if (gi >= 0) {
      if (groups[gi]?.start === a) out.push(...(citesOf[gi] ?? []));
      return;
    }
    const part = parts[partEnds.findIndex((e) => a < e)];
    const emphasis = part?.emphasis === true;
    const sentenceOf = groups.flatMap((g, k) =>
      g.sentenceStart <= a && b <= g.start ? (citesOf[k] ?? []).map((c) => c.id) : [],
    );
    const prev = out.at(-1);
    if (
      prev?.kind === "text" &&
      prev.emphasis === emphasis &&
      prev.sentenceOf.join() === sentenceOf.join()
    ) {
      out[out.length - 1] = { ...prev, text: prev.text + text.slice(a, b) };
    } else {
      out.push({ kind: "text", text: text.slice(a, b), emphasis, sentenceOf });
    }
  });
  return out;
}

type Text = Extract<CitationSegment, { kind: "text" }>;

/** Pills with the word before them and the punctuation after: one line never splits them. */
export type CitationRun =
  | CitationSegment
  | { readonly kind: "nowrap"; readonly items: readonly CitationSegment[] };

const splitText = (t: Text, at: number): readonly [Text, Text] => [
  { ...t, text: t.text.slice(0, at) },
  { ...t, text: t.text.slice(at) },
];

export function keepCitesWithWords(segments: readonly CitationSegment[]): readonly CitationRun[] {
  const out: CitationRun[] = [];
  let i = 0;
  while (i < segments.length) {
    const seg = segments[i];
    if (seg === undefined) break;
    if (seg.kind === "text") {
      out.push(seg);
      i++;
      continue;
    }
    const glued: CitationSegment[] = [];
    const prev = out.at(-1);
    if (prev?.kind === "text") {
      // The word before the pill, or the whole formula when the pill follows one.
      const [head, tail] = splitText(prev, cutOutsideMaths(prev.text, prev.text.search(/\S*$/)));
      out.pop();
      if (head.text !== "") out.push(head);
      if (tail.text !== "") glued.push(tail);
    }
    for (let next = segments[i]; next?.kind === "cite"; next = segments[++i]) glued.push(next);
    const after = segments[i];
    if (after?.kind === "text") {
      const [punct, rest] = splitText(after, after.text.match(/^[.,!?;:)]*/)?.[0].length ?? 0);
      if (punct.text !== "") glued.push(punct);
      if (rest.text !== "") out.push({ kind: "nowrap", items: glued }, rest);
      else out.push({ kind: "nowrap", items: glued });
      i++;
    } else {
      out.push({ kind: "nowrap", items: glued });
    }
  }
  return out;
}

/** Which source preview is open, and which pill opened it (null when a chip did). */
export type CiteFocus = { readonly sourceId: string | null; readonly citeId: string | null };

/** The pill whose sentence is marked: the one pressed, else the first pill of the open source. */
export const markedCite = (
  segments: readonly CitationSegment[],
  focus: CiteFocus,
): string | null => {
  if (focus.sourceId === null) return null;
  const cites = segments.filter((s): s is Cite => s.kind === "cite");
  if (focus.citeId !== null && cites.some((c) => c.id === focus.citeId)) return focus.citeId;
  return cites.find((c) => c.sourceId === focus.sourceId)?.id ?? null;
};

/** Pressing a pill opens its source and marks its sentence; pressing the marked pill closes it. */
export const pressCite = (
  segments: readonly CitationSegment[],
  focus: CiteFocus,
  cite: { readonly id: string; readonly sourceId: string },
): CiteFocus =>
  markedCite(segments, focus) === cite.id
    ? { sourceId: null, citeId: null }
    : { sourceId: cite.sourceId, citeId: cite.id };
