import type { TanyaMessage } from "./types";

/** A conversation as it streams: stored messages plus, while asking, one answer arriving. */
export type ChatState = {
  readonly messages: readonly TanyaMessage[];
  /** True from sending until the answer is stored or failed: sending is paused. */
  readonly busy: boolean;
};

export type ChatAction =
  /** GET .../messages answered. */
  | { readonly type: "loaded"; readonly messages: readonly TanyaMessage[] }
  /** The student sent a question: show it at once, and an empty answer that is arriving. */
  | { readonly type: "asked"; readonly question: string }
  /** `message`: the server stored the question under its real id. */
  | { readonly type: "stored"; readonly question: TanyaMessage }
  /** `delta`: the next piece of the answer. */
  | { readonly type: "delta"; readonly text: string }
  /** `done`: the stored answer replaces the one that streamed. */
  | { readonly type: "answered"; readonly answer: TanyaMessage }
  /** `error`, a dropped connection, or a stream that ended without `done`. */
  | { readonly type: "failed"; readonly text: string };

export const PENDING_QUESTION = "pending-question";
export const PENDING_ANSWER = "pending-answer";

export const emptyChat: ChatState = { messages: [], busy: false };

const replace = (
  messages: readonly TanyaMessage[],
  id: string,
  next: TanyaMessage,
): readonly TanyaMessage[] => messages.map((m) => (m.id === id ? next : m));

export function chatReducer(s: ChatState, a: ChatAction): ChatState {
  switch (a.type) {
    case "loaded":
      // A question sent before the list arrived stays at the end.
      return {
        ...s,
        messages: [...a.messages, ...s.messages.filter((m) => m.id.startsWith("pending-"))],
      };
    case "asked":
      if (s.busy) return s;
      return {
        busy: true,
        messages: [
          ...s.messages,
          { id: PENDING_QUESTION, kind: "question", text: a.question },
          { id: PENDING_ANSWER, kind: "answer", status: "streaming", parts: [], sources: [] },
        ],
      };
    case "stored":
      return { ...s, messages: replace(s.messages, PENDING_QUESTION, a.question) };
    case "delta":
      return {
        ...s,
        messages: s.messages.map((m) =>
          m.id === PENDING_ANSWER && m.kind === "answer"
            ? { ...m, parts: [{ text: (m.parts[0]?.text ?? "") + a.text }] }
            : m,
        ),
      };
    case "answered":
      return { busy: false, messages: replace(s.messages, PENDING_ANSWER, a.answer) };
    case "failed": {
      const error: TanyaMessage = { id: `error-${s.messages.length}`, kind: "error", text: a.text };
      // A question the server never stored keeps its bubble, so the student sees what failed.
      const kept = s.messages.map((m) =>
        m.id === PENDING_QUESTION ? { ...m, id: `unsent-${s.messages.length}` } : m,
      );
      return {
        busy: false,
        messages: kept.some((m) => m.id === PENDING_ANSWER)
          ? replace(kept, PENDING_ANSWER, error)
          : [...kept, error],
      };
    }
  }
}

/** The last question the student asked, for "Buat 3 soal tentang ini". */
export const lastQuestion = (messages: readonly TanyaMessage[]): string | null => {
  for (let i = messages.length - 1; i >= 0; i--) {
    const m = messages[i];
    if (m?.kind === "question") return m.text;
  }
  return null;
};
