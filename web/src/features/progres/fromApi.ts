import type { components } from "../../contracts/api.gen";
import { LESSON_STEPS } from "../belajar";
import { latihanHref } from "../latihan";
import type {
  ActivityRange,
  ActivityView,
  ChapterRow,
  DayBar,
  Period,
  ProgresView,
  SessionRow,
  StatTile,
  SubjectBlock,
  TopicRow,
  Trend,
} from "./types";

export type ProgressDto = components["schemas"]["ProgressDto"];
type PeriodSummaryDto = components["schemas"]["PeriodSummaryDto"];
type TopicProgressDto = components["schemas"]["TopicProgressDto"];
type ChapterProgressDto = components["schemas"]["ChapterProgressDto"];
type ActivityWindowDto = components["schemas"]["ActivityWindowDto"];
type RecentSessionDto = components["schemas"]["RecentSessionDto"];

/** The real chapter title, when the app knows it (the lesson's title can be a chat title). */
export type ChapterTitleOf = (subjectId: string, chapterId: string) => string | undefined;

const MONTHS = [
  "Januari",
  "Februari",
  "Maret",
  "April",
  "Mei",
  "Juni",
  "Juli",
  "Agustus",
  "September",
  "Oktober",
  "November",
  "Desember",
] as const;
const WEEKDAYS = ["Minggu", "Senin", "Selasa", "Rabu", "Kamis", "Jumat", "Sabtu"] as const;
const DAY_MS = 86_400_000;
const JAKARTA_OFFSET_MS = 7 * 3_600_000;

/** `2026-11-16` as a UTC midnight, so day arithmetic never meets a local time zone. */
const dayNumber = (on: string): number => Math.floor(Date.parse(`${on}T00:00:00Z`) / DAY_MS);

const parts = (on: string) => {
  const d = new Date(`${on}T00:00:00Z`);
  return { day: d.getUTCDate(), month: MONTHS[d.getUTCMonth()] ?? "", weekday: d.getUTCDay() };
};

/** "Senin, 16 November". */
export const longDate = (on: string): string => {
  const p = parts(on);
  return `${WEEKDAYS[p.weekday] ?? ""}, ${p.day} ${p.month}`;
};

/** "16–22 November", or "30 November – 6 Desember" across months. */
export const weekLabel = (startsOn: string, endsOn: string): string => {
  const a = parts(startsOn);
  const b = parts(endsOn);
  return a.month === b.month
    ? `${a.day}–${b.day} ${b.month}`
    : `${a.day} ${a.month} – ${b.day} ${b.month}`;
};

/** The Jakarta calendar date of an instant (fixed UTC+7, like the server). */
export const jakartaDay = (instant: string): string =>
  new Date(Date.parse(instant) + JAKARTA_OFFSET_MS).toISOString().slice(0, 10);

/** "Hari ini", "Kemarin", "3 hari lalu", or "12 November" once it is a week old. */
export const whenLabel = (instant: string, today: string): string => {
  const on = jakartaDay(instant);
  const ago = dayNumber(today) - dayNumber(on);
  if (ago <= 0) return "Hari ini";
  if (ago === 1) return "Kemarin";
  if (ago < 7) return `${ago} hari lalu`;
  const p = parts(on);
  return `${p.day} ${p.month}`;
};

/** Kind words for an accuracy, low scores included. */
export const cheerFor = (accuracy: number | null): string => {
  if (accuracy === null) return "Yuk jawab soal pertamamu. Aku temani!";
  if (accuracy >= 80) return "Hebat! Jawabanmu hampir selalu tepat.";
  if (accuracy >= 60) return "Bagus! Sedikit lagi kamu makin jago.";
  if (accuracy >= 40) return "Kamu sedang berkembang. Terus berlatih, ya!";
  return "Setiap soal bikin otakmu makin kuat. Coba lagi, pasti bisa!";
};

