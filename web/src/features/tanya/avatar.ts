import type { MascotIconKind } from "../../ui";
import type { TanyaMessage } from "./types";

type AssistantMessage = Exclude<TanyaMessage, { kind: "question" }>;

/**
 * The mascot icon beside an assistant bubble. "Not in your book" comes from the stored answer's
 * `isGrounded` flag (see messageFromDto), never from the answer's wording.
 */
export const avatarFor = (m: AssistantMessage): MascotIconKind => {
  switch (m.kind) {
    case "answer":
      return m.status === "streaming" ? "thinking" : "assistant";
    case "not-in-book":
      return "confused";
    case "error":
      return "error";
  }
};
