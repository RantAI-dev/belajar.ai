import type { Mission } from "../../ui";

export type ContinueLesson = {
  readonly subjectId: string;
  readonly chapterTitle: string;
  /** 0-based index of the step the student is on. */
  readonly step: number;
  readonly totalSteps: number;
  readonly href: string;
};

export type SubjectProgress = {
  readonly subjectId: string;
  readonly progressLabel: string;
  /** 0..100 */
  readonly progress: number;
  readonly href: string;
};

export type BerandaView = {
  readonly studentName: string;
  readonly missions: readonly Mission[];
  readonly continueLesson: ContinueLesson | null;
  readonly subjects: readonly SubjectProgress[];
};
