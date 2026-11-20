import { useCallback, useEffect, useRef, useState } from "react";
import { useLocation, useNavigate, useParams } from "react-router";
import { api } from "../../contracts/client";
import { generalMessage } from "../../contracts/messages";
import {
  type ConversationDto,
  type ConversationSummary,
  FOLLOW_UP_QUESTIONS,
  practiceTopicOf,
  type SubjectFilter,
  summaryFromDto,
  TANYA_EMPTY,
  TanyaScreen,
} from "../../features/tanya";
import { isSubjectId } from "../../ui";
import { useChat } from "../chat";
import { latihanHref } from "./latihanHref";
import { NotFound } from "./NotFound";

/** Location state that carries a question asked elsewhere (Beranda, a new Tanya) to send here. */
type AskState = { readonly ask: string };

const askOf = (state: unknown): string | null =>
  typeof state === "object" &&
  state !== null &&
  "ask" in state &&
  typeof (state as AskState).ask === "string"
    ? (state as AskState).ask
    : null;

/** Starts a conversation and opens it with the question waiting to be sent. */
export function useStartConversation(): (
  question: string,
  subjectId: string | null,
) => Promise<string | null> {
  const navigate = useNavigate();
  return useCallback(
    async (question, subjectId) => {
      const r = await api.post("/v1/conversations", { subjectId });
      if (!r.ok) return generalMessage(r.error);
      navigate(`/tanya/${r.value.id}`, { state: { ask: question } satisfies AskState });
      return null;
    },
    [navigate],
  );
}

export function TanyaRoute() {
  const { conversationId } = useParams();
  return conversationId === undefined ? (
    <NewConversation />
  ) : (
    <Conversation key={conversationId} id={conversationId} />
  );
}

type History = {
  readonly items: readonly ConversationSummary[];
  readonly cursor: string | null;
  readonly loading: boolean;
  readonly error: string | null;
};

function NewConversation() {
  const start = useStartConversation();
  const location = useLocation();
  // The chapter map's empty state sends students here with their subject preselected.
  const subjectFromUrl = new URLSearchParams(location.search).get("subject");
  // It only seeds the filter chip: what the chip shows is what is sent, and the student can change it.
  const [filter, setFilter] = useState<SubjectFilter>(
    subjectFromUrl !== null && isSubjectId(subjectFromUrl) ? subjectFromUrl : "semua",
  );
  const [sending, setSending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [history, setHistory] = useState<History>({
    items: [],
    cursor: null,
    loading: true,
    error: null,
  });

  const loadPage = useCallback(async (cursor: string | null, signal?: AbortSignal) => {
    setHistory((h) => ({ ...h, loading: true, error: null }));
    const r = await api.get("/v1/conversations", {
      query: { mode: "ask", limit: 20, ...(cursor === null ? {} : { cursor }) },
      ...(signal === undefined ? {} : { signal }),
    });
    if (signal?.aborted === true) return;
    setHistory((h) =>
      r.ok
        ? {
            items: [...(cursor === null ? [] : h.items), ...r.value.data.map(summaryFromDto)],
            cursor: r.value.nextCursor ?? null,
            loading: false,
            error: null,
          }
        : { ...h, loading: false, error: generalMessage(r.error) },
    );
  }, []);

  useEffect(() => {
    const ctrl = new AbortController();
    void loadPage(null, ctrl.signal);
    return () => ctrl.abort();
  }, [loadPage]);

  const send = async (question: string) => {
    if (sending) return;
    setSending(true);
    const problem = await start(question, filter === "semua" ? null : filter);
    setSending(false);
    setError(problem);
  };

  return (
    <TanyaScreen
      view={{
        ...TANYA_EMPTY,
        filter,
        busy: sending,
        messages: error === null ? [] : [{ id: "start-error", kind: "error", text: error }],
      }}
      onFilterChange={setFilter}
      onSend={(q) => void send(q)}
      onFollowUp={() => {}}
      history={{
        view: {
          items: history.items,
          hasMore: history.cursor !== null,
          loading: history.loading,
          error: history.error,
        },
        onLoadMore: () => void loadPage(history.error !== null ? null : history.cursor),
      }}
    />
  );
}

function Conversation({ id }: { readonly id: string }) {
  const navigate = useNavigate();
  const location = useLocation();
  const [titleVersion, setTitleVersion] = useState(0);
  // The server names a new conversation after its first question: show that name.
  const chat = useChat(id, () => setTitleVersion((n) => n + 1));
  const [conversation, setConversation] = useState<ConversationDto | null>(null);
  const [missing, setMissing] = useState(false);
  const asked = useRef(false);

  useEffect(() => {
    void titleVersion;
    const ctrl = new AbortController();
    void api
      .get("/v1/conversations/{id}", { pathParams: { id }, signal: ctrl.signal })
      .then((r) => {
        if (ctrl.signal.aborted) return;
        if (r.ok) setConversation(r.value);
        else if (r.error.status === 404 || r.error.code === "invalid_request") setMissing(true);
      });
    return () => ctrl.abort();
  }, [id, titleVersion]);

  // A question handed over by Beranda or the new-conversation screen is sent once, then dropped
  // from the history entry so a reload does not ask it again.
  const pending = askOf(location.state);
  const { send, load } = chat;
  useEffect(() => {
    if (pending === null || asked.current || load !== "ready") return;
    asked.current = true;
    navigate(location.pathname, { replace: true, state: null });
    void send(pending);
  }, [pending, load, send, navigate, location.pathname]);

  if (missing) return <NotFound />;
  return (
    <TanyaScreen
      view={{ ...TANYA_EMPTY, filter: null, messages: chat.messages, busy: chat.busy }}
      loading={chat.load === "loading"}
      {...(typeof chat.load === "object"
        ? { loadError: { message: chat.load.error, onRetry: chat.reload } }
        : {})}
      onFilterChange={() => {}}
      onSend={(q) => void chat.send(q)}
      onFollowUp={(followUp) => {
        const q = FOLLOW_UP_QUESTIONS[followUp];
        if (q !== undefined) {
          void chat.send(q);
          return;
        }
        const { subjectId, topic } = practiceTopicOf(
          chat.messages,
          conversation?.subjectId ?? null,
        );
        navigate(latihanHref(subjectId, topic));
      }}
      {...(conversation === null
        ? {}
        : {
            conversation: {
              title: conversation.title,
              backHref: "/tanya",
              onRename: async (title) => {
                const r = await api.patch(
                  "/v1/conversations/{id}",
                  { title },
                  { pathParams: { id } },
                );
                if (!r.ok) return generalMessage(r.error);
                setConversation(r.value);
                return null;
              },
            },
          })}
    />
  );
}
