import { useEffect, useRef, useState } from "react";
import { useParams } from "react-router";
import { api } from "../../contracts/client";
import { generalMessage } from "../../contracts/messages";
import {
  BelajarHomeScreen,
  type ChapterDto,
  ChapterMapScreen,
  chapterNodes,
  explainPrompt,
  type LessonDto,
  type LessonResult,
  LessonScreen,
  lessonStep,
  nextStep,
  QUIZ_TOTAL,
} from "../../features/belajar";
import { ErrorState, isSubjectId, LoadingState, SUBJECTS } from "../../ui";
import { useChat } from "../chat";
import { useHome } from "../home";
import { useAuth } from "../session";
import { useSubjectGrid } from "../subjects";
import { latihanHref } from "./latihanHref";
import { NotFound } from "./NotFound";

export function BelajarHomeRoute() {
  const { grid, retry } = useSubjectGrid();
  if (grid.status === "failed") return <ErrorState message={grid.message} onRetry={retry} />;
  return <BelajarHomeScreen subjects={grid.entries} />;
}

type Load<T> =
  | { readonly status: "loading" }
  | { readonly status: "ready"; readonly value: T }
  | {
      readonly status: "failed";
      readonly message: string;
      readonly missing: boolean;
    };

export function ChapterMapRoute() {
  const { subjectId = "" } = useParams();
  if (!isSubjectId(subjectId)) return <NotFound />;
  return <ChapterMap key={subjectId} subjectId={subjectId} />;
}

function ChapterMap({ subjectId }: { readonly subjectId: string }) {
  const [selected, setSelected] = useState<string | null>(null);
  const [chapters, setChapters] = useState<Load<readonly ChapterDto[]>>({ status: "loading" });
  const [attempt, setAttempt] = useState(0);
  useEffect(() => {
    void attempt;
    const ctrl = new AbortController();
    setChapters({ status: "loading" });
    void api
      .get("/v1/subjects/{subjectId}/chapters", { pathParams: { subjectId }, signal: ctrl.signal })
      .then((r) => {
        if (ctrl.signal.aborted) return;
        setChapters(
          r.ok
            ? { status: "ready", value: r.value.data }
            : {
                status: "failed",
                message: generalMessage(r.error),
                missing: r.error.status === 404,
              },
        );
      });
    return () => ctrl.abort();
  }, [subjectId, attempt]);

  if (chapters.status === "loading") return <LoadingState text="Sebentar, aku buka peta babmu…" />;
  if (chapters.status === "failed")
    return chapters.missing ? (
      <NotFound />
    ) : (
      <ErrorState message={chapters.message} onRetry={() => setAttempt((n) => n + 1)} />
    );
  return (
    <ChapterMapScreen
      view={{
        subjectId,
        subjectIds: SUBJECTS.map((s) => s.id),
        chapters: chapterNodes(chapters.value),
      }}
      selectedChapterId={selected}
      onSelectChapter={setSelected}
      subjectHref={(id) => `/belajar/${id}`}
      chapterHref={(id) => `/belajar/${subjectId}/${id}`}
    />
  );
}

export function ChapterSessionRoute() {
  const { subjectId = "", chapterId = "" } = useParams();
  if (!isSubjectId(subjectId)) return <NotFound />;
  return (
    <LessonStart key={`${subjectId}/${chapterId}`} subjectId={subjectId} chapterId={chapterId} />
  );
}

