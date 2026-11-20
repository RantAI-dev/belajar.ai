import { useState } from "react";
import {
  AppLink,
  ArrowLeftIcon,
  Button,
  ButtonLink,
  ErrorState,
  LoadingState,
  Mascot,
  paintFor,
  StarRating,
  Stepper,
  subjectById,
  ToggleChip,
} from "../../ui";
import { ChatThread, Composer, type TanyaMessage } from "../tanya";
import { LESSON_STEPS, QUIZ_TOTAL, quizHeadline, stepIndex } from "./lesson";
import type { LessonStep } from "./types";

export type LessonResult = {
  readonly correct: number;
  readonly total: number;
  /** Stars this quiz added to the student's total. */
  readonly starsEarned: number;
  /** The chapter's best rating, 0..3. */
  readonly stars: number;
};

export type LessonScreenProps = {
  readonly subjectId: string;
  readonly chapterTitle: string;
  readonly step: LessonStep;
  readonly messages: readonly TanyaMessage[];
  /** An answer is arriving. */
  readonly busy: boolean;
  readonly loading: boolean;
  readonly loadError?: { readonly message: string; readonly onRetry: () => void };
  readonly onSend: (text: string) => void;
  /** Moves explain → check → quiz. */
  readonly onNext: () => void;
  readonly moving: boolean;
  /** Completes the lesson with the number of quiz questions the student got right. */
  readonly onComplete: (correct: number) => void;
  readonly completing: boolean;
  /** A step change or completion failed. */
  readonly actionError: string | null;
  /** The quiz result just sent; null when the lesson was already done before this visit. */
  readonly result: LessonResult | null;
  /** The chapter's best stars so far. */
  readonly bestStars: number;
  readonly mapHref: string;
  readonly practiceHref: string;
};

function QuizScore({
  onComplete,
  completing,
}: Pick<LessonScreenProps, "onComplete" | "completing">) {
  const [correct, setCorrect] = useState<number | null>(null);
  return (
    <fieldset className="m-0 flex min-w-0 flex-col gap-3 rounded-card border-2 border-line bg-surface p-4 sm:p-5">
      <legend className="float-left mb-1 p-0 font-display text-[20px] font-semibold">
        Sudah kamu jawab? Berapa yang benar?
      </legend>
      <div className="clear-left flex gap-2.5">
        {Array.from({ length: QUIZ_TOTAL + 1 }, (_, n) => n).map((n) => (
          <ToggleChip
            key={n}
            shape="tile"
            look="solid"
            pressed={correct === n}
            onPressedChange={() => setCorrect(n)}
            className="min-h-14 flex-1 justify-center font-display text-[24px] font-semibold"
          >
            {n}
            <span className="sr-only"> benar dari {QUIZ_TOTAL}</span>
          </ToggleChip>
        ))}
      </div>
      <Button
        variant="success"
        size="lg"
        block
        disabled={correct === null}
        loading={completing}
        loadingLabel="Menghitung bintang…"
        onClick={() => {
          if (correct !== null) onComplete(correct);
        }}
      >
        Selesai, lihat bintangku
      </Button>
    </fieldset>
  );
}

function Celebration({ result, bestStars }: Pick<LessonScreenProps, "result" | "bestStars">) {
  return (
    <section
      aria-label="Hasil sesi"
      className="on-dark flex flex-wrap items-center gap-5 rounded-panel bg-celebrate p-5 text-on-fill sm:p-6"
    >
      <Mascot mood="celebrate" size={96} backdrop="var(--color-celebrate-glow)" inverse animate />
      <div className="flex min-w-0 flex-1 basis-[240px] flex-col gap-2">
        <h2 className="m-0 font-display text-[28px] font-semibold leading-tight">
          {result === null ? "Bab ini sudah selesai!" : quizHeadline(result.correct, result.total)}
        </h2>
        <StarRating
          value={result?.stars ?? bestStars}
          size={36}
          emptyColor="var(--color-celebrate-star-empty)"
        />
        {result !== null && (
          <span className="inline-flex min-h-9 items-center self-start rounded-[12px] bg-surface px-3.5 text-[16px] font-extrabold text-success-ink">
            {result.starsEarned > 0
              ? `+${result.starsEarned} bintang untukmu`
              : "Bintang terbaikmu tetap tersimpan"}
          </span>
        )}
      </div>
    </section>
  );
}

export function LessonScreen(p: LessonScreenProps) {
  const subject = subjectById(p.subjectId);
  const paint = paintFor(subject?.id);
  const at = stepIndex(p.step);
  const idle = !p.busy && !p.loading;
  return (
    <div className="mx-auto flex w-full max-w-[860px] flex-1 flex-col gap-[18px]">
      <div className="flex flex-col gap-3">
        <AppLink
          href={p.mapHref}
          className="inline-flex min-h-12 items-center gap-2 self-start rounded-chip pr-3 text-[16px] font-extrabold"
        >
          <ArrowLeftIcon size={20} />
          Peta {subject?.name ?? "belajar"}
        </AppLink>
        <h1 className="m-0 font-display text-[26px] font-semibold leading-tight sm:text-[30px]">
          {p.chapterTitle}
        </h1>
        <Stepper
          steps={LESSON_STEPS}
          current={at}
          color={paint.color}
          edge={paint.edge}
          ink={paint.ink}
          className="max-w-[520px]"
        />
      </div>

      {p.loadError !== undefined ? (
        <ErrorState message={p.loadError.message} onRetry={p.loadError.onRetry} />
      ) : p.loading ? (
        <LoadingState text="Sebentar, aku buka bukumu…" />
      ) : (
        <div className="flex flex-col gap-[18px]">
          <ChatThread messages={p.messages} />
          {p.step === "done" && <Celebration result={p.result} bestStars={p.bestStars} />}
        </div>
      )}

      {p.actionError !== null && (
        <p role="alert" className="m-0 text-[16px] font-bold text-oops-ink">
          {p.actionError}
        </p>
      )}
      {idle && p.loadError === undefined && (
        <div className="flex flex-wrap gap-3">
          {(p.step === "explain" || p.step === "check") && (
            <Button size="lg" block loading={p.moving} loadingLabel="Sebentar…" onClick={p.onNext}>
              {p.step === "explain" ? "Aku paham, lanjut" : "Lanjut ke kuis kecil"}
            </Button>
          )}
          {p.step === "quiz" && (
            <div className="w-full">
              <QuizScore onComplete={p.onComplete} completing={p.completing} />
            </div>
          )}
          {p.step === "done" && (
            <>
              <ButtonLink href={p.mapHref} size="lg" className="flex-1 basis-[220px]">
                Kembali ke peta
              </ButtonLink>
              <ButtonLink
                href={p.practiceHref}
                variant="secondary"
                size="lg"
                className="flex-1 basis-[220px]"
              >
                Latihan bab ini
              </ButtonLink>
            </>
          )}
        </div>
      )}
      <Composer
        busy={p.busy || p.loading}
        onSend={p.onSend}
        placeholder="Ada yang belum jelas? Tanya di sini…"
      />
    </div>
  );
}
