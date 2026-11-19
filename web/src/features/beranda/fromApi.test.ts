import { describe, expect, test } from "bun:test";
import { berandaView, type HomeDto } from "./fromApi";

const HOME: HomeDto = {
  nickname: "Rani",
  grade: 7,
  stars: 40,
  streakDays: 3,
  missions: [
    {
      kind: "ask",
      title: "Tanya 1 pertanyaan",
      progress: 1,
      target: 1,
      isCompleted: true,
      rewardStars: 10,
    },
    {
      kind: "lesson",
      title: "Selesaikan 1 bab",
      progress: 0,
      target: 1,
      isCompleted: false,
      rewardStars: 10,
    },
    {
      kind: "practice",
      title: "Selesaikan 1 latihan",
      progress: 0,
      target: 1,
      isCompleted: false,
      rewardStars: 10,
    },
  ],
  continueLearning: [
    {
      lessonId: "l1",
      subjectId: "ipa",
      chapterId: "bab-4",
      chapterTitle: "Suhu, Kalor, dan Pemuaian",
      step: "check",
      stars: 0,
      updatedAt: "2026-11-19T08:30:00.000Z",
    },
  ],
};

describe("berandaView", () => {
  test("greets by nickname and lists today's missions with their links", () => {
    const v = berandaView(HOME, []);
    expect(v.studentName).toBe("Rani");
    expect(v.missions).toEqual([
      { id: "ask", text: "Tanya 1 pertanyaan", done: true, stars: 10, href: "/tanya" },
      {
        id: "lesson",
        text: "Selesaikan 1 bab",
        done: false,
        stars: 10,
        href: "/belajar/ipa/bab-4",
      },
      { id: "practice", text: "Selesaikan 1 latihan", done: false, stars: 10, href: "/latihan" },
    ]);
  });

  test("continue learning shows the newest lesson on its step", () => {
    expect(berandaView(HOME, []).continueLesson).toEqual({
      subjectId: "ipa",
      chapterTitle: "Suhu, Kalor, dan Pemuaian",
      step: 1,
      totalSteps: 3,
      href: "/belajar/ipa/bab-4",
    });
  });

  test("with nothing to continue the card is hidden and the lesson mission opens Belajar", () => {
    const v = berandaView({ ...HOME, continueLearning: [] }, []);
    expect(v.continueLesson).toBeNull();
    expect(v.missions[1]?.href).toBe("/belajar");
  });

  test("passes the subject grid through", () => {
    const grid = [
      { subjectId: "ipa", progressLabel: "Belum mulai", progress: 0, href: "/belajar/ipa" },
    ];
    expect(berandaView(HOME, grid).subjects).toBe(grid);
  });
});
