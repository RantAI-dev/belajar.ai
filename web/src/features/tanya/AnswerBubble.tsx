import { useMemo, useState } from "react";
import { BulbIcon, ChatBubble, CITE, CitationPill, cx, PagePreview, SourceChip } from "../../ui";
import {
  type CitationSegment,
  type CiteFocus,
  citationSegments,
  keepCitesWithWords,
  markedCite,
  pressCite,
} from "./citations";
import type { TanyaMessage } from "./types";

type Answer = Extract<TanyaMessage, { kind: "answer" }>;

export function AnswerBubble({ answer }: { readonly answer: Answer }) {
  const [focus, setFocus] = useState<CiteFocus>({
    sourceId: answer.sources[0]?.id ?? null,
    citeId: null,
  });
  const streaming = answer.status === "streaming";
  const allSources = answer.allSources ?? answer.sources;
  const active = allSources.find((s) => s.id === focus.sourceId);
  const hasText = answer.parts.some((p) => p.text.length > 0);
  const segments = useMemo(
    () => citationSegments(answer.parts, allSources, streaming),
    [answer.parts, allSources, streaming],
  );
  const marked = markedCite(segments, focus);
  const runs = useMemo(() => keepCitesWithWords(segments), [segments]);
  // Segments are an ordered split of one answer; position plus content is a stable key.
  const render = (s: CitationSegment, key: string) => {
    if (s.kind === "cite")
      return (
        <CitationPill
          key={key}
          label={s.label}
          ariaLabel={s.ariaLabel}
          pressed={s.sourceId === focus.sourceId}
          onPress={() => setFocus((f) => pressCite(segments, f, s))}
        />
      );
    const isMarked = marked !== null && s.sentenceOf.includes(marked);
    const style = isMarked ? { background: CITE.marker } : undefined;
    const className = cx(isMarked && "rounded-[4px] [box-decoration-break:clone]");
    return s.emphasis ? (
      <strong key={key} className={cx("text-primary", className)} style={style}>
        {s.text}
      </strong>
    ) : (
      <span key={key} className={className} style={style}>
        {s.text}
      </span>
    );
  };
  const keyOf = (s: CitationSegment, i: number) => `${i}:${s.kind === "text" ? s.text : s.id}`;
  return (
    <ChatBubble from="mascot" mood={streaming ? "thinking" : "happy"} streaming={streaming}>
      {hasText ? (
        <>
          <p className="m-0 text-[18px] font-semibold leading-[1.65]">
            {runs.map((r, i) =>
              r.kind === "nowrap" ? (
                <span
                  key={r.items.flatMap((s) => (s.kind === "cite" ? [s.id] : [])).join()}
                  className="whitespace-nowrap"
                >
                  {r.items.map((s, j) => render(s, keyOf(s, j)))}
                </span>
              ) : (
                render(r, keyOf(r, i))
              ),
            )}
          </p>
          {!streaming && answer.example !== undefined && (
            <div className="flex items-center gap-3 rounded-[16px] bg-warm px-3.5 py-3">
              <BulbIcon className="shrink-0" />
              <span className="text-[16px] font-bold text-warm-ink">{answer.example}</span>
            </div>
          )}
          {!streaming && answer.sources.length > 0 && (
            <div className="flex flex-col gap-2">
              <span className="text-[14px] font-extrabold text-ink-muted">
                Aku ambil dari buku ini:
              </span>
              <div className="flex flex-wrap gap-2">
                {answer.sources.map((s) => (
                  <SourceChip
                    key={s.id}
                    label={s.label}
                    pressed={s.id === focus.sourceId}
                    onPressedChange={(on) => setFocus({ sourceId: on ? s.id : null, citeId: null })}
                  />
                ))}
              </div>
              {active !== undefined && (
                <PagePreview
                  title={active.title}
                  {...(active.fullPageHref !== undefined
                    ? { fullPageHref: active.fullPageHref }
                    : {})}
                  {...(active.imageUrl !== undefined ? { imageUrl: active.imageUrl } : {})}
                />
              )}
            </div>
          )}
        </>
      ) : null}
    </ChatBubble>
  );
}
