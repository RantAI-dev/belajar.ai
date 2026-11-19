export type QuestionKind = "single" | "multiple" | "story";
export type PracticeKind = QuestionKind | "mixed";

/** `id` is the option's label ("A"), which is what a submission sends. */
export type QuizOption = { readonly id: string; readonly text: string };

/** A question as it is answered: nothing in it reveals the answer. */
export type QuizQuestion = {
  readonly id: string;
  readonly kind: QuestionKind;
  readonly prompt: string;
  /** The short story a "story" question is about. */
  readonly story?: string;
  readonly options: readonly QuizOption[];
};

export type QuizSet = {
  readonly id: string;
  readonly subjectId: string;
  readonly topic: string;
  readonly questions: readonly QuizQuestion[];
  readonly submitted: boolean;
};

/** questionId -> chosen option ids. */
export type Answers = Readonly<Record<string, readonly string[]>>;

/** One graded question, as the server marked it. */
export type ReviewItem = {
  readonly question: QuizQuestion;
  readonly isCorrect: boolean;
  readonly chosen: readonly string[];
  readonly correct: readonly string[];
  readonly explanation: string;
  readonly source: { readonly label: string; readonly href?: string } | null;
};

export type QuizResult = {
  readonly correct: number;
  readonly total: number;
  readonly starsEarned: number;
  readonly items: readonly ReviewItem[];
};

export type Choice = { readonly id: string; readonly label: string };

export type GenerationStage = "retrieval" | "generating" | "checking" | "labelling" | "ready";

export type Generation = {
  readonly stage: GenerationStage;
  readonly ready: number;
  readonly total: number;
};

export type MasteryRow = { readonly topic: string; readonly percent: number };

export type SetupView = {
  readonly subjects: readonly Choice[];
  readonly chapters: readonly Choice[];
  readonly subjectId: string;
  readonly chapterId: string;
  readonly topic: string;
  readonly kind: PracticeKind;
  readonly count: number;
  readonly counts: readonly number[];
  /** Null until "Buatkan soal" is pressed. */
  readonly generation: Generation | null;
  /** Why the last generation failed, in the student's words. */
  readonly error: string | null;
  readonly funFact: { readonly text: string; readonly source?: string };
  /** The student's mastery of earlier topics in this subject, best first. */
  readonly mastery: readonly MasteryRow[];
};
