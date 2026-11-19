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

export type SessionOption = { readonly id: string; readonly text: string };

export type SessionQuestion = {
  readonly id: string;
  readonly prompt: string;
  readonly options: readonly SessionOption[];
  readonly answerId: string;
  readonly explanation: string;
};

export type SessionSource = {
  readonly label: string;
  readonly title: string;
  readonly fullPageHref: string;
};

export type SessionContent = {
  readonly subjectId: string;
  readonly chapterId: string;
  readonly chapterTitle: string;
  readonly explanation: readonly string[];
  readonly source: SessionSource;
  readonly check: SessionQuestion;
  readonly quiz: readonly SessionQuestion[];
};
