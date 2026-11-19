import { ChatBubble } from "../../ui";
import { AnswerBubble } from "./AnswerBubble";
import type { TanyaMessage } from "./types";

/** The bubbles of one conversation, in order. Shared by Tanya and the Belajar session. */
export function ChatThread({ messages }: { readonly messages: readonly TanyaMessage[] }) {
  return (
    <>
      {messages.map((m) => {
        if (m.kind === "question")
          return (
            <ChatBubble key={m.id} from="student">
              {m.text}
            </ChatBubble>
          );
        if (m.kind === "answer") return <AnswerBubble key={m.id} answer={m} />;
        if (m.kind === "error")
          return (
            <ChatBubble key={m.id} from="mascot" mood="hmm" variant="ungrounded">
              <span role="alert" className="text-[17px] font-extrabold">
                {m.text}
              </span>
            </ChatBubble>
          );
        return (
          <ChatBubble key={m.id} from="mascot" mood="hmm" variant="ungrounded">
            <span className="text-[17px] font-extrabold">
              Hmm, itu tidak ada di buku pelajaranmu.
            </span>
            <span className="text-[16px] font-semibold leading-relaxed text-ink-muted">
              Aku cuma menjawab dari buku {m.gradeLabel}, supaya jawabanku pasti benar. Coba tanya
              soal pelajaran, ya!
            </span>
          </ChatBubble>
        );
      })}
    </>
  );
}
