import { useEffect, useState } from "react";
import { useNavigate, useParams } from "react-router";
import {
  type Answers,
  isCorrect,
  LATIHAN_CHAPTERS,
  LATIHAN_SAMPLE_ANSWERS,
  LATIHAN_SET,
  LATIHAN_SETUP,
  LatihanSetupScreen,
  type PracticeKind,
  pointsFor,
  QuizScreen,
  ResultScreen,
  scoreSet,
  toggleChoice,
} from "../../features/latihan";
import { useDemo } from "../demo";
import { demoSet } from "../demoQuiz";
import { useInterval } from "../timers";
import { NotFound } from "./NotFound";

const READY_EVERY_MS = 650;
const OPEN_AFTER_MS = 900;

export function LatihanSetupRoute() {
  const navigate = useNavigate();
  const demo = useDemo();
  const [subjectId, setSubjectId] = useState(LATIHAN_SETUP.subjectId);
  const [chapterId, setChapterId] = useState(LATIHAN_SETUP.chapterId);
  const [kind, setKind] = useState<PracticeKind>(LATIHAN_SETUP.kind);
  const [count, setCount] = useState(LATIHAN_SETUP.count);
  const [ready, setReady] = useState<number | null>(null);
  const setId = `set-${count}`;
  const chapters = LATIHAN_CHAPTERS[subjectId] ?? [];

  const generating = ready !== null && ready < count;
  useInterval(
    () => setReady((r) => (r === null ? r : Math.min(count, r + 1))),
    generating ? READY_EVERY_MS : null,
  );
  useEffect(() => {
    if (ready === null || ready < count) return;
    const t = window.setTimeout(() => navigate(`/latihan/${setId}`), OPEN_AFTER_MS);
    return () => window.clearTimeout(t);
  }, [ready, count, navigate, setId]);

  const reset = () => setReady(null);
  return (
    <LatihanSetupScreen
      view={{
        ...LATIHAN_SETUP,
        subjectId,
        chapterId,
        kind,
        count,
        chapters,
        generation: ready === null ? null : { ready, total: count },
      }}
      onSubjectChange={(id) => {
        setSubjectId(id);
        setChapterId(LATIHAN_CHAPTERS[id]?.[0]?.id ?? "");
        reset();
      }}
      onChapterChange={(id) => {
        setChapterId(id);
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
      onGenerate={() => {
        demo.addSet(demoSet(LATIHAN_SET, count, setId));
        demo.setAnswers(setId, {});
        setReady(0);
      }}
      startHref={`/latihan/${setId}`}
    />
  );
}

export function QuizRoute() {
  const { setId = "" } = useParams();
  const navigate = useNavigate();
  const demo = useDemo();
  const set = demo.sets[setId];
  // "Coba lagi soal yang salah" comes back here: keep the right answers, start at the first gap.
  const [answers, setAnswers] = useState<Answers>(() => {
    const saved = demo.answers[setId] ?? {};
    return Object.fromEntries(
      Object.entries(saved).filter(([id, a]) => {
        const q = set?.questions.find((x) => x.id === id);
        return q !== undefined && isCorrect(q, a);
      }),
    );
  });
  const [index, setIndex] = useState(() =>
    Math.max(0, set?.questions.findIndex((q) => (answers[q.id] ?? []).length === 0) ?? 0),
  );
  if (set === undefined) return <NotFound />;
  const question = set.questions[index];
  if (question === undefined) return <NotFound />;
  const chosen = answers[question.id] ?? [];
  return (
    <QuizScreen
      chapterTitle={set.chapterTitle}
      subjectId={set.subjectId}
      question={question}
      index={index}
      total={set.questions.length}
      chosen={chosen}
      onChoose={(optionId) =>
        setAnswers({ ...answers, [question.id]: toggleChoice(question, chosen, optionId) })
      }
      onPrev={() => setIndex((i) => Math.max(0, i - 1))}
      onNext={() => setIndex((i) => Math.min(set.questions.length - 1, i + 1))}
      onFinish={() => {
        demo.setAnswers(set.id, answers);
        demo.addStars(pointsFor(scoreSet(set, answers).correct));
        navigate(`/latihan/${set.id}/hasil`);
      }}
    />
  );
}

export function ResultRoute() {
  const { setId = "" } = useParams();
  const demo = useDemo();
  const set = demo.sets[setId];
  if (set === undefined) return <NotFound />;
  const saved = demo.answers[setId];
  // Opened directly (no quiz taken yet): show the demo answers so the page is never empty.
  const answers =
    saved !== undefined && Object.keys(saved).length > 0
      ? saved
      : set.id === LATIHAN_SET.id
        ? LATIHAN_SAMPLE_ANSWERS
        : {};
  return (
    <ResultScreen
      set={set}
      answers={answers}
      retryHref={`/latihan/${set.id}`}
      studyHref="/belajar/ipa/ipa-3"
    />
  );
}