/** POST /v1/lessons starts the chapter's lesson or resumes it. */
function LessonStart({
  subjectId,
  chapterId,
}: {
  readonly subjectId: string;
  readonly chapterId: string;
}) {
  const [lesson, setLesson] = useState<Load<LessonDto>>({ status: "loading" });
  const [title, setTitle] = useState<string | null>(null);
  const [attempt, setAttempt] = useState(0);
  useEffect(() => {
    void attempt;
    const ctrl = new AbortController();
    setLesson({ status: "loading" });
    // The lesson's own chapterTitle follows its conversation's name, which the server changes
    // after the first message; the chapters list keeps the real title.
    void api
      .get("/v1/subjects/{subjectId}/chapters", { pathParams: { subjectId }, signal: ctrl.signal })
      .then((r) => {
        if (!ctrl.signal.aborted && r.ok)
          setTitle(r.value.data.find((c) => c.id === chapterId)?.title ?? null);
      });
    void api.post("/v1/lessons", { subjectId, chapterId }, { signal: ctrl.signal }).then((r) => {
      if (ctrl.signal.aborted) return;
      setLesson(
        r.ok
          ? { status: "ready", value: r.value }
          : {
              status: "failed",
              message: generalMessage(r.error),
              missing: r.error.status === 404 || r.error.code === "invalid_request",
            },
      );
    });
    return () => ctrl.abort();
  }, [subjectId, chapterId, attempt]);

  if (lesson.status === "loading") return <LoadingState text="Sebentar, aku siapkan babnya…" />;
  if (lesson.status === "failed")
    return lesson.missing ? (
      <NotFound />
    ) : (
      <ErrorState message={lesson.message} onRetry={() => setAttempt((n) => n + 1)} />
    );
  return <Lesson initial={lesson.value} chapterTitle={title ?? lesson.value.chapterTitle} />;
}

function Lesson({
  initial,
  chapterTitle,
}: {
  readonly initial: LessonDto;
  readonly chapterTitle: string;
}) {
  const grade = useAuth().student?.grade ?? 0;
  const { refresh } = useHome();
  const chat = useChat(initial.conversationId);
  const [lesson, setLesson] = useState(initial);
  const [moving, setMoving] = useState(false);
  const [completing, setCompleting] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);
  const [result, setResult] = useState<LessonResult | null>(null);
  const started = useRef(false);
  const step = lessonStep(lesson.step);

  // The first open of a chapter starts its explanation by itself.
  const { send, load, messages } = chat;
  useEffect(() => {
    if (started.current || load !== "ready") return;
    started.current = true;
    if (messages.length === 0) void send(explainPrompt(chapterTitle, grade));
  }, [load, messages.length, send, chapterTitle, grade]);

  const next = async () => {
    const to = nextStep(step);
    if (to === null) return;
    setMoving(true);
    setActionError(null);
    const r = await api.patch(
      "/v1/lessons/{id}",
      { step: to.step },
      { pathParams: { id: lesson.id } },
    );
    setMoving(false);
    if (!r.ok) {
      setActionError(generalMessage(r.error));
      return;
    }
    setLesson(r.value);
    void chat.send(to.prompt);
  };

  const complete = async (correct: number) => {
    setCompleting(true);
    setActionError(null);
    const r = await api.post(
      "/v1/lessons/{id}/complete",
      { quizCorrect: correct, quizTotal: QUIZ_TOTAL },
      { pathParams: { id: lesson.id } },
    );
    setCompleting(false);
    if (!r.ok) {
      setActionError(generalMessage(r.error));
      return;
    }
    setLesson(r.value.lesson);
    setResult({
      correct,
      total: QUIZ_TOTAL,
      starsEarned: r.value.starsEarned,
      stars: r.value.lesson.stars,
    });
    void refresh();
  };

  return (
    <LessonScreen
      subjectId={lesson.subjectId}
      chapterTitle={chapterTitle}
      step={step}
      messages={chat.messages}
      busy={chat.busy}
      loading={chat.load === "loading"}
      {...(typeof chat.load === "object"
        ? { loadError: { message: chat.load.error, onRetry: chat.reload } }
        : {})}
      onSend={(q) => void chat.send(q)}
      onNext={() => void next()}
      moving={moving}
      onComplete={(n) => void complete(n)}
      completing={completing}
      actionError={actionError}
      result={result}
      bestStars={lesson.stars}
      mapHref={`/belajar/${lesson.subjectId}`}
      practiceHref={latihanHref(lesson.subjectId, chapterTitle)}
    />
  );
}
