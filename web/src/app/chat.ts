import { useCallback, useEffect, useReducer, useRef, useState } from "react";
import { api } from "../contracts/client";
import { generalMessage } from "../contracts/messages";
import { postStream } from "../contracts/sse";
import {
  chatReducer,
  emptyChat,
  messageFromDto,
  parseDelta,
  parseMessage,
  type TanyaMessage,
} from "../features/tanya";
import { useHome } from "./home";
import { useAbortOnLeave } from "./leave";
import { useAuth } from "./session";

export type LoadState = "loading" | "ready" | { readonly error: string };

export type Chat = {
  readonly messages: readonly TanyaMessage[];
  readonly busy: boolean;
  readonly load: LoadState;
  readonly reload: () => void;
  /** Sends a question; resolves when its answer is stored or has failed. */
  readonly send: (question: string) => Promise<void>;
};

const ENDED_EARLY = "Jawabanku terputus. Coba tanya lagi, ya.";

/**
 * One conversation's messages and the streaming answer (POST .../messages). The stream is
 * aborted when the student leaves the page; the header stats are refreshed after every answer.
 */
export function useChat(conversationId: string | null, onAnswered?: () => void): Chat {
  const grade = useAuth().student?.grade ?? 0;
  const gradeLabel = `Kelas ${grade}`;
  const { refresh } = useHome();
  const controller = useAbortOnLeave();
  const [state, dispatch] = useReducer(chatReducer, emptyChat);
  const [load, setLoad] = useState<LoadState>("loading");
  const [attempt, setAttempt] = useState(0);
  const busy = useRef(false);
  const answeredListener = useRef(onAnswered);
  answeredListener.current = onAnswered;

  useEffect(() => {
    if (conversationId === null) return;
    // `attempt` re-runs the load after "Coba lagi".
    void attempt;
    const ctrl = new AbortController();
    setLoad("loading");
    void api
      .get("/v1/conversations/{id}/messages", {
        pathParams: { id: conversationId },
        query: { limit: 100 },
        signal: ctrl.signal,
      })
      .then((r) => {
        if (ctrl.signal.aborted) return;
        if (!r.ok) {
          setLoad({ error: generalMessage(r.error) });
          return;
        }
        dispatch({
          type: "loaded",
          messages: r.value.data.map((m) => messageFromDto(m, gradeLabel)),
        });
        setLoad("ready");
      });
    return () => ctrl.abort();
  }, [conversationId, gradeLabel, attempt]);

  const send = useCallback(
    async (question: string) => {
      if (conversationId === null || busy.current) return;
      busy.current = true;
      dispatch({ type: "asked", question });
      // An object, so the stream callback's write is visible after the await.
      const seen = { answered: false };
      const r = await postStream(
        "/v1/conversations/{id}/messages",
        { content: question },
        {
          pathParams: { id: conversationId },
          signal: controller().signal,
          onEvent: (e) => {
            if (e.event === "delta") {
              const text = parseDelta(e.data);
              if (text !== null) dispatch({ type: "delta", text });
              return;
            }
            const m = parseMessage(e.data);
            if (m === null) return;
            if (e.event === "message")
              dispatch({ type: "stored", question: messageFromDto(m, gradeLabel) });
            if (e.event === "done") {
              seen.answered = true;
              dispatch({ type: "answered", answer: messageFromDto(m, gradeLabel) });
            }
          },
        },
      );
      busy.current = false;
      if (!r.ok) dispatch({ type: "failed", text: generalMessage(r.error) });
      else if (r.value === "ended" && !seen.answered)
        dispatch({ type: "failed", text: ENDED_EARLY });
      if (seen.answered) {
        void refresh();
        answeredListener.current?.();
      }
    },
    [conversationId, gradeLabel, controller, refresh],
  );

  return {
    messages: state.messages,
    busy: state.busy,
    load,
    reload: () => setAttempt((n) => n + 1),
    send,
  };
}
