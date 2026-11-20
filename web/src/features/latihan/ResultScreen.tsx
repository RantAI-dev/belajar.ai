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
  RichText,
  StarRating,
  starsForScore,
} from "../../ui";
import { resultMood } from "./resultMood";
import { resultHeadline } from "./scoring";
import type { QuizResult, ReviewItem as Review } from "./types";

export type ResultScreenProps = {
  readonly result: QuizResult;
  /** A fresh set on the same topic. */
  readonly againHref: string;
  readonly studyHref: string;
};

function ReviewItem({
  item,
  no,
  open,
  onToggle,
}: {
  readonly item: Review;
  readonly no: number;
  readonly open: boolean;
  readonly onToggle: () => void;
}) {
  const q = item.question;
  const ok = item.isCorrect;
  const panelId = `review-${q.id}`;
  return (
    <article
      className={cx(
        "overflow-hidden rounded-[22px] border-2 bg-surface",
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
              "flex size-9 shrink-0 items-center justify-center rounded-full font-display text-[18px] font-semibold text-on-fill",
              ok ? "bg-success-solid" : "bg-oops-solid",
            )}
          >
            {ok ? <CheckIcon size={20} /> : <BangIcon size={20} />}
          </span>
          <span className="flex-1 text-[17px] font-extrabold leading-snug">
            <span className="sr-only">{ok ? "Benar. " : "Belum tepat. "}</span>
            {no}. <RichText text={q.prompt} />
          </span>
          <span className="text-[15px] font-extrabold text-primary-ink">
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
                state={choiceState(o.id, item.chosen, item.correct)}
                locked
              />
            ))}
          </div>
          <div className="flex flex-col gap-1.5 rounded-[16px] bg-page px-4 py-3.5">
            <span
              className={cx(
                "text-[17px] font-extrabold",
                ok ? "text-success-ink" : "text-oops-ink",
              )}
            >
              {ok ? "Benar!" : "Hampir benar!"}
            </span>
            <p className="m-0 text-[16px] font-semibold leading-relaxed">
              <RichText text={item.explanation} />
            </p>
            {item.source !== null &&
              (item.source.href !== undefined ? (
                <a
                  href={item.source.href}
                  target="_blank"
                  rel="noopener"
                  className="inline-flex min-h-12 items-center self-start text-[15px] font-extrabold underline-offset-4 hover:underline"
                >
                  {item.source.label}
                  <span className="sr-only"> (di tab baru)</span>
                </a>
              ) : (
                <span className="text-[15px] font-bold text-ink-muted">{item.source.label}</span>
              ))}
          </div>
        </div>
      )}
    </article>
  );
}

export function ResultScreen({ result, againHref, studyHref }: ResultScreenProps) {
  const { correct, total } = result;
  const mood = resultMood(correct, total);
  const firstWrong = result.items.find((i) => !i.isCorrect)?.question.id;
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
        className="on-dark flex flex-wrap items-center gap-6 rounded-panel bg-celebrate p-5 text-on-fill sm:p-7"
      >
        <Mascot
          expression={mood.expression}
          size={120}
          backdrop="var(--color-celebrate-glow)"
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
          {mood.cheer !== null && (
            <p className="m-0 text-[17px] font-bold leading-snug">{mood.cheer}</p>
          )}
          <StarRating
            value={starsForScore(correct, total)}
            size={40}
            emptyColor="var(--color-celebrate-star-empty)"
          />
          <span className="inline-flex min-h-9 items-center gap-1.5 self-start rounded-[12px] bg-surface px-3.5 text-[16px] font-extrabold text-success-ink">
            {result.starsEarned > 0
              ? `+${result.starsEarned} bintang untukmu`
              : "Ayo coba lagi untuk dapat bintang"}
          </span>
        </div>
      </section>

      <div className="flex flex-wrap gap-3">
        <ButtonLink href={againHref} size="lg" className="flex-1 basis-[220px]">
          {correct < total ? "Latihan lagi topik ini" : "Latihan soal baru"}
        </ButtonLink>
        <ButtonLink href={studyHref} variant="secondary" size="lg" className="flex-1 basis-[220px]">
          Pelajari lagi bab ini
        </ButtonLink>
      </div>

      <section aria-labelledby="review-title" className="flex flex-col gap-3">
        <h2 id="review-title" className="m-0 font-display text-[24px] font-semibold">
          Yuk, lihat pembahasannya
        </h2>
        {result.items.map((item, i) => (
          <ReviewItem
            key={item.question.id}
            item={item}
            no={i + 1}
            open={open.has(item.question.id)}
            onToggle={() => toggle(item.question.id)}
          />
        ))}
      </section>
    </div>
  );
}
