import { expect, test } from "bun:test";
import { avatarFor } from "./avatar";
import { chatReducer, emptyChat } from "./chat";
import { type MessageDto, messageFromDto } from "./fromApi";

const answer = (status: "streaming" | "done") =>
  ({ id: "a", kind: "answer", status, parts: [], sources: [] }) as const;

test("an answer arriving shows the thinking mascot, a finished one the assistant", () => {
  expect(avatarFor(answer("streaming"))).toBe("thinking");
  expect(avatarFor(answer("done"))).toBe("assistant");
});

test("an answer that is not in the book shows the confused mascot", () => {
  expect(avatarFor({ id: "n", kind: "not-in-book", gradeLabel: "Kelas 7" })).toBe("confused");
});

test("a failed answer shows the sad error mascot", () => {
  expect(avatarFor({ id: "e", kind: "error", text: "Koneksi terputus." })).toBe("error");
});

test("the avatar follows the chat as it streams, fails and loads", () => {
  const asked = chatReducer(emptyChat, { type: "asked", question: "Apa itu kalor?" });
  const pending = asked.messages.at(-1);
  expect(pending?.kind === "answer" && avatarFor(pending)).toBe("thinking");
  const failed = chatReducer(asked, { type: "failed", text: "Koneksi terputus." });
  const error = failed.messages.at(-1);
  expect(error?.kind === "error" && avatarFor(error)).toBe("error");
  const ungrounded: MessageDto = {
    id: "m1",
    conversationId: "c1",
    role: "assistant",
    content: "Maaf, itu tidak ada di bukumu.",
    sources: [],
    isGrounded: false,
    createdAt: "2026-11-20T08:30:00.000Z",
  };
  const stored = messageFromDto(ungrounded, "Kelas 7");
  expect(stored.kind !== "question" && avatarFor(stored)).toBe("confused");
});
