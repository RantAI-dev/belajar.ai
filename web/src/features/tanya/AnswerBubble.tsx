import { useState } from "react";
import { BulbIcon, ChatBubble, PagePreview, RichText, SourceChip } from "../../ui";
import type { TanyaMessage } from "./types";

type Answer = Extract<TanyaMessage, { kind: "answer" }>;

export function AnswerBubble({ answer }: { readonly answer: Answer }) {
  const [openSource, setOpenSource] = useState<string | null>(answer.sources[0]?.id ?? null);
  const streaming = answer.status === "streaming";
  const active = answer.sources.find((s) => s.id === openSource);
  const hasText = answer.parts.some((p) => p.text.length > 0);
  // Parts are an ordered split of one answer; position plus text is a stable key.
  const parts = answer.parts.map((p, i) => ({ ...p, key: `${i}:${p.text}` }));
  return (
    <ChatBubble from="mascot" mood={streaming ? "thinking" : "happy"} streaming={streaming}>
      {hasText ? (
        <>
          <p className="m-0 text-[18px] font-semibold leading-[1.65]">
            {parts.map((p) =>
              p.emphasis === true ? (
                <strong key={p.key} className="text-primary">
                  <RichText text={p.text} streaming={streaming} />
                </strong>
              ) : (
                <span key={p.key}>
                  <RichText text={p.text} streaming={streaming} />
                </span>
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
                    pressed={s.id === openSource}
                    onPressedChange={(on) => setOpenSource(on ? s.id : null)}
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
