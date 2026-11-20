import { describe, expect, test } from "bun:test";
import {
  activityView,
  cheerFor,
  jakartaDay,
  type ProgressDto,
  progresView,
  topicRow,
  weekLabel,
  whenLabel,
} from "./fromApi";

const day = (on: string, questions = 0, isActive = questions > 0) => ({
  on,
  isActive,
  questionsAnswered: questions,
  correctAnswers: Math.floor(questions / 2),
  lessonsCompleted: 0,
  questionsAsked: 0,
});

const week = (endsOn: string) => ({
  startsOn: "2026-11-13",
  endsOn,
  daysActive: 2,
  questionsAnswered: 10,
  lessonsCompleted: 1,
  questionsAsked: 1,
  days: [
    day("2026-11-13"),
    day("2026-11-14"),
    day("2026-11-15", 5),
    day("2026-11-16"),
    day("2026-11-17", 0, true),
    day("2026-11-18", 10),
    day("2026-11-19"),
  ],
});

const pemuaian = {
  subjectId: "ipa",
  topic: "Pemuaian",
  attempts: 10,
  correct: 6,
  masteryPercent: 60,
  level: "almost",
  trend: "up",
  thisWeekPercent: 80,
  lastWeekPercent: 40,
  lastPracticedAt: "2026-11-18T03:00:00.000Z",
} as const;

const summary = {
  stars: 25,
  lessonsCompleted: 1,
  questionsAnswered: 10,
  correctAnswers: 5,
  accuracyPercent: 50,
  daysActive: 3,
};

const DTO: ProgressDto = {
  streakDays: 3,
  weekStartsOn: "2026-11-16",
  weekEndsOn: "2026-11-22",
  thisWeek: summary,
  allTime: { ...summary, stars: 60, questionsAnswered: 20, accuracyPercent: 85, daysActive: 5 },
  subjects: [
    {
      subjectId: "ipa",
      name: "IPA",
      attempts: 10,
      correct: 6,
      masteryPercent: 60,
      level: "almost",
      lessonsStarted: 2,
      lessonsCompleted: 1,
      topics: [pemuaian],
      chapters: [
        {
          lessonId: "l2",
          subjectId: "ipa",
          chapterId: "bab-2",
          chapterTitle: "Halo, aku mau tanya",
          step: "check",
          stars: 0,
          isCompleted: false,
          updatedAt: "2026-11-19T02:00:00.000Z",
        },
      ],
    },
  ],
  activity: { last7Days: week("2026-11-19"), last30Days: week("2026-11-19") },
  recentSessions: [
    {
      kind: "lesson",
      id: "l2",
      subjectId: "ipa",
      title: "Halo, aku mau tanya",
      correctCount: null,
      totalCount: null,
      stars: 0,
      occurredAt: "2026-11-19T02:00:00.000Z",
    },
    {
      kind: "practice",
      id: "set-1",
      subjectId: "ipa",
      title: "Pemuaian",
      correctCount: 4,
      totalCount: 5,
      stars: null,
      occurredAt: "2026-11-17T18:00:00.000Z",
    },
  ],
  practiceAgain: [pemuaian],
  continueLessons: [],
};

describe("dates in Jakarta", () => {
  test("an instant after 17:00 UTC is already the next Jakarta day", () => {
    expect(jakartaDay("2026-11-18T16:59:00.000Z")).toBe("2026-11-18");
    expect(jakartaDay("2026-11-18T17:00:00.000Z")).toBe("2026-11-19");
  });

  test("relative labels count Jakarta days", () => {
    expect(whenLabel("2026-11-19T02:00:00.000Z", "2026-11-19")).toBe("Hari ini");
    expect(whenLabel("2026-11-17T18:00:00.000Z", "2026-11-19")).toBe("Kemarin");
    expect(whenLabel("2026-11-15T03:00:00.000Z", "2026-11-19")).toBe("4 hari lalu");
    expect(whenLabel("2026-11-02T03:00:00.000Z", "2026-11-19")).toBe("2 November");
  });

  test("the week label spans months when it has to", () => {
    expect(weekLabel("2026-11-16", "2026-11-22")).toBe("16–22 November");
    expect(weekLabel("2026-11-30", "2026-12-06")).toBe("30 November – 6 Desember");
  });
});

