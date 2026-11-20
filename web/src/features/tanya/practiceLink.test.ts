import { describe, expect, test } from "bun:test";
import { practiceTopicOf, subjectOfBook, topicOfSection } from "./practiceLink";
import type { TanyaMessage } from "./types";

const answer = (section?: string): TanyaMessage => ({
  id: "a",
  kind: "answer",
  status: "done",
  parts: [{ text: "…" }],
  sources: [
    {
      id: "a-1",
      number: 1,
      label: "[1]",
      title: "t",
      book: "IPA Buku Siswa Kelas VII",
      ...(section === undefined ? {} : { section }),
    },
  ],
});

describe("practice link from a conversation", () => {
  test("strips the chapter number from a section", () => {
    expect(topicOfSection("Bab 5: Suhu, Kalor, dan Pemuaian")).toBe("Suhu, Kalor, dan Pemuaian");
    expect(topicOfSection("Chapter 1 - About me")).toBe("About me");
    expect(topicOfSection("Pecahan")).toBe("Pecahan");
  });

  test("reads the subject from the book title, preferring the longest name", () => {
    expect(subjectOfBook("IPA Buku Siswa Kelas VII")).toBe("ipa");
    expect(subjectOfBook("Bahasa Indonesia Kelas VII")).toBe("bind");
    expect(subjectOfBook("Buku Lain")).toBeNull();
  });

  test("uses the latest answer's chapter and book", () => {
    const q: TanyaMessage = { id: "q", kind: "question", text: "Kenapa rel diberi celah?" };
    expect(practiceTopicOf([q, answer("Bab 5: Suhu, Kalor, dan Pemuaian")], null)).toEqual({
      subjectId: "ipa",
      topic: "Suhu, Kalor, dan Pemuaian",
    });
  });

  test("the conversation's own subject wins; without a section the question is the topic", () => {
    const q: TanyaMessage = { id: "q", kind: "question", text: "Apa itu pecahan?" };
    expect(practiceTopicOf([q, answer()], "mtk")).toEqual({
      subjectId: "mtk",
      topic: "Apa itu pecahan",
    });
  });
});
