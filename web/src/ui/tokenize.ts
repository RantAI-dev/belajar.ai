/** One piece of a model answer, in reading order. Plain data: rendering lives in `RichText.tsx`. */
export type Segment =
  | { readonly kind: "text"; readonly text: string }
  /** `source` is the original text with its dollars, shown when KaTeX cannot draw `tex`. */
  | {
      readonly kind: "math";
      readonly tex: string;
      readonly display: boolean;
      readonly source: string;
    }
  | { readonly kind: "code"; readonly lang: string; readonly code: string }
  /** `complete` is false only while a stream has not yet closed the fence. */
  | { readonly kind: "mermaid"; readonly code: string; readonly complete: boolean };

export type TokenizeOptions = {
  /** The text is still arriving: an unclosed mermaid fence is a placeholder, not a code block. */
  readonly streaming?: boolean;
};

const FENCE_OPEN = /^ {0,3}(`{3,})[ \t]*([A-Za-z0-9_+-]*)[ \t]*(?:\n|$)/;
const WORD_CHAR = /[\p{L}\p{N}]/u;
const MATH_SIGN = /[\\^_=+<>{}*/]/;

/**
 * The currency guard. `$` is a dollar far more often than maths in a school answer ("Rp5.000 dan
 * $3", "$5 dan $3"), so a would-be `$…$` pair is money, and stays text, when the closing `$` is
 * followed by a number (it opens the next price), or when the content starts with a number and has
 * no maths sign in it (`$10, jadi $`).
 */
export const looksLikeMoney = (content: string, after: string): boolean =>
  /^\s*\d/.test(after) || (/^\s*\d/.test(content) && !MATH_SIGN.test(content));

/** Index of the next `$$` at or after `from` that is not escaped, or -1. */
const findDisplayClose = (s: string, from: number): number => {
  for (let j = from; j < s.length - 1; j++) {
    if (s[j] === "\\") j++;
    else if (s[j] === "$" && s[j + 1] === "$") return j;
  }
  return -1;
};

/** Index of the `$` that closes an inline formula opened at `open`, or -1 when it is text. */
const findInlineClose = (s: string, open: number): number => {
  const before = open > 0 ? (s[open - 1] ?? "") : "";
  if (WORD_CHAR.test(before)) return -1;
  for (let j = open + 1; j < s.length; j++) {
    const c = s[j];
    if (c === "\\") {
      j++;
      continue;
    }
    if (c === "\n" && /^\n[ \t]*\n/.test(s.slice(j))) return -1;
    if (c !== "$") continue;
    if (s[j + 1] === "$") return -1;
    const content = s.slice(open + 1, j);
    if (content.trim() === "" || looksLikeMoney(content, s.slice(j + 1))) return -1;
    return j;
  }
  return -1;
};

/** [start, end) of a formula or a fenced block in the text it was found in. */
type Range = readonly [number, number];

/** Maths inside prose (text outside code fences); `base` is where `s` starts in the answer. */
const tokenizeInline = (s: string, out: Segment[], base: number, ranges: Range[]): void => {
  let buf = "";
  const flush = () => {
    if (buf !== "") out.push({ kind: "text", text: buf });
    buf = "";
  };
  let i = 0;
  while (i < s.length) {
    const c = s[i] ?? "";
    if (c === "\\" && s[i + 1] === "$") {
      buf += "$";
      i += 2;
      continue;
    }
    if (c === "$" && s[i + 1] === "$") {
      const end = findDisplayClose(s, i + 2);
      const tex = end === -1 ? "" : s.slice(i + 2, end);
      if (tex.trim() !== "") {
        flush();
        out.push({ kind: "math", tex: tex.trim(), display: true, source: s.slice(i, end + 2) });
        ranges.push([base + i, base + end + 2]);
        i = end + 2;
      } else {
        buf += "$$";
        i += 2;
      }
      continue;
    }
    if (c === "$") {
      const end = findInlineClose(s, i);
      if (end !== -1) {
        flush();
        const tex = s.slice(i + 1, end).trim();
        out.push({ kind: "math", tex, display: false, source: s.slice(i, end + 1) });
        ranges.push([base + i, base + end + 1]);
        i = end + 1;
        continue;
      }
    }
    buf += c;
    i++;
  }
  flush();
};

/**
 * Splits a model answer into text, maths and fenced blocks. `[n]` citations are not handled here:
 * Tanya cuts them out first (features/tanya/citations.ts) and passes the text runs in between.
 *
 * Maths rule: `\$` is a literal dollar. `$$…$$` is a display formula and may span lines. `$…$` is
 * an inline formula only when it closes in the same paragraph (no blank line), the opening `$` does
 * not follow a letter or digit (`US$5`), the content is not blank, and `looksLikeMoney` says no.
 * Anything that does not close, including a formula still streaming in, stays plain text.
 */
export function tokenize(text: string, options: TokenizeOptions = {}): Segment[] {
  return scan(text, options.streaming ?? false, []);
}

/**
 * `at`, moved back to the start of the formula or fenced block it falls inside, so that cutting
 * the text there leaves every formula whole (`$ x = 1 $[1]` must not be cut before the last `$`).
 */
export function cutOutsideMaths(text: string, at: number): number {
  const ranges: Range[] = [];
  scan(text, false, ranges);
  return ranges.find(([start, end]) => start < at && at < end)?.[0] ?? at;
}

function scan(text: string, streaming: boolean, ranges: Range[]): Segment[] {
  const out: Segment[] = [];
  let textStart = 0;
  const fenceOpen = new RegExp(FENCE_OPEN.source, "gm");
  for (let m = fenceOpen.exec(text); m !== null; m = fenceOpen.exec(text)) {
    const fence = m[1] ?? "```";
    const lang = (m[2] ?? "").toLowerCase();
    const bodyStart = m.index + m[0].length;
    // A stream may stop inside the opening line ("```merm"): wait for the rest of it.
    if (!m[0].endsWith("\n") && streaming) break;
    tokenizeInline(text.slice(textStart, m.index), out, textStart, ranges);
    const rest = text.slice(bodyStart);
    const close = new RegExp(`^ {0,3}${fence}\`*[ \\t]*$`, "m").exec(rest);
    const code = close === null ? rest : rest.slice(0, close.index).replace(/\n$/, "");
    if (lang === "mermaid" && (close !== null || streaming)) {
      out.push({ kind: "mermaid", code, complete: close !== null });
    } else {
      out.push({ kind: "code", lang, code });
    }
    if (close === null) {
      ranges.push([m.index, text.length]);
      textStart = text.length;
      break;
    }
    ranges.push([m.index, bodyStart + close.index + close[0].length]);
    textStart = bodyStart + close.index + close[0].length;
    fenceOpen.lastIndex = textStart;
  }
  tokenizeInline(text.slice(textStart), out, textStart, ranges);
  return out;
}
