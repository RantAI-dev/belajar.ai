export type QuestionKind = "single" | "multiple" | "story";
export type PracticeKind = QuestionKind | "mixed";

export type QuizOption = { readonly id: string; readonly text: string };

export type QuizQuestion = {
  readonly id: string;
  readonly kind: QuestionKind;
  readonly prompt: string;
  /** The short story a "story" question is about. */
  readonly story?: string;
  readonly options: readonly QuizOption[];
  readonly answerIds: readonly string[];
  readonly explanation: string;
  readonly source: { readonly label: string; readonly href: string };
};

export type QuizSet = {
  readonly id: string;
  readonly subjectId: string;
  readonly chapterTitle: string;
  readonly questions: readonly QuizQuestion[];
};

/** questionId -> chosen option ids. */
export type Answers = Readonly<Record<string, readonly string[]>>;

export type Choice = { readonly id: string; readonly label: string };

export type Generation = { readonly ready: number; readonly total: number };

export type SetupView = {
  readonly subjects: readonly Choice[];
  readonly chapters: readonly Choice[];
  readonly subjectId: string;
  readonly chapterId: string;
  readonly kind: PracticeKind;
  readonly count: number;
  readonly counts: readonly number[];
  /** Null until "Buatkan soal" is pressed. */
  readonly generation: Generation | null;
  readonly funFact: { readonly text: string; readonly source: string };
};
