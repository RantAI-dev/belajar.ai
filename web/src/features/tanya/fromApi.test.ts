import { describe, expect, test } from "bun:test";
import {
  dateLabel,
  type MessageDto,
  messageFromDto,
  parseDelta,
  parseMessage,
  sourceFromDto,
  summaryFromDto,
} from "./fromApi";

const SOURCE = {
  number: 1,
  title: "IPA Buku Siswa Kelas VII",
  section: "Bab 5: Suhu, Kalor, dan Pemuaian",
  page: 78,
  isCited: true,
  pagePreviewUrl: "/v1/messages/m2/sources/1/page",
};

const answer = (over: Partial<MessageDto> = {}): MessageDto => ({
  id: "m2",
  conversationId: "c1",
  role: "assistant",
  content: "Rel memuai [1].",
  sources: [SOURCE, { ...SOURCE, number: 2, page: 81, isCited: false }],
  isGrounded: true,
  createdAt: "2026-11-19T08:30:00.000Z",
  ...over,
});

describe("sourceFromDto", () => {
  test("labels the chip with its number and page, and previews the same-origin page", () => {
    expect(sourceFromDto("m2", SOURCE)).toEqual({
      id: "m2-1",
      label: "[1] IPA Buku Siswa Kelas VII · hal. 78",
      title: "IPA Buku Siswa Kelas VII · Bab 5: Suhu, Kalor, dan Pemuaian · halaman 78",
      book: "IPA Buku Siswa Kelas VII",
      section: "Bab 5: Suhu, Kalor, dan Pemuaian",
      imageUrl: "/v1/messages/m2/sources/1/page",
      fullPageHref: "/v1/messages/m2/sources/1/page",
    });
  });

  test("a source without page or image has no preview link", () => {
    const s = sourceFromDto("m2", { number: 3, title: "Buku", isCited: true });
    expect(s).toEqual({ id: "m2-3", label: "[3] Buku", title: "Buku", book: "Buku" });
  });
});

describe("messageFromDto", () => {
  test("a question is the student's bubble", () => {
    expect(
      messageFromDto(
        answer({ id: "m1", role: "user", content: "Kenapa?", sources: [] }),
        "Kelas 7",
      ),
    ).toEqual({ id: "m1", kind: "question", text: "Kenapa?" });
  });

  test("a grounded answer shows only its cited sources", () => {
    const m = messageFromDto(answer(), "Kelas 7");
    expect(m.kind).toBe("answer");
    if (m.kind !== "answer") return;
    expect(m.status).toBe("done");
    expect(m.parts).toEqual([{ text: "Rel memuai [1]." }]);
    expect(m.sources.map((s) => s.id)).toEqual(["m2-1"]);
  });

  test("an answer that cites none by number still shows the passages it used", () => {
    const m = messageFromDto(answer({ sources: [{ ...SOURCE, isCited: false }] }), "Kelas 7");
    expect(m.kind === "answer" ? m.sources.length : -1).toBe(1);
  });

  test("an ungrounded answer, or one without sources, is 'not in your book'", () => {
    expect(messageFromDto(answer({ isGrounded: false, sources: [] }), "Kelas 7")).toEqual({
      id: "m2",
      kind: "not-in-book",
      gradeLabel: "Kelas 7",
    });
    expect(messageFromDto(answer({ sources: [] }), "Kelas 8").kind).toBe("not-in-book");
  });
});

describe("conversation list", () => {
  test("a summary links to its conversation", () => {
    const s = summaryFromDto({
      id: "c1",
      mode: "ask",
      subjectId: null,
      title: "Kenapa rel diberi celah?",
      createdAt: "2026-11-19T08:30:00.000Z",
      updatedAt: "2026-11-19T08:30:00.000Z",
    });
    expect(s.href).toBe("/tanya/c1");
    expect(s.subjectId).toBeNull();
    expect(s.dateLabel).toMatch(/^\d{1,2} (Nov)$/);
  });

  test("a broken date has no label", () => {
    expect(dateLabel("kemarin")).toBe("");
  });
});

describe("stream events", () => {
  test("parseMessage keeps stored messages and refuses anything else", () => {
    expect(parseMessage(answer())?.id).toBe("m2");
    expect(parseMessage({ id: "x" })).toBeNull();
    expect(parseMessage({ ...answer(), role: "system" })).toBeNull();
    expect(parseMessage({ ...answer(), sources: [{ number: "1" }] })).toBeNull();
    expect(parseMessage("text")).toBeNull();
  });

  test("parseDelta reads { text }", () => {
    expect(parseDelta({ text: "Rel " })).toBe("Rel ");
    expect(parseDelta({ text: 1 })).toBeNull();
    expect(parseDelta(null)).toBeNull();
  });
});