const TREND_LABEL: Readonly<Record<Trend, string>> = {
  up: "Naik",
  down: "Turun",
  same: "Tetap",
  no_data: "Belum bisa dibandingkan",
};

export const trendLabel = (trend: Trend): string => TREND_LABEL[trend];

const percentText = (p: number | null): string => (p === null ? "belum ada" : `${p}%`);

const topicEncouragement = (t: TopicProgressDto): string => {
  if (t.level === "mastered") return "Kamu sudah menguasai topik ini. Keren!";
  if (t.trend === "up") return "Makin baik dari minggu lalu. Lanjutkan!";
  if (t.level === "almost") return "Sedikit lagi dikuasai!";
  return "Latihan sedikit lagi, kamu pasti bisa.";
};

export const topicRow = (t: TopicProgressDto): TopicRow => ({
  subjectId: t.subjectId,
  topic: t.topic,
  level: t.level,
  percent: Math.max(0, Math.min(100, t.masteryPercent)),
  attempts: t.attempts,
  correct: t.correct,
  trend: t.trend,
  trendLabel: trendLabel(t.trend),
  trendDetail: `Minggu ini ${percentText(t.thisWeekPercent)}, minggu lalu ${percentText(t.lastWeekPercent)}`,
  encouragement: topicEncouragement(t),
  practiceHref: latihanHref(t.subjectId, t.topic, { start: true }),
});

const STEP_INDEX: Readonly<Record<string, number>> = { explain: 0, check: 1, quiz: 2 };

const chapterHref = (subjectId: string, chapterId: string): string =>
  `/belajar/${subjectId}/${chapterId}`;

export const chapterRow = (c: ChapterProgressDto, titleOf: ChapterTitleOf): ChapterRow => ({
  lessonId: c.lessonId,
  subjectId: c.subjectId,
  title: titleOf(c.subjectId, c.chapterId) ?? c.chapterTitle,
  stars: c.stars,
  isCompleted: c.isCompleted,
  status: c.isCompleted
    ? "Selesai"
    : `Sampai: ${LESSON_STEPS[STEP_INDEX[c.step] ?? 0] ?? LESSON_STEPS[0]}`,
  href: chapterHref(c.subjectId, c.chapterId),
});

const statTiles = (
  s: PeriodSummaryDto,
  streakDays: number,
  period: Period,
): readonly StatTile[] => [
  {
    id: "streak",
    label: "Runtunan",
    value: `${streakDays} hari`,
    hint: streakDays > 0 ? "belajar berturut-turut" : "mulai lagi hari ini, yuk",
  },
  {
    id: "stars",
    label: "Bintang",
    value: String(s.stars),
    hint: period === "week" ? "dikumpulkan minggu ini" : "dikumpulkan semuanya",
  },
  {
    id: "lessons",
    label: "Bab selesai",
    value: String(s.lessonsCompleted),
    hint: period === "week" ? "selesai minggu ini" : "sudah kamu tamatkan",
  },
  {
    id: "questions",
    label: "Soal dijawab",
    value: String(s.questionsAnswered),
    hint: `${s.correctAnswers} di antaranya benar`,
  },
  {
    id: "accuracy",
    label: "Ketepatan",
    value: s.accuracyPercent === null ? "–" : `${s.accuracyPercent}%`,
    hint: s.accuracyPercent === null ? "belum ada soal" : "jawaban yang tepat",
  },
  {
    id: "days",
    label: "Hari aktif",
    value: period === "week" ? `${s.daysActive} dari 7` : String(s.daysActive),
    hint: period === "week" ? "hari minggu ini" : "hari kamu belajar",
  },
];

const SHORT_WEEKDAYS = ["Min", "Sen", "Sel", "Rab", "Kam", "Jum", "Sab"] as const;

