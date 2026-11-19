export type ChapterStatus = "done" | "current" | "todo";

export type ChapterNode = {
  readonly id: string;
  readonly number: number;
  readonly title: string;
  /** 0..3 earned so far. */
  readonly stars: number;
  readonly status: ChapterStatus;
};

export type ChapterMapView = {
  readonly subjectId: string;
  /** Subjects offered in the switcher at the top. */
  readonly subjectIds: readonly string[];
  readonly chapters: readonly ChapterNode[];
};

export type SubjectEntry = {
  readonly subjectId: string;
  readonly progressLabel: string;
  readonly progress: number;
  readonly href: string;
};

/** The guided session's steps; `done` once the quiz result is in. */
export type LessonStep = "explain" | "check" | "quiz" | "done";
