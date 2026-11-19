import { useState } from "react";
import {
  AppLink,
  ArrowLeftIcon,
  Button,
  ButtonLink,
  ChatBubble,
  ChoiceButton,
  choiceState,
  letterAt,
  Mascot,
  PagePreview,
  SourceChip,
  StarRating,
  Stepper,
  subjectById,
} from "../../ui";
import { quizScore, SESSION_STEPS, type SessionState, stepIndex } from "./session";
import type { SessionContent, SessionQuestion } from "./types";

export type ChapterSessionScreenProps = {
  readonly content: SessionContent;
  readonly state: SessionState;
  readonly onUnderstood: () => void;
  readonly onAnswer: (questionId: string, optionId: string) => void;
  readonly onNext: () => void;
  readonly mapHref: string;
  readonly practiceHref: string;
};

function QuestionTurn({
  question,
  chosen,
  onAnswer,
  label,
}: {
  readonly question: SessionQuestion;
  readonly chosen: string | undefined;
  readonly onAnswer: (optionId: string) => void;
  readonly label: string;
}) {
  const revealed = chosen !== undefined;
  const right = chosen === question.answerId;
  const chosenText = question.options.find((o) => o.id === chosen)?.text;
  return (
    <>
      <ChatBubble from="mascot" mood="thinking">
        <span className="text-[14px] font-extrabold text-ink-muted">{label}</span>
        <p className="m-0 text-[18px] font-bold leading-snug">{question.prompt}</p>
        <fieldset className="m-0 flex min-w-0 flex-col gap-2 border-0 p-0">
          <legend className="sr-only">Pilihan jawaban</legend>
          {question.options.map((o, i) => (
            <ChoiceButton
              key={o.id}
              letter={letterAt(i)}
              text={o.text}
              state={choiceState(
                o.id,
                chosen === undefined ? [] : [chosen],
                revealed ? [question.answerId] : null,
              )}
              locked={revealed}
              onChoose={() => onAnswer(o.id)}
            />
          ))}
        </fieldset>
      </ChatBubble>
      {revealed && (
        <>
          <ChatBubble from="student">{chosenText}</ChatBubble>
          <ChatBubble from="mascot" mood={right ? "celebrate" : "neutral"}>
            <span
              className={
                right
                  ? "text-[18px] font-extrabold text-success-edge"
                  : "text-[18px] font-extrabold text-oops-ink"
              }
            >
              {right ? "Benar! Hebat!" : "Hampir benar!"}
            </span>
            <p className="m-0 text-[17px] font-semibold leading-relaxed">{question.explanation}</p>
          </ChatBubble>
        </>
      )}
    </>
  );
}

export function ChapterSessionScreen({
  content,
  state,
  onUnderstood,
  onAnswer,
  onNext,
  mapHref,
  practiceHref,
}: ChapterSessionScreenProps) {
  const [showPage, setShowPage] = useState(false);
  const subject = subjectById(content.subjectId);
  const color = subject?.color ?? "#0069a8";
  const at = stepIndex(state);
  const quizShown =
    state.step === "done"
      ? content.quiz
      : content.quiz.slice(0, state.step === "quiz" ? state.quizIndex + 1 : 0);
  const current =
    state.step === "check"
      ? content.check
      : state.step === "quiz"
        ? content.quiz[state.quizIndex]
        : undefined;
  const canNext = current !== undefined && state.answers[current.id] !== undefined;
  const isLastQuiz = state.step === "quiz" && state.quizIndex >= content.quiz.length - 1;
  const score = quizScore(content, state);

  return (
    <div className="mx-auto flex w-full max-w-[860px] flex-col gap-[18px]">
      <div className="flex flex-col gap-3">
        <AppLink
          href={mapHref}
          className="inline-flex min-h-12 items-center gap-2 self-start rounded-chip pr-3 text-[16px] font-extrabold"
        >
          <ArrowLeftIcon size={20} />
          Peta {subject?.name ?? "belajar"}
        </AppLink>
        <h1 className="m-0 font-display text-[26px] font-semibold leading-tight sm:text-[30px]">
          {content.chapterTitle}
        </h1>
        <Stepper
          steps={SESSION_STEPS}
          current={at}
          color={color}
          edge={subject?.edge ?? "#c4dcf2"}
          ink={subject?.ink ?? "#00598a"}
          className="max-w-[520px]"
        />
      </div>

      <div className="flex flex-col gap-[18px]" aria-live="polite">
        <ChatBubble from="mascot" mood="happy">
          {content.explanation.map((p) => (
            <p key={p} className="m-0 text-[18px] font-semibold leading-[1.65]">
              {p}
            </p>
          ))}
          <div className="flex flex-col gap-2">
            <span className="text-[14px] font-extrabold text-ink-muted">
              Aku ambil dari buku ini:
            </span>
            <div>
              <SourceChip
                label={content.source.label}
                pressed={showPage}
                onPressedChange={setShowPage}
                color={color}
              />
            </div>
            {showPage && (
              <PagePreview
                title={content.source.title}
                fullPageHref={content.source.fullPageHref}
              />
            )}
          </div>
        </ChatBubble>

        {at >= 1 && <ChatBubble from="student">Aku paham, lanjut!</ChatBubble>}
        {at >= 1 && (
          <QuestionTurn
            question={content.check}
            chosen={state.answers[content.check.id]}
            onAnswer={(optionId) => onAnswer(content.check.id, optionId)}
            label="Cek paham"
          />
        )}
        {quizShown.map((q, i) => (
          <QuestionTurn
            key={q.id}
            question={q}
            chosen={state.answers[q.id]}
            onAnswer={(optionId) => onAnswer(q.id, optionId)}
            label={`Kuis kecil · soal ${i + 1} dari ${content.quiz.length}`}
          />
        ))}

        {state.step === "done" && (
          <section
            aria-label="Hasil sesi"
            className="on-dark flex flex-wrap items-center gap-5 rounded-panel bg-success p-5 text-white sm:p-6"
          >
            <Mascot mood="celebrate" size={96} backdrop="#2bb77c" inverse />
            <div className="flex min-w-0 flex-1 basis-[240px] flex-col gap-2">
              <h2 className="m-0 font-display text-[28px] font-semibold leading-tight">
                {score.correct === score.total
                  ? "Hebat! Semua benar!"
                  : `Kerja bagus! ${score.correct} dari ${score.total} benar`}
              </h2>
              <StarRating value={score.stars} size={36} emptyColor="#5fc596" />
            </div>
          </section>
        )}
      </div>

      <div className="sticky bottom-[calc(76px+env(safe-area-inset-bottom))] flex flex-wrap gap-3 bg-[linear-gradient(to_top,#f3f9ff_75%,rgb(243_249_255/0))] pt-3 pb-3 md:bottom-0 md:pb-6">
        {state.step === "explain" && (
          <Button size="lg" block onClick={onUnderstood}>
            Aku paham, lanjut
          </Button>
        )}
        {current !== undefined && (
          <Button size="lg" block disabled={!canNext} onClick={onNext}>
            {state.step === "check"
              ? "Lanjut ke kuis kecil"
              : isLastQuiz
                ? "Lihat hasilku"
                : "Soal berikutnya"}
          </Button>
        )}
        {state.step === "done" && (
          <>
            <ButtonLink href={mapHref} size="lg" className="flex-1 basis-[220px]">
              Kembali ke peta
            </ButtonLink>
            <ButtonLink
              href={practiceHref}
              variant="secondary"
              size="lg"
              className="flex-1 basis-[220px]"
            >
              Latihan bab ini
            </ButtonLink>
          </>
        )}
      </div>
    </div>
  );
}
