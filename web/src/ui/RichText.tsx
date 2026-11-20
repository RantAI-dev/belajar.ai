import { useEffect, useId, useLayoutEffect, useMemo, useRef, useState } from "react";
import { renderDiagram } from "./diagram";
import { isMathReady, loadMath, renderMath } from "./math";
import { useThemeVersion } from "./themeWatch";
import { type Segment, tokenize } from "./tokenize";

// Every piece is phrasing content (spans), so RichText may sit inside a <p>, <h1> or <button>.

type MarkupProps = {
  readonly html: string;
  readonly className?: string | undefined;
  readonly role?: string;
  readonly "aria-label"?: string;
  readonly "data-math"?: string;
};

/**
 * Puts markup made by KaTeX (`trust: false`, links and HTML commands refused) or by mermaid
 * (`securityLevel: "strict"`, sanitised SVG) into the page. Model text never reaches it unescaped.
 */
function Markup({ html, ...rest }: MarkupProps) {
  const ref = useRef<HTMLSpanElement>(null);
  useLayoutEffect(() => {
    if (ref.current !== null) ref.current.innerHTML = html;
  }, [html]);
  return <span ref={ref} {...rest} />;
}

function SourceCode({ children }: { readonly children: string }) {
  return (
    <code className="rounded-md bg-page px-1.5 py-0.5 font-mono text-[0.9em] text-ink [overflow-wrap:anywhere]">
      {children}
    </code>
  );
}

function CodeBlock({ code }: { readonly code: string }) {
  return (
    <code className="my-2 block overflow-x-auto whitespace-pre rounded-[12px] border-2 border-line bg-page p-3 font-mono text-[15px] text-ink">
      {code}
    </code>
  );
}

function MathSegment({ seg }: { readonly seg: Extract<Segment, { kind: "math" }> }) {
  const result = renderMath(seg.tex, seg.display);
  if (result.kind === "pending") return <span>{seg.source}</span>;
  if (result.kind === "invalid") return <SourceCode>{seg.source}</SourceCode>;
  return (
    <Markup
      html={result.html}
      data-math={seg.display ? "display" : "inline"}
      // Long equations scroll inside their own box instead of widening the page.
      className={
        seg.display
          ? "my-2 block max-w-full overflow-x-auto overflow-y-hidden py-1 [&_.katex-display]:m-0"
          : undefined
      }
    />
  );
}

function Placeholder() {
  return (
    <span className="my-2 block rounded-[12px] border-2 border-dashed border-line-sky px-3 py-4 text-[15px] font-bold text-ink-muted">
      Sebentar, aku gambar diagramnya…
    </span>
  );
}

type DiagramState =
  | { readonly kind: "loading" }
  | { readonly kind: "svg"; readonly svg: string }
  | { readonly kind: "invalid" };

/** A ```mermaid block, drawn by the lazily loaded mermaid and redrawn when the theme changes. */
export function MermaidDiagram({ code }: { readonly code: string }) {
  const domId = `diagram-${useId().replace(/[^A-Za-z0-9]/g, "")}`;
  const theme = useThemeVersion();
  const [state, setState] = useState<DiagramState>({ kind: "loading" });
  useEffect(() => {
    let live = true;
    // Only a trigger: a new theme means drawing again with the colours read now.
    void theme;
    renderDiagram(domId, code).then((svg) => {
      if (live) setState(svg === null ? { kind: "invalid" } : { kind: "svg", svg });
    });
    return () => {
      live = false;
    };
  }, [domId, code, theme]);
  if (state.kind === "loading") return <Placeholder />;
  if (state.kind === "invalid") return <CodeBlock code={code} />;
  return (
    <Markup
      html={state.svg}
      role="img"
      aria-label="Diagram"
      className="my-2 block max-w-full overflow-x-auto [&_svg]:h-auto [&_svg]:max-w-full"
    />
  );
}

function Citation({ n }: { readonly n: number }) {
  return (
    <span className="text-[0.85em] font-extrabold text-primary">
      <span className="sr-only">sumber </span>[{n}]
    </span>
  );
}

export type RichTextProps = {
  readonly text: string;
  /** The text is still arriving. */
  readonly streaming?: boolean;
};

/** A model answer with maths, `[n]` citations and diagrams. Use it for every answer the student reads. */
export function RichText({ text, streaming = false }: RichTextProps) {
  // Segments are an ordered split of one text; position plus kind is a stable key.
  const segments = useMemo(
    () => tokenize(text, { streaming }).map((seg, i) => ({ seg, key: `${i}:${seg.kind}` })),
    [text, streaming],
  );
  const hasMath = segments.some((s) => s.seg.kind === "math");
  const [, setMathReady] = useState(isMathReady);
  useEffect(() => {
    if (!hasMath || isMathReady()) return;
    let live = true;
    loadMath().then((ok) => {
      if (live && ok) setMathReady(true);
    });
    return () => {
      live = false;
    };
  }, [hasMath]);
  return segments.map(({ seg, key }) => <SegmentView key={key} seg={seg} />);
}

function SegmentView({ seg }: { readonly seg: Segment }) {
  switch (seg.kind) {
    case "text":
      return <span>{seg.text}</span>;
    case "citation":
      return <Citation n={seg.n} />;
    case "math":
      return <MathSegment seg={seg} />;
    case "code":
      return <CodeBlock code={seg.code} />;
    case "mermaid":
      return seg.complete ? <MermaidDiagram code={seg.code} /> : <Placeholder />;
  }
}
