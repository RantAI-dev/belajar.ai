import { useEffect, useReducer, useRef, useState } from "react";
import { useNavigate, useParams, useSearchParams } from "react-router";
import { api } from "../../contracts/client";
import { generalMessage } from "../../contracts/messages";
import { postStream } from "../../contracts/sse";
import type { ChapterDto } from "../../features/belajar";
import {
  canSubmit,
  chapterChoices,
  funFactFor,
  type Generation,
  LATIHAN_SUBJECTS,
  LatihanSetupScreen,
  type MasteryRow,
  masteryRows,
  PRACTICE_COUNTS,
  type PracticeKind,
  type PracticeSetDto,
  parseProgress,
  parseReady,
  practiceTypeOf,
  QuizScreen,
  quizReducer,
  ResultScreen,
  resultFromDto,
  setFromDto,
  startQuiz,
  submissionOf,
} from "../../features/latihan";
import { ErrorState, isSubjectId, LoadingState } from "../../ui";
import { useHome } from "../home";
import { useAbortOnLeave } from "../leave";
import { rememberedSet, rememberSet } from "../practiceCache";
import { latihanHref } from "./latihanHref";
import { NotFound } from "./NotFound";

const NO_SET = "Soalnya belum jadi. Coba buat lagi, ya.";

export function LatihanSetupRoute() {
  const navigate = useNavigate();
  const [params] = useSearchParams();
  const controller = useAbortOnLeave();
  const fromLink = params.get("mapel") ?? "";
  const [subjectId, setSubjectId] = useState<string>(isSubjectId(fromLink) ? fromLink : "ipa");
  const [chapters, setChapters] = useState<readonly ChapterDto[]>([]);
  const [chapterId, setChapterId] = useState("");
  const [topic, setTopic] = useState(params.get("topik") ?? "");
  const [kind, setKind] = useState<PracticeKind>("single");
  const [count, setCount] = useState(5);
  const [generation, setGeneration] = useState<Generation | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [mastery, setMastery] = useState<readonly MasteryRow[]>([]);
  // A topic handed over by Tanya or Belajar stays until the student picks a chapter.
  const [keepTopic, setKeepTopic] = useState(params.get("topik") !== null);
  // Read when a subject's chapters arrive, without reloading them on every keystroke.
  const latest = useRef({ topic, keepTopic });
  latest.current = { topic, keepTopic };

  useEffect(() => {
    const ctrl = new AbortController();
    void api
      .get("/v1/subjects/{subjectId}/chapters", { pathParams: { subjectId }, signal: ctrl.signal })
      .then((r) => {
        if (ctrl.signal.aborted) return;
        const list = r.ok ? r.value.data : [];
        setChapters(list);
        const lower = latest.current.topic.trim().toLowerCase();
        const match = list.find((c) => c.title.toLowerCase() === lower);
        const first = match ?? list.find((c) => !c.isCompleted) ?? list[0];
        setChapterId(first?.id ?? "");
        if (!latest.current.keepTopic && first !== undefined) setTopic(first.title);
      });
    void api.get("/v1/topic-mastery", { query: { subjectId }, signal: ctrl.signal }).then((r) => {
      if (!ctrl.signal.aborted && r.ok) setMastery(masteryRows(r.value.data, subjectId));
    });
    return () => ctrl.abort();
  }, [subjectId]);

  const reset = () => {
    setGeneration(null);
    setError(null);
  };

  const generate = async () => {
    if (generation !== null && generation.stage !== "ready") return;
    setError(null);
    setGeneration({ stage: "retrieval", ready: 0, total: count });
    // An object, so the stream callback's write is visible after the await.
    const got: { set: PracticeSetDto | null } = { set: null };
    const r = await postStream(
      "/v1/practice-sets",
      { subjectId, topic: topic.trim(), practiceType: practiceTypeOf(kind), totalQuestions: count },
      {
        signal: controller().signal,
        onEvent: (e) => {
          if (e.event === "progress") {
            const g = parseProgress(e.data);
            if (g !== null) setGeneration(g);
          } else if (e.event === "ready") {
            got.set = parseReady(e.data);
          }
        },
      },
    );
    const ready = got.set;
    if (!r.ok || ready === null) {
      if (r.ok && r.value === "aborted") return;
      setGeneration(null);
      setError(r.ok ? NO_SET : generalMessage(r.error));
      return;
    }
    rememberSet(ready);
    setGeneration({ stage: "ready", ready: ready.questions.length, total: ready.questions.length });
    navigate(`/latihan/${ready.id}`);
  };

  return (
    <LatihanSetupScreen
      view={{
        subjects: LATIHAN_SUBJECTS,
        chapters: chapterChoices(chapters),
        subjectId,
        chapterId,
        topic,
        kind,
        count,
        counts: PRACTICE_COUNTS,
        generation,
        error,
        funFact: { text: funFactFor(subjectId) },
        mastery,
      }}
      onSubjectChange={(id) => {
        setSubjectId(id);
        setKeepTopic(false);
        reset();
      }}
      onChapterChange={(id) => {
        setChapterId(id);
        const c = chapters.find((x) => x.id === id);
        if (c !== undefined) setTopic(c.title);
        setKeepTopic(false);
        reset();
      }}
      onTopicChange={(t) => {
        setTopic(t);
        reset();
      }}
      onKindChange={(k) => {
        setKind(k);
        reset();
      }}
      onCountChange={(n) => {
        setCount(n);
        reset();
      }}
      onGenerate={() => void generate()}
    />
  );
}