describe("progresView", () => {
  test("is empty only when nothing was practised, studied or asked", () => {
    expect(progresView(DTO).isEmpty).toBe(false);
    const empty: ProgressDto = {
      ...DTO,
      subjects: [],
      recentSessions: [],
      practiceAgain: [],
      allTime: { ...summary, questionsAnswered: 0, accuracyPercent: null },
    };
    expect(progresView(empty).isEmpty).toBe(true);
  });

  test("summaries per period, with kind words for any score", () => {
    const view = progresView(DTO);
    const week = Object.fromEntries(view.summary.week.map((t) => [t.id, t.value]));
    expect(week).toEqual({
      streak: "3 hari",
      stars: "25",
      lessons: "1",
      questions: "10",
      accuracy: "50%",
      days: "3 dari 7",
    });
    expect(view.summary.all.find((t) => t.id === "accuracy")?.value).toBe("85%");
    expect(view.cheer.all).toBe(cheerFor(85));
    for (const accuracy of [null, 0, 20, 45, 65, 90]) expect(cheerFor(accuracy)).not.toBe("");
    expect(view.weekLabel).toBe("16–22 November");
  });

  test("Latih lagi starts a Latihan on that very topic", () => {
    const [row] = progresView(DTO).practiceAgain;
    expect(row?.practiceHref).toBe("/latihan?mapel=ipa&topik=Pemuaian&mulai=1");
    expect(row?.trendLabel).toBe("Naik");
    expect(row?.trendDetail).toBe("Minggu ini 80%, minggu lalu 40%");
  });

  test("a trend without data says so in words", () => {
    const row = topicRow({
      ...pemuaian,
      trend: "no_data",
      thisWeekPercent: null,
      level: "learning",
    });
    expect(row.trendLabel).toBe("Belum bisa dibandingkan");
    expect(row.trendDetail).toBe("Minggu ini belum ada, minggu lalu 40%");
    expect(row.encouragement).not.toContain("salah");
  });

  test("chapter titles come from the chapters list when the app knows them", () => {
    const view = progresView(DTO, (s, c) => (s === "ipa" && c === "bab-2" ? "Zat" : undefined));
    const chapter = view.subjects[0]?.chapters[0];
    expect(chapter?.title).toBe("Zat");
    expect(chapter?.status).toBe("Sampai: Cek paham");
    expect(chapter?.href).toBe("/belajar/ipa/bab-2");
    // A recent lesson session links to its chapter, under the same title.
    expect(view.recent[0]?.title).toBe("Zat");
    expect(view.recent[0]?.href).toBe("/belajar/ipa/bab-2");
    expect(view.recent[1]).toMatchObject({
      href: "/latihan/set-1/hasil",
      detail: "4 dari 5 benar",
      when: "Kemarin",
    });
  });
});

describe("activityView", () => {
  test("bars scale to the busiest day and mark today", () => {
    const view = activityView(week("2026-11-19"), 7);
    expect(view.max).toBe(10);
    expect(view.bars.map((b) => b.shortLabel)).toEqual([
      "Jum",
      "Sab",
      "Min",
      "Sen",
      "Sel",
      "Rab",
      "Kam",
    ]);
    expect(view.bars.at(-1)?.isToday).toBe(true);
    expect(view.bars[4]?.isActive).toBe(true);
    expect(view.summary).toContain("aktif 2 hari");
  });

  test("a month without any answers still has a usable scale", () => {
    const quiet = { ...week("2026-11-19"), days: [day("2026-11-19")] };
    expect(activityView(quiet, 30).max).toBe(1);
    expect(activityView(quiet, 30).bars[0]?.shortLabel).toBe("19");
  });
});
