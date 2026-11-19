import type { components } from "../../contracts/api.gen";
import type { Mission } from "../../ui";
import type { BerandaView, ContinueLesson, SubjectProgress } from "./types";

export type HomeDto = components["schemas"]["HomeDto"];
type ContinueItemDto = components["schemas"]["ContinueItemDto"];

const LESSON_STEPS: readonly string[] = ["explain", "check", "quiz"];

const chapterHref = (c: ContinueItemDto): string => `/belajar/${c.subjectId}/${c.chapterId}`;

/** The newest unfinished lesson, on the three-step stepper. */
export const continueFromDto = (c: ContinueItemDto): ContinueLesson => ({
  subjectId: c.subjectId,
  chapterTitle: c.chapterTitle,
  step: Math.max(0, LESSON_STEPS.indexOf(c.step)),
  totalSteps: LESSON_STEPS.length,
  href: chapterHref(c),
});

/** Each mission links to where it is done: the lesson to continue, when there is one. */
export const missionsFromDto = (home: HomeDto): readonly Mission[] => {
  const lesson = home.continueLearning[0];
  return home.missions.map((m) => ({
    id: m.kind,
    text: m.title,
    done: m.isCompleted,
    stars: m.rewardStars,
    href:
      m.kind === "ask"
        ? "/tanya"
        : m.kind === "lesson"
          ? lesson === undefined
            ? "/belajar"
            : chapterHref(lesson)
          : m.kind === "practice"
            ? "/latihan"
            : "/",
  }));
};

export const berandaView = (home: HomeDto, subjects: readonly SubjectProgress[]): BerandaView => {
  const lesson = home.continueLearning[0];
  return {
    studentName: home.nickname,
    missions: missionsFromDto(home),
    continueLesson: lesson === undefined ? null : continueFromDto(lesson),
    subjects,
  };
};