type SetLoad =
  | { readonly status: "loading" }
  | { readonly status: "ready"; readonly set: PracticeSetDto }
  | { readonly status: "failed"; readonly message: string; readonly missing: boolean };

/** The set from the stream that just made it, or GET /v1/practice-sets/{id}. */
function usePracticeSet(id: string): { readonly load: SetLoad; readonly retry: () => void } {
  const [load, setLoad] = useState<SetLoad>(() => {
    const known = rememberedSet(id);
    return known === undefined ? { status: "loading" } : { status: "ready", set: known };
  });
  const [attempt, setAttempt] = useState(0);
  useEffect(() => {
    void attempt;
    if (rememberedSet(id) !== undefined) return;
    const ctrl = new AbortController();
    setLoad({ status: "loading" });
    void api
      .get("/v1/practice-sets/{id}", { pathParams: { id }, signal: ctrl.signal })
      .then((r) => {
        if (ctrl.signal.aborted) return;
        if (r.ok) rememberSet(r.value);
        setLoad(
          r.ok
            ? { status: "ready", set: r.value }
            : {
                status: "failed",
                message: generalMessage(r.error),
                missing: r.error.status === 404 || r.error.code === "invalid_request",
              },
        );
      });
    return () => ctrl.abort();
  }, [id, attempt]);
  return { load, retry: () => setAttempt((n) => n + 1) };
}

export function QuizRoute() {
  const { setId = "" } = useParams();
  const { load, retry } = usePracticeSet(setId);
  if (load.status === "loading") return <LoadingState text="Sebentar, aku ambil soalmu…" />;
  if (load.status === "failed")
    return load.missing ? <NotFound /> : <ErrorState message={load.message} onRetry={retry} />;
  return <Quiz key={load.set.id} dto={load.set} />;
}

function Quiz({ dto }: { readonly dto: PracticeSetDto }) {
  const navigate = useNavigate();
  const { refresh } = useHome();
  const set = setFromDto(dto);
  const [state, dispatch] = useReducer(quizReducer(set), undefined, startQuiz);
  const [submitting, setSubmitting] = useState(false);
  const [submitError, setSubmitError] = useState<string | null>(null);

  // A set already sent for grading only has its results to show.
  useEffect(() => {
    if (set.submitted) navigate(`/latihan/${set.id}/hasil`, { replace: true });
  }, [set.submitted, set.id, navigate]);

  const question = set.questions[state.index];
  if (question === undefined) return <ErrorState message={NO_SET} />;
  const chosen = state.answers[question.id] ?? [];

  const finish = async () => {
    setSubmitting(true);
    setSubmitError(null);
    const r = await api.post("/v1/practice-sets/{id}/submit", submissionOf(set, state.answers), {
      pathParams: { id: set.id },
    });
    setSubmitting(false);
    if (!r.ok) {
      setSubmitError(generalMessage(r.error));
      return;
    }
    rememberSet({ ...dto, status: "submitted", result: r.value });
    void refresh();
    navigate(`/latihan/${set.id}/hasil`);
  };

  return (
    <QuizScreen
      topic={set.topic}
      subjectId={set.subjectId}
      question={question}
      index={state.index}
      total={set.questions.length}
      chosen={chosen}
      onChoose={(optionId) => dispatch({ type: "choose", optionId })}
      onPrev={() => dispatch({ type: "prev" })}
      onNext={() => dispatch({ type: "next" })}
      onFinish={() => void finish()}
      canFinish={canSubmit(set, state)}
      submitting={submitting}
      submitError={submitError}
    />
  );
}

export function ResultRoute() {
  const { setId = "" } = useParams();
  const { load, retry } = usePracticeSet(setId);
  if (load.status === "loading") return <LoadingState text="Sebentar, aku ambil hasilmu…" />;
  if (load.status === "failed")
    return load.missing ? <NotFound /> : <ErrorState message={load.message} onRetry={retry} />;
  const dto = load.set;
  const result = dto.result ?? null;
  // Not graded yet: the student still has to answer it.
  if (result === null) return <QuizRoute />;
  return (
    <ResultScreen
      result={resultFromDto(setFromDto(dto), result)}
      againHref={latihanHref(dto.subjectId, dto.topic)}
      studyHref={`/belajar/${dto.subjectId}`}
    />
  );
}
