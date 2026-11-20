import type { components } from "../../contracts/api.gen";
import { progressPercent } from "../../ui";
import type { ChapterNode, LessonStep, SubjectEntry } from "./types";

export type ChapterDto = components["schemas"]["ChapterDto"];
export type LessonDto = components["schemas"]["LessonDto"];

/**
 * The chapter map: finished chapters are done; the one the student is in (or else the first not
 * finished) is current; the rest wait.
 */
export const chapterNodes = (chapters: readonly ChapterDto[]): readonly ChapterNode[] => {
  const sorted = [...chapters].sort((a, b) => a.number - b.number);
  const started = sorted.find((c) => !c.isCompleted && c.step !== null);
  const current = started ?? sorted.find((c) => !c.isCompleted);
  return sorted.map((c) => ({
    id: c.id,
    number: c.number,
    bookNumber: c.bookNumber,
    title: c.title,
    page: c.page,
    documentTitle: c.documentTitle,
    topics: c.topics.map((topic) => ({ title: topic.title, page: topic.page })),
    stars: Math.max(0, Math.min(3, c.stars)),
    status: c.isCompleted ? "done" : c.id === current?.id ? "current" : "todo",
  }));
};

/** A subject card: "Bab 2 dari 5 selesai" and how far along. `null` while chapters load. */
export const subjectEntry = (
  subjectId: string,
  chapters: readonly ChapterDto[] | null,
): SubjectEntry => {
  const href = `/belajar/${subjectId}`;
  if (chapters === null) return { subjectId, progressLabel: "…", progress: 0, href };
  const total = chapters.length;
  const done = chapters.filter((c) => c.isCompleted).length;
  const started = chapters.filter((c) => !c.isCompleted && c.step !== null).length;
  const progressLabel =
    total === 0
      ? "Segera hadir"
      : done > 0
        ? `Bab ${done} dari ${total} selesai`
        : started > 0
          ? "Sedang belajar"
          : "Belum mulai";
  // A started chapter counts half, so the bar moves as soon as the student begins.
  return { subjectId, progressLabel, progress: progressPercent(done + started / 2, total), href };
};

/** The server's step name; anything unknown starts from the explanation. */
export const lessonStep = (step: string | null): LessonStep =>
  step === "check" || step === "quiz" || step === "done" ? step : "explain";
