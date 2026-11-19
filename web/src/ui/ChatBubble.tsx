import type { ReactNode } from "react";
import { cx } from "./cx";
import { Mascot, type MascotMood } from "./Mascot";

export type ChatBubbleProps =
  | { readonly from: "student"; readonly children: ReactNode }
  | {
      readonly from: "mascot";
      readonly mood?: MascotMood;
      /** `ungrounded`: a dashed bubble for answers that are not in the student's book. */
      readonly variant?: "grounded" | "ungrounded";
      /** The answer is still arriving: announce it politely and show typing dots while empty. */
      readonly streaming?: boolean;
      readonly children?: ReactNode;
    };

function TypingDots() {
  return (
    <span className="inline-flex items-center gap-1.5 py-1" aria-hidden="true">
      {[0, 1, 2].map((i) => (
        <span
          key={i}
          className="typing-dot size-2.5 rounded-full bg-brand-sky"
          style={{ animationDelay: `${i * 0.18}s` }}
        />
      ))}
    </span>
  );
}

export function ChatBubble(props: ChatBubbleProps) {
  if (props.from === "student") {
    return (
      <div className="max-w-[85%] self-end rounded-[22px_22px_6px_22px] bg-primary px-[18px] py-3.5 text-[18px] font-bold leading-snug text-white sm:max-w-[78%]">
        <span className="sr-only">Kamu: </span>
        {props.children}
      </div>
    );
  }
  const { mood = "happy", variant = "grounded", streaming = false, children } = props;
  const empty = children === undefined || children === null || children === "";
  return (
    <div className="flex items-start gap-3">
      <Mascot mood={mood} size={48} className="max-sm:size-10" />
      <div
        aria-live={streaming ? "polite" : undefined}
        aria-busy={streaming || undefined}
        className={cx(
          "flex min-w-0 flex-1 flex-col gap-3.5 rounded-[6px_22px_22px_22px] bg-white px-4 py-4 sm:px-5",
          variant === "ungrounded"
            ? "border-2 border-dashed border-[#c4d6e8]"
            : "border-2 border-line",
        )}
      >
        {streaming && empty ? (
          <>
            <TypingDots />
            <span className="sr-only">Sebentar, aku cari di bukumu…</span>
          </>
        ) : (
          children
        )}
      </div>
    </div>
  );
}
