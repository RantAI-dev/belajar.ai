import { useState } from "react";
import {
  BangIcon,
  ButtonLink,
  CheckIcon,
  ChoiceButton,
  choiceState,
  cx,
  letterAt,
  Mascot,
  StarRating,
  starsForScore,
} from "../../ui";
import { isCorrect, pointsFor, resultHeadline, scoreSet } from "./scoring";
import type { Answers, QuizQuestion, QuizSet } from "./types";

export type ResultScreenProps = {
  readonly set: QuizSet;
  readonly answers: Answers;
  readonly retryHref: string;
  readonly studyHref: string;
};

function ReviewItem({
  q,
  no,
  chosen,
  open,
  onToggle,
}: {
  readonly q: QuizQuestion;
  readonly no: number;
  readonly chosen: readonly string[];
  readonly open: boolean;
  readonly onToggle: () => void;
}) {
  const ok = isCorrect(q, chosen);
  const panelId = `review-${q.id}`;
  return (
    <article
      className={cx(
        "overflow-hidden rounded-[22px] border-2 bg-white",
        ok ? "border-line" : "border-oops-line",
      )}
    >
      <h3 className="m-0">
        <button
          type="button"
          aria-expanded={open}
          aria-controls={panelId}
          onClick={onToggle}
          className="flex min-h-16 w-full cursor-pointer items-center gap-3.5 border-0 bg-transparent px-[18px] py-3 text-left text-ink"
        >
          <span
            aria-hidden="true"
            className={cx(
              "flex size-9 shrink-0 items-center justify-center rounded-full font-display text-[18px] font-semibold text-white",
              ok ? "bg-success" : "bg-oops",
            )}
          >
            {ok ? <CheckIcon size={20} /> : <BangIcon size={20} />}
          </span>
          <span className="flex-1 text-[17px] font-extrabold leading-snug">
            <span className="sr-only">{ok ? "Benar. " : "Belum tepat. "}</span>
            {no}. {q.prompt}
          </span>
          <span className="text-[15px] font-extrabold text-primary">
            {open ? "Tutup" : "Lihat"}
          </span>
        </button>
      </h3>
      {open && (
        <div id={panelId} className="flex flex-col gap-3 px-[18px] pb-[18px] sm:pl-[68px]">
          <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
            {q.options.map((o, i) => (
              <ChoiceButton
                key={o.id}
                letter={letterAt(i)}
                text={o.text}
                multiple={q.kind === "multiple"}
                state={choiceState(o.id, chosen, q.answerIds)}
                locked
              />
            ))}
          </div>
          <div className="flex flex-col gap-1.5 rounded-[16px] bg-page px-4 py-3.5">
            <span
              className={cx(
                "text-[17px] font-extrabold",
                ok ? "text-success-edge" : "text-oops-ink",
              )}
            >
              {ok ? "Benar!" : "Hampir benar!"}
            </span>
            <p className="m-0 text-[16px] font-semibold leading-relaxed">{q.explanation}</p>
            <a
              href={q.source.href}
              target="_blank"
              rel="noopener"
              className="inline-flex min-h-12 items-center self-start text-[15px] font-extrabold underline-offset-4 hover:underline"
            >
              {q.source.label}
              <span className="sr-only"> (di tab baru)</span>
            </a>
          </div>
        </div>
      )}
    </article>
  );
}

export function ResultScreen({ set, answers, retryHref, studyHref }: ResultScreenProps) {
  const { correct, total } = scoreSet(set, answers);
  const firstWrong = set.questions.find((q) => !isCorrect(q, answers[q.id] ?? []))?.id;
  const [open, setOpen] = useState<ReadonlySet<string>>(
    new Set(firstWrong === undefined ? [] : [firstWrong]),
  );
  const toggle = (id: string) =>
    setOpen((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  return (
    <div className="mx-auto flex w-full max-w-[960px] flex-col gap-[22px]">
      <section
        aria-labelledby="hasil-title"
        className="on-dark flex flex-wrap items-center gap-6 rounded-panel bg-success p-5 text-white sm:p-7"
      >
        <Mascot
          mood="celebrate"
          size={120}
          backdrop="#2bb77c"
          inverse
          animate
          className="max-sm:size-24"
        />
        <div className="flex min-w-0 flex-1 basis-[260px] flex-col gap-2.5">
          <h1
            id="hasil-title"
            className="m-0 font-display text-[32px] font-semibold leading-tight sm:text-[40px]"
          >
            {resultHeadline(correct, total)}
          </h1>
          <StarRating value={starsForScore(correct, total)} size={40} emptyColor="#5fc596" />
          <span className="inline-flex min-h-9 items-center gap-1.5 self-start rounded-[12px] bg-white px-3.5 text-[16px] font-extrabold text-success-edge">
            +{pointsFor(correct)} bintang untukmu
          </span>
        </div>
      </section>

      <div className="flex flex-wrap gap-3">
        {correct < total && (
          <ButtonLink href={retryHref} size="lg" className="flex-1 basis-[220px]">
            Coba lagi soal yang salah
          </ButtonLink>
        )}
        <ButtonLink href={studyHref} variant="secondary" size="lg" className="flex-1 basis-[220px]">
          Pelajari lagi bab ini
        </ButtonLink>
      </div>

      <section aria-labelledby="review-title" className="flex flex-col gap-3">
        <h2 id="review-title" className="m-0 font-display text-[24px] font-semibold">
          Yuk, lihat pembahasannya
        </h2>
        {set.questions.map((q, i) => (
          <ReviewItem
            key={q.id}
            q={q}
            no={i + 1}
            chosen={answers[q.id] ?? []}
            open={open.has(q.id)}
            onToggle={() => toggle(q.id)}
          />
        ))}
      </section>
    </div>
  );
}