export const activityView = (w: ActivityWindowDto, range: ActivityRange): ActivityView => {
  const bars: DayBar[] = w.days.map((d) => {
    const p = parts(d.on);
    return {
      on: d.on,
      shortLabel: range === 7 ? (SHORT_WEEKDAYS[p.weekday] ?? "") : String(p.day),
      longLabel: longDate(d.on),
      questions: d.questionsAnswered,
      correct: d.correctAnswers,
      lessons: d.lessonsCompleted,
      asked: d.questionsAsked,
      isActive: d.isActive,
      isToday: d.on === w.endsOn,
    };
  });
  const max = Math.max(1, ...bars.map((b) => b.questions));
  return {
    range,
    daysActive: w.daysActive,
    questions: w.questionsAnswered,
    lessons: w.lessonsCompleted,
    asked: w.questionsAsked,
    bars,
    max,
    summary:
      `${range} hari terakhir: aktif ${w.daysActive} hari, ${w.questionsAnswered} soal dijawab, ` +
      `${w.lessonsCompleted} bab selesai, ${w.questionsAsked} pertanyaan.`,
  };
};

const KIND_LABEL = { practice: "Latihan", lesson: "Belajar", ask: "Tanya" } as const;

const sessionRow = (
  s: RecentSessionDto,
  today: string,
  chapters: ReadonlyMap<string, ChapterRow>,
): SessionRow => {
  const chapter = s.kind === "lesson" ? chapters.get(s.id) : undefined;
  const detail =
    s.kind === "practice"
      ? `${s.correctCount ?? 0} dari ${s.totalCount ?? 0} benar`
      : s.kind === "lesson"
        ? chapter?.isCompleted === true
          ? `Selesai · ${s.stars ?? 0} bintang`
          : "Sedang dipelajari"
        : "Pertanyaan";
  const href =
    s.kind === "practice"
      ? `/latihan/${s.id}/hasil`
      : s.kind === "ask"
        ? `/tanya/${s.id}`
        : (chapter?.href ?? "/belajar");
  return {
    id: s.id,
    kind: s.kind,
    kindLabel: KIND_LABEL[s.kind],
    title: chapter?.title ?? s.title,
    subjectId: s.subjectId,
    detail,
    when: whenLabel(s.occurredAt, today),
    href,
  };
};

/** Everything the Progres page shows, from GET /v1/progress. */
export const progresView = (
  dto: ProgressDto,
  titleOf: ChapterTitleOf = () => undefined,
): ProgresView => {
  const subjects: SubjectBlock[] = dto.subjects.map((s) => ({
    subjectId: s.subjectId,
    name: s.name,
    level: s.level,
    percent: Math.max(0, Math.min(100, s.masteryPercent)),
    attempts: s.attempts,
    lessonsLabel:
      s.lessonsStarted === 0
        ? "Belum ada bab yang dimulai"
        : `${s.lessonsCompleted} bab selesai dari ${s.lessonsStarted} yang dimulai`,
    topics: s.topics.map(topicRow),
    chapters: s.chapters.map((c) => chapterRow(c, titleOf)),
  }));
  const chapters = new Map(subjects.flatMap((s) => s.chapters).map((c) => [c.lessonId, c]));
  const today = dto.activity.last7Days.endsOn;
  return {
    isEmpty:
      dto.subjects.length === 0 &&
      dto.recentSessions.length === 0 &&
      dto.allTime.questionsAnswered === 0,
    weekLabel: weekLabel(dto.weekStartsOn, dto.weekEndsOn),
    cheer: {
      week: cheerFor(dto.thisWeek.accuracyPercent),
      all: cheerFor(dto.allTime.accuracyPercent),
    },
    summary: {
      week: statTiles(dto.thisWeek, dto.streakDays, "week"),
      all: statTiles(dto.allTime, dto.streakDays, "all"),
    },
    practiceAgain: dto.practiceAgain.map(topicRow),
    continueLessons: dto.continueLessons.map((c) => chapterRow(c, titleOf)),
    subjects,
    activity: {
      7: activityView(dto.activity.last7Days, 7),
      30: activityView(dto.activity.last30Days, 30),
    },
    recent: dto.recentSessions.map((s) => sessionRow(s, today, chapters)),
  };
};
