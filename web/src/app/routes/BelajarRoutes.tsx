import { useEffect, useMemo, useReducer, useRef, useState } from "react";
import { useParams } from "react-router";
import {
  BELAJAR_SESSION,
  BELAJAR_SUBJECTS,
  BelajarHomeScreen,
  ChapterMapScreen,
  ChapterSessionScreen,
  chaptersOf,
  quizScore,
  sessionReducer,
  startSession,
} from "../../features/belajar";
import { isSubjectId, SUBJECTS } from "../../ui";
import { useDemo } from "../demo";
import { NotFound } from "./NotFound";

export function BelajarHomeRoute() {
  return <BelajarHomeScreen subjects={BELAJAR_SUBJECTS} />;
}

export function ChapterMapRoute() {
  const { subjectId = "" } = useParams();
  const [selected, setSelected] = useState<string | null>(null);
  if (!isSubjectId(subjectId)) return <NotFound />;
  return (
    <ChapterMapScreen
      key={subjectId}
      view={{ subjectId, subjectIds: SUBJECTS.map((s) => s.id), chapters: chaptersOf(subjectId) }}
      selectedChapterId={selected}
      onSelectChapter={setSelected}
      subjectHref={(id) => `/belajar/${id}`}
      chapterHref={(id) => `/belajar/${subjectId}/${id}`}
    />
  );
}

export function ChapterSessionRoute() {
  const { subjectId = "", chapterId = "" } = useParams();
  const chapter = chaptersOf(subjectId).find((c) => c.id === chapterId);
  if (chapter === undefined) return <NotFound />;
  return (
    <Session key={chapterId} subjectId={subjectId} chapterId={chapterId} title={chapter.title} />
  );
}

function Session({
  subjectId,
  chapterId,
  title,
}: {
  readonly subjectId: string;
  readonly chapterId: string;
  readonly title: string;
}) {
  const demo = useDemo();
  // Every chapter plays the IPA demo lesson until the API supplies real content.
  const content = useMemo(
    () => ({ ...BELAJAR_SESSION, subjectId, chapterId, chapterTitle: title }),
    [subjectId, chapterId, title],
  );
  const reducer = useMemo(() => sessionReducer(content), [content]);
  const [state, dispatch] = useReducer(reducer, undefined, startSession);
  const rewarded = useRef(false);
  useEffect(() => {
    if (state.step === "done" && !rewarded.current) {
      rewarded.current = true;
      demo.addStars(quizScore(content, state).correct * 5);
    }
  }, [state, content, demo]);
  return (
    <ChapterSessionScreen
      content={content}
      state={state}
      onUnderstood={() => dispatch({ type: "understood" })}
      onAnswer={(questionId, optionId) => dispatch({ type: "answer", questionId, optionId })}
      onNext={() => dispatch({ type: "next" })}
      mapHref={`/belajar/${subjectId}`}
      practiceHref="/latihan"
    />
  );
}
