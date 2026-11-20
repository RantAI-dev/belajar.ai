import { describe, expect, test } from "bun:test";
import {
  chapterChoices,
  masteryRows,
  type PracticeSetDto,
  parseProgress,
  parseReady,
  practiceTypeOf,
  resultFromDto,
  setFromDto,
  submissionOf,
} from "./fromApi";

const DTO: PracticeSetDto = {
  id: "s1",
  subjectId: "ipa",
  topic: "Suhu, Kalor, dan Pemuaian",
  practiceType: "mcq_mixed",
  totalQuestions: 2,
  status: "ready",
  createdAt: "2026-11-19T08:30:00.000Z",
  result: null,
  questions: [
    {
      id: "q2",
      number: 2,
      kind: "story",
      story: "Dina menuang teh panas.",
      stem: "Mengapa gelas retak?",
      options: [
        { label: "A", text: "Berat" },
        { label: "B", text: "Memuai" },
      ],
    },
    {
      id: "q1",
      number: 1,
      kind: "single",
      story: null,
      stem: "Alat ukur suhu?",
      options: [
        { label: "A", text: "Termometer" },
        { label: "B", text: "Barometer" },
      ],
    },
  ],
};

describe("practice set", () => {
  test("kinds map to the server's practice types", () => {
    expect(["single", "multiple", "story", "mixed"].map((k) => practiceTypeOf(k as never))).toEqual(
      ["mcq_normal", "mcq_multiple_correct", "mcq_story", "mcq_mixed"],
    );
  });

  test("questions come in number order with labels as option ids; no story means none", () => {
    const set = setFromDto(DTO);
    expect(set.questions.map((q) => q.id)).toEqual(["q1", "q2"]);
    expect(set.questions[0]).toEqual({
      id: "q1",
      kind: "single",
      prompt: "Alat ukur suhu?",
      options: [
        { id: "A", text: "Termometer" },
        { id: "B", text: "Barometer" },
      ],
    });
    expect(set.questions[1]?.story).toBe("Dina menuang teh panas.");
    expect(set.submitted).toBe(false);
    expect(setFromDto({ ...DTO, status: "submitted" }).submitted).toBe(true);
  });

  test("a submission answers every question once, in order", () => {
    expect(submissionOf(setFromDto(DTO), { q2: ["B"] })).toEqual({
      answers: [
        { questionId: "q1", selected: [] },
        { questionId: "q2", selected: ["B"] },
      ],
    });
  });

  test("the result joins the server's grading with the questions", () => {
    const set = setFromDto(DTO);
    const r = resultFromDto(set, {
      correctCount: 1,
      totalCount: 2,
      starsEarned: 15,
      items: [
        {
          questionId: "q1",
          isCorrect: true,
          selected: ["A"],
          correctAnswer: ["A"],
          explanation: "Termometer mengukur suhu.",
          source: {
            number: 1,
            title: "IPA Buku Siswa Kelas VII",
            page: 70,
            isCited: true,
            pagePreviewUrl: "/v1/practice-sets/s1/questions/q1/page",
          },
        },
        {
          questionId: "q2",
          isCorrect: false,
          selected: ["A"],
          correctAnswer: ["B"],
          explanation: "Kaca memuai.",
          source: null,
        },
        {
          questionId: "gone",
          isCorrect: false,
          selected: [],
          correctAnswer: [],
          explanation: "",
        },
      ],
    });
    expect([r.correct, r.total, r.starsEarned]).toEqual([1, 2, 15]);
    expect(r.items.map((i) => i.question.id)).toEqual(["q1", "q2"]);
    expect(r.items[0]?.source).toEqual({
      label: "Lihat di IPA Buku Siswa Kelas VII · hal. 70",
      href: "/v1/practice-sets/s1/questions/q1/page",
    });
    expect(r.items[1]).toMatchObject({
      isCorrect: false,
      chosen: ["A"],
      correct: ["B"],
      source: null,
    });
  });
});

describe("setup", () => {
  test("chapters become numbered choices", () => {
    expect(
      chapterChoices([
        {
          id: "bab-4",
          number: 4,
          title: "Suhu",
          stars: 0,
          step: null,
          isCompleted: false,
          lessonId: null,
        },
        {
          id: "bab-1",
          number: 1,
          title: "Metode",
          stars: 0,
          step: null,
          isCompleted: false,
          lessonId: null,
        },
      ]),
    ).toEqual([
      { id: "bab-1", label: "Bab 1: Metode" },
      { id: "bab-4", label: "Bab 4: Suhu" },
    ]);
  });

  test("mastery keeps this subject's topics, best first, within 0..100, with the server's level", () => {
    expect(
      masteryRows(
        [
          {
            subjectId: "ipa",
            topic: "Suhu",
            attempts: 1,
            correct: 3,
            masteryPercent: 60,
            level: "almost",
          },
          {
            subjectId: "mtk",
            topic: "Pecahan",
            attempts: 1,
            correct: 5,
            masteryPercent: 100,
            level: "almost",
          },
          {
            subjectId: "ipa",
            topic: "Zat",
            attempts: 2,
            correct: 9,
            masteryPercent: 120,
            level: "mastered",
          },
        ],
        "ipa",
      ),
    ).toEqual([
      { topic: "Zat", percent: 100, level: "mastered" },
      { topic: "Suhu", percent: 60, level: "almost" },
    ]);
  });
});

describe("generation stream", () => {
  test("progress events drive the bar", () => {
    expect(parseProgress({ stage: "generating", delivered: 2, total: 5, elapsedMs: 800 })).toEqual({
      stage: "generating",
      ready: 2,
      total: 5,
    });
    expect(parseProgress({ stage: "generating", delivered: 9, total: 5 })?.ready).toBe(5);
    expect(parseProgress({ stage: "dreaming", delivered: 1, total: 5 })).toBeNull();
    expect(parseProgress({ stage: "checking", delivered: 1, total: 0 })).toBeNull();
    expect(parseProgress("x")).toBeNull();
  });

  test("ready carries the practice set", () => {
    expect(parseReady({ practiceSet: DTO })?.id).toBe("s1");
    expect(parseReady({ practiceSet: { ...DTO, questions: [{ id: 1 }] } })).toBeNull();
    expect(parseReady({})).toBeNull();
  });
});
