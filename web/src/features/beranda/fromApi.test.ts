import { describe, expect, test } from "bun:test";
import { berandaView, type HomeDto } from "./fromApi";
import {
  type HeroMoodInput,
  heroMood,
  REACTION_MS,
  reactionFrames,
  reactionLine,
} from "./mascotMood";

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

describe("heroMood", () => {
  const NOW = new Date("2026-11-20T08:00:00.000Z");
  const mood = (over: Partial<HeroMoodInput>) =>
    heroMood({
      missionsDone: 0,
      missionsTotal: 3,
      streakDays: 0,
      lastActivityAt: null,
      now: NOW,
      ...over,
    }).expression;

  test("nothing done today and no streak: an inviting cheerful face", () => {
    const m = heroMood({
      missionsDone: 0,
      missionsTotal: 3,
      streakDays: 0,
      lastActivityAt: null,
      now: NOW,
    });
    expect(m).toEqual({ expression: "cheerful", line: "Yuk mulai belajar hari ini!" });
  });

  test("every mission done celebrates, even on a streak milestone", () => {
    expect(mood({ missionsDone: 3 })).toBe("celebration");
    expect(mood({ missionsDone: 3, streakDays: 7 })).toBe("celebration");
  });

  test("some missions done: happy, excited on a streak milestone", () => {
    expect(mood({ missionsDone: 1, streakDays: 2 })).toBe("happy");
    expect(mood({ missionsDone: 1, streakDays: 3 })).toBe("excited");
    expect(mood({ missionsDone: 2, streakDays: 14 })).toBe("excited");
    expect(mood({ missionsDone: 2, streakDays: 15 })).toBe("happy");
  });

  test("nothing yet today but the streak is alive: an idea to keep it", () => {
    expect(mood({ streakDays: 4 })).toBe("idea");
  });

  test("away a week or more with no streak: sleepy; less than that: cheerful", () => {
    expect(mood({ lastActivityAt: "2026-11-12T08:00:00.000Z" })).toBe("sleepy");
    expect(mood({ lastActivityAt: "2026-11-17T08:00:00.000Z" })).toBe("cheerful");
    expect(mood({ lastActivityAt: "not a date" })).toBe("cheerful");
  });

  test("berandaView derives the mood from the home response", () => {
    // One mission done on a 3-day streak.
    expect(berandaView(HOME, [], NOW).mood.expression).toBe("excited");
    const fresh = { ...HOME, streakDays: 0, continueLearning: [], missions: [] };
    expect(berandaView(fresh, [], NOW).mood.expression).toBe("cheerful");
  });
});

describe("mascot reaction", () => {
  test("a tap cycles excited, surprised, happy within the reaction time", () => {
    const frames = reactionFrames(false);
    expect(frames.map((f) => f.expression)).toEqual(["excited", "surprised", "happy"]);
    expect(frames.every((f) => f.at < REACTION_MS)).toBe(true);
  });

  test("under reduced motion one face is held, no cycling", () => {
    expect(reactionFrames(true)).toEqual([{ expression: "excited", at: 0 }]);
  });

  test("taps take turns through the one-liners", () => {
    expect(reactionLine(0)).not.toBe(reactionLine(1));
    expect(reactionLine(5)).toBe(reactionLine(0));
    expect(reactionLine(0).length).toBeGreaterThan(0);
  });
});
