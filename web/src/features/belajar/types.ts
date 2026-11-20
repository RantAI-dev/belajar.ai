export type ChapterStatus = "done" | "current" | "todo";

/** A topic ("materi") from the book's table of contents. Pages are 0-based, or null. */
export type ChapterTopic = {
  readonly title: string;
  readonly page: number | null;
};

export type ChapterNode = {
  readonly id: string;
  readonly number: number;
  /** The number printed in the book; may repeat across books. */
  readonly bookNumber: number;
  readonly title: string;
  /** 0-based page where the chapter starts, or null. */
  readonly page: number | null;
  readonly documentTitle: string;
  readonly topics: readonly ChapterTopic[];
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
