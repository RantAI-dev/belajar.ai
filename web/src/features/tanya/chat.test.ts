import { describe, expect, test } from "bun:test";
import { chatReducer, emptyChat, lastQuestion, PENDING_ANSWER } from "./chat";
import type { TanyaMessage } from "./types";

const run = (...actions: Parameters<typeof chatReducer>[1][]) =>
  actions.reduce(chatReducer, emptyChat);

const stored: TanyaMessage = { id: "m1", kind: "question", text: "Kenapa rel diberi celah?" };
const final: TanyaMessage = {
  id: "m2",
  kind: "answer",
  status: "done",
  parts: [{ text: "Rel memuai." }],
  sources: [],
};

describe("chatReducer", () => {
  test("asking shows the question and an empty streaming answer, and pauses sending", () => {
    const s = run({ type: "asked", question: "Kenapa rel diberi celah?" });
    expect(s.busy).toBe(true);
    expect(s.messages.map((m) => m.kind)).toEqual(["question", "answer"]);
    const a = s.messages[1];
    expect(a?.kind === "answer" ? a.status : null).toBe("streaming");
  });

  test("a second question while busy is ignored", () => {
    const s = run({ type: "asked", question: "a" }, { type: "asked", question: "b" });
    expect(s.messages).toHaveLength(2);
  });

  test("deltas are appended in order into the streaming answer", () => {
    const s = run(
      { type: "asked", question: "q" },
      { type: "stored", question: stored },
      { type: "delta", text: "Rel " },
      { type: "delta", text: "memuai" },
    );
    expect(s.messages[0]).toEqual(stored);
    const a = s.messages.find((m) => m.id === PENDING_ANSWER);
    expect(a?.kind === "answer" ? a.parts : null).toEqual([{ text: "Rel memuai" }]);
  });

  test("done replaces the streamed answer with the stored one and resumes sending", () => {
    const s = run(
      { type: "asked", question: "q" },
      { type: "stored", question: stored },
      { type: "delta", text: "Rel" },
      { type: "answered", answer: final },
    );
    expect(s.busy).toBe(false);
    expect(s.messages).toEqual([stored, final]);
  });

  test("a failure turns the arriving answer into the friendly error", () => {
    const s = run(
      { type: "asked", question: "q" },
      { type: "failed", text: "Tidak bisa terhubung." },
    );
    expect(s.busy).toBe(false);
    expect(s.messages.map((m) => m.kind)).toEqual(["question", "error"]);
    expect(s.messages.some((m) => m.id.startsWith("pending-"))).toBe(false);
  });

  test("loading keeps a question sent before the list arrived at the end", () => {
    const s = run({ type: "asked", question: "q" }, { type: "loaded", messages: [final] });
    expect(s.messages.map((m) => m.id)).toEqual(["m2", "pending-question", PENDING_ANSWER]);
  });
});

describe("lastQuestion", () => {
  test("finds the most recent question", () => {
    expect(lastQuestion([stored, final, { id: "m3", kind: "question", text: "Lagi" }])).toBe(
      "Lagi",
    );
    expect(lastQuestion([final])).toBeNull();
  });
});
