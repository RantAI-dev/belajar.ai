import { useState } from "react";
import { useNavigate, useParams } from "react-router";
import { type SubjectFilter, TANYA_EMPTY, TanyaScreen } from "../../features/tanya";
import { useDemo } from "../demo";
import { NotFound } from "./NotFound";

const FOLLOW_UP_QUESTIONS: Readonly<Record<string, string>> = {
  simpler: "Bisa jelaskan lebih mudah?",
  example: "Kasih contoh lain, dong.",
};

export function TanyaRoute() {
  const { conversationId } = useParams();
  const navigate = useNavigate();
  const demo = useDemo();
  const [filter, setFilter] = useState<SubjectFilter>("semua");
  const messages = conversationId === undefined ? [] : demo.conversations[conversationId];
  if (messages === undefined) return <NotFound />;
  const last = messages.at(-1);
  const busy = last?.kind === "answer" && last.status === "streaming";

  const send = (question: string) => {
    const id = demo.ask(conversationId ?? null, question);
    if (conversationId === undefined) navigate(`/tanya/${id}`);
  };
  return (
    <TanyaScreen
      view={{ ...TANYA_EMPTY, filter, messages, busy }}
      onFilterChange={setFilter}
      onSend={send}
      onFollowUp={(id) => {
        const q = FOLLOW_UP_QUESTIONS[id];
        if (q === undefined) navigate("/latihan");
        else send(q);
      }}
    />
  );
}
