type Katex = typeof import("katex").default;

let katex: Katex | null = null;
let loading: Promise<boolean> | null = null;

/** True once KaTeX is loaded; from then on `renderMath` answers synchronously. */
export const isMathReady = (): boolean => katex !== null;

/** Loads KaTeX (and its CSS and fonts) once, on the first answer that has maths in it. */
export function loadMath(): Promise<boolean> {
  // Its own chunk, shared with mermaid's import of KaTeX; Vite bundles the CSS's fonts locally.
  loading ??= Promise.all([import("katex"), import("katex/dist/katex.min.css")]).then(
    ([m]) => {
      katex = m.default;
      return true;
    },
    () => {
      // A failed chunk load (offline school PC) may succeed on the next answer.
      loading = null;
      return false;
    },
  );
  return loading;
}

export type MathResult =
  | { readonly kind: "pending" }
  | { readonly kind: "html"; readonly html: string }
  | { readonly kind: "invalid" };

// `trust: false` already refuses these, but KaTeX then draws the command name in red; we show the
// source instead, like any other formula it cannot draw.
const LINKS_OR_HTML = /\\(href|url|includegraphics|html[A-Za-z]+)(?![A-Za-z])/;

/** KaTeX HTML for one formula, or `invalid` when the student should see its source instead. */
export function renderMath(tex: string, display: boolean): MathResult {
  if (katex === null) return { kind: "pending" };
  if (LINKS_OR_HTML.test(tex)) return { kind: "invalid" };
  try {
    const html = katex.renderToString(tex, {
      displayMode: display,
      // Errors throw so we can show readable source, never KaTeX's red error text.
      throwOnError: true,
      trust: false,
      strict: "ignore",
      maxExpand: 500,
      maxSize: 20,
    });
    return { kind: "html", html };
  } catch {
    return { kind: "invalid" };
  }
}
