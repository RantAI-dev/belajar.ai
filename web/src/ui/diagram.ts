import type { MermaidConfig } from "mermaid";

type Mermaid = typeof import("mermaid").default;

let loading: Promise<Mermaid> | null = null;

/** Mermaid is large: it is fetched only when an answer has a ```mermaid block. */
const loadMermaid = (): Promise<Mermaid> => {
  loading ??= import("mermaid").then(
    (m) => m.default,
    (e: unknown) => {
      loading = null;
      return Promise.reject(e);
    },
  );
  return loading;
};

/** Reads a theme token (from theme.css, or the dark-mode override) as it is right now. */
export type ReadToken = (name: string) => string;

export const readCssToken: ReadToken = (name) =>
  typeof document === "undefined"
    ? ""
    : getComputedStyle(document.documentElement).getPropertyValue(name).trim();

// Mermaid colour slot <- theme token. Mermaid computes shades from these, so they must be resolved
// colour values (it cannot read `var(...)`), which is why they are read at render time.
const SLOTS: ReadonlyArray<readonly [string, string]> = [
  ["primaryColor", "--color-primary-soft"],
  ["primaryBorderColor", "--color-primary"],
  ["primaryTextColor", "--color-ink"],
  ["secondaryColor", "--color-warm"],
  ["secondaryBorderColor", "--color-warm-line"],
  ["secondaryTextColor", "--color-warm-ink"],
  ["tertiaryColor", "--color-success-soft"],
  ["tertiaryBorderColor", "--color-success"],
  ["tertiaryTextColor", "--color-ink"],
  ["mainBkg", "--color-primary-soft"],
  ["nodeBorder", "--color-primary"],
  ["lineColor", "--color-ink-muted"],
  ["textColor", "--color-ink"],
  ["titleColor", "--color-ink"],
  ["edgeLabelBackground", "--color-page"],
  ["clusterBkg", "--color-page"],
  ["clusterBorder", "--color-line-strong"],
  ["background", "--color-page"],
  ["fontFamily", "--font-sans"],
];

/** Mermaid settings for untrusted model output, coloured from the current theme tokens. */
export function diagramConfig(read: ReadToken, themed = true): MermaidConfig {
  const themeVariables: Record<string, string> = {};
  if (themed) {
    for (const [slot, token] of SLOTS) {
      const value = read(token);
      if (value !== "") themeVariables[slot] = value;
    }
  }
  return {
    startOnLoad: false,
    // Model output is untrusted: no scripts, no click handlers, labels as SVG text, not HTML.
    securityLevel: "strict",
    htmlLabels: false,
    suppressErrorRendering: true,
    theme: "base",
    themeVariables,
  };
}

/** True when mermaid can parse the diagram; an invalid one is shown as code. */
export async function isValidDiagram(code: string): Promise<boolean> {
  try {
    const mermaid = await loadMermaid();
    return (await mermaid.parse(code, { suppressErrors: true })) !== false;
  } catch {
    return false;
  }
}

/** The diagram's SVG, or null when it is invalid or mermaid could not be loaded. */
export async function renderDiagram(
  domId: string,
  code: string,
  read: ReadToken = readCssToken,
): Promise<string | null> {
  if (!(await isValidDiagram(code))) return null;
  const mermaid = await loadMermaid();
  // A theme colour mermaid cannot parse must not cost the diagram: retry in its own colours.
  for (const themed of [true, false]) {
    try {
      mermaid.initialize(diagramConfig(read, themed));
      const { svg } = await mermaid.render(domId, code);
      return svg;
    } catch {
      // try the next config
    }
  }
  return null;
}
