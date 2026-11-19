import type { components } from "../../contracts/api.gen";
import type { ChapterDto } from "../belajar";
import type {
  Answers,
  Choice,
  Generation,
  MasteryRow,
  PracticeKind,
  QuizQuestion,
  QuizResult,
  QuizSet,
} from "./types";

export type PracticeSetDto = components["schemas"]["PracticeSetDto"];
type QuestionDto = components["schemas"]["QuestionDto"];
type PracticeResultDto = components["schemas"]["PracticeResultDto"];
type SourceDto = components["schemas"]["SourceDto"];
type TopicMasteryDto = components["schemas"]["TopicMasteryDto"];
type SubmitPracticeRequest = components["schemas"]["SubmitPracticeRequest"];

const PRACTICE_TYPES: Readonly<Record<PracticeKind, string>> = {
  single: "mcq_normal",
  multiple: "mcq_multiple_correct",
  story: "mcq_story",
  mixed: "mcq_mixed",
};

/** The `practiceType` the server takes for a kind of practice. */
export const practiceTypeOf = (kind: PracticeKind): string => PRACTICE_TYPES[kind];

export const questionFromDto = (q: QuestionDto): QuizQuestion => {
  const story = q.story ?? "";
  return {
    id: q.id,
    kind: q.kind,
    prompt: q.stem,
    ...(story === "" ? {} : { story }),
    options: q.options.map((o) => ({ id: o.label, text: o.text })),
  };
};

export const setFromDto = (s: PracticeSetDto): QuizSet => ({
  id: s.id,
  subjectId: s.subjectId,
  topic: s.topic,
  questions: [...s.questions].sort((a, b) => a.number - b.number).map(questionFromDto),
  submitted: s.status === "submitted",
});

const sourceLink = (s: SourceDto): { label: string; href?: string } => {
  const page = s.page ?? null;
  const url = s.pagePreviewUrl ?? null;
  return {
    label: `Lihat di ${s.title}${page === null ? "" : ` · hal. ${page}`}`,
    ...(url === null ? {} : { href: url }),
  };
};

/** The server's review, joined with the questions it grades; a question it skipped is left out. */
export const resultFromDto = (set: QuizSet, r: PracticeResultDto): QuizResult => ({
  correct: r.correctCount,
  total: r.totalCount,
  starsEarned: r.starsEarned,
  items: r.items.flatMap((item) => {
    const question = set.questions.find((q) => q.id === item.questionId);
    if (question === undefined) return [];
    const source = item.source ?? null;
    return [
      {
        question,
        isCorrect: item.isCorrect,
        chosen: item.selected,
        correct: item.correctAnswer,
        explanation: item.explanation,
        source: source === null ? null : sourceLink(source),
      },
    ];
  }),
});

/** Every question once, in order: what POST .../submit takes. */
export const submissionOf = (set: QuizSet, answers: Answers): SubmitPracticeRequest => ({
  answers: set.questions.map((q) => ({ questionId: q.id, selected: [...(answers[q.id] ?? [])] })),
});

/** "Bab 4: Suhu, Kalor, dan Pemuaian" for the chapter picker. */
export const chapterChoices = (chapters: readonly ChapterDto[]): readonly Choice[] =>
  [...chapters]
    .sort((a, b) => a.number - b.number)
    .map((c) => ({ id: c.id, label: `Bab ${c.number}: ${c.title}` }));

/** This subject's topics, best mastered first. */
export const masteryRows = (
  rows: readonly TopicMasteryDto[],
  subjectId: string,
): readonly MasteryRow[] =>
  rows
    .filter((r) => r.subjectId === subjectId)
    .map((r) => ({ topic: r.topic, percent: Math.max(0, Math.min(100, r.masteryPercent)) }))
    .sort((a, b) => b.percent - a.percent);

const isRecord = (v: unknown): v is Record<string, unknown> =>
  typeof v === "object" && v !== null && !Array.isArray(v);

const STAGES = ["retrieval", "generating", "checking", "labelling"] as const;
const isStage = (v: unknown): v is (typeof STAGES)[number] =>
  typeof v === "string" && (STAGES as readonly string[]).includes(v);

/** A `progress` event as the progress bar shows it; anything malformed is null. */
export const parseProgress = (data: unknown): Generation | null => {
  if (!isRecord(data)) return null;
  const { stage, delivered, total } = data;
  if (!isStage(stage) || typeof delivered !== "number" || typeof total !== "number") return null;
  if (total <= 0) return null;
  return { stage, ready: Math.max(0, Math.min(total, delivered)), total };
};

const isQuestion = (v: unknown): boolean =>
  isRecord(v) &&
  typeof v.id === "string" &&
  typeof v.stem === "string" &&
  (v.kind === "single" || v.kind === "multiple" || v.kind === "story") &&
  Array.isArray(v.options) &&
  v.options.every((o) => isRecord(o) && typeof o.label === "string" && typeof o.text === "string");

/** The `ready` event's practice set, or null when it is not one. */
export const parseReady = (data: unknown): PracticeSetDto | null => {
  if (!isRecord(data) || !isRecord(data.practiceSet)) return null;
  const s = data.practiceSet;
  if (typeof s.id !== "string" || typeof s.subjectId !== "string") return null;
  if (!Array.isArray(s.questions) || !s.questions.every(isQuestion)) return null;
  return s as PracticeSetDto;
};
