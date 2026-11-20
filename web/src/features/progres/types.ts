import type { MasteryLevel } from "../../ui";

export type Period = "week" | "all";
export type ActivityRange = 7 | 30;
export type Trend = "up" | "down" | "same" | "no_data";
export type SessionKind = "practice" | "lesson" | "ask";

/** One headline number. `hint` says what it means in a child's words. */
export type StatTile = {
  readonly id: "streak" | "stars" | "lessons" | "questions" | "accuracy" | "days";
  readonly label: string;
  readonly value: string;
  readonly hint: string;
};

export type TopicRow = {
  readonly subjectId: string;
  readonly topic: string;
  readonly level: MasteryLevel;
  readonly percent: number;
  readonly attempts: number;
  readonly correct: number;
  readonly trend: Trend;
  /** "Naik", "Turun", "Tetap", "Belum bisa dibandingkan". */
  readonly trendLabel: string;
  /** "Minggu ini 80%, minggu lalu 40%". */
  readonly trendDetail: string;
  /** A kind sentence for this topic, never a scolding one. */
  readonly encouragement: string;
  /** Starts a Latihan on this topic in one tap. */
  readonly practiceHref: string;
};

export type ChapterRow = {
  readonly lessonId: string;
  readonly subjectId: string;
  readonly title: string;
  readonly stars: number;
  readonly isCompleted: boolean;
  /** "Selesai" or the step the student is on. */
  readonly status: string;
  readonly href: string;
};

export type SubjectBlock = {
  readonly subjectId: string;
  readonly name: string;
  readonly level: MasteryLevel;
  readonly percent: number;
  readonly attempts: number;
  /** "2 bab selesai dari 3 yang dimulai". */
  readonly lessonsLabel: string;
  readonly topics: readonly TopicRow[];
  readonly chapters: readonly ChapterRow[];
};

export type DayBar = {
  readonly on: string;
  /** "Sen" or "16". */
  readonly shortLabel: string;
  /** "Senin, 16 November". */
  readonly longLabel: string;
  readonly questions: number;
  readonly correct: number;
  readonly lessons: number;
  readonly asked: number;
  readonly isActive: boolean;
  readonly isToday: boolean;
};

export type ActivityView = {
  readonly range: ActivityRange;
  readonly daysActive: number;
  readonly questions: number;
  readonly lessons: number;
  readonly asked: number;
  readonly bars: readonly DayBar[];
  /** The tallest bar's value, at least 1. */
  readonly max: number;
  /** One sentence for screen readers and the chart's caption. */
  readonly summary: string;
};

export type SessionRow = {
  readonly id: string;
  readonly kind: SessionKind;
  readonly kindLabel: string;
  readonly title: string;
  readonly subjectId: string | null;
  readonly detail: string;
  readonly when: string;
  readonly href: string;
};

export type ProgresView = {
  /** Nothing practised, studied or asked yet. */
  readonly isEmpty: boolean;
  /** "16–22 November". */
  readonly weekLabel: string;
  readonly cheer: Readonly<Record<Period, string>>;
  readonly summary: Readonly<Record<Period, readonly StatTile[]>>;
  readonly practiceAgain: readonly TopicRow[];
  readonly continueLessons: readonly ChapterRow[];
  readonly subjects: readonly SubjectBlock[];
  readonly activity: Readonly<Record<ActivityRange, ActivityView>>;
  readonly recent: readonly SessionRow[];
};
