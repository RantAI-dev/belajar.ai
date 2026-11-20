import {
  ArrowLeftIcon,
  Button,
  Card,
  ChoiceButton,
  choiceState,
  letterAt,
  ProgressBar,
  paintFor,
  RichText,
  subjectById,
} from "../../ui";
import type { QuizQuestion } from "./types";

export type QuizScreenProps = {
  readonly topic: string;
  readonly subjectId: string;
  readonly question: QuizQuestion;
  /** 0-based */
  readonly index: number;
  readonly total: number;
  readonly chosen: readonly string[];
  readonly onChoose: (optionId: string) => void;
  readonly onPrev: () => void;
  readonly onNext: () => void;
  readonly onFinish: () => void;
  /** Every question has a choice. */
  readonly canFinish: boolean;
  readonly submitting: boolean;
  readonly submitError: string | null;
};

const HINT: Record<QuizQuestion["kind"], string> = {
  single: "Pilih 1 jawaban yang benar.",
  multiple: "Pilih semua jawaban yang benar. Bisa lebih dari satu!",
  story: "Baca ceritanya dulu, lalu pilih 1 jawaban.",
};

export function QuizScreen(p: QuizScreenProps) {
  const subject = subjectById(p.subjectId);
  const isLast = p.index >= p.total - 1;
  const q = p.question;
  return (
    <div className="mx-auto flex w-full max-w-[760px] flex-col gap-5">
      <div className="flex flex-col gap-2">
        <span className="text-[15px] font-extrabold" style={{ color: paintFor(subject?.id).ink }}>
          {subject?.name ?? "Latihan"} · {p.topic}
        </span>
        <div className="flex items-center gap-3">
          <ProgressBar
            value={p.index + 1}
            max={p.total}
            label="Soal ke"
            valueText={`Soal ${p.index + 1} dari ${p.total}`}
            size="lg"
            color={paintFor(subject?.id).color}
            className="flex-1"
          />
          <span aria-hidden="true" className="font-display text-[22px] font-semibold">
            {p.index + 1}/{p.total}
          </span>
        </div>
      </div>

      <Card as="section" aria-labelledby="soal-title" className="flex flex-col gap-4">
        <span className="text-[14px] font-extrabold text-ink-muted">Soal {p.index + 1}</span>
        {q.story !== undefined && (
          <div className="rounded-[18px] border-2 border-warm-line bg-warm px-4 py-3.5">
            <span className="text-[14px] font-extrabold text-warm-ink">Cerita</span>
            <p className="m-0 mt-1 text-[17px] font-semibold leading-relaxed text-warm-ink">
              <RichText text={q.story} />
            </p>
          </div>
        )}
        <h1 id="soal-title" className="m-0 font-display text-[24px] font-semibold leading-snug">
          <RichText text={q.prompt} />
        </h1>
        <p className="m-0 text-[15px] font-bold text-ink-muted">{HINT[q.kind]}</p>
        <fieldset className="m-0 flex min-w-0 flex-col gap-2.5 border-0 p-0">
          <legend className="sr-only">Pilihan jawaban</legend>
          {q.options.map((o, i) => (
            <ChoiceButton
              key={o.id}
              letter={letterAt(i)}
              text={o.text}
              multiple={q.kind === "multiple"}
              state={choiceState(o.id, p.chosen, null)}
              onChoose={() => p.onChoose(o.id)}
            />
          ))}
        </fieldset>
      </Card>

      {p.submitError !== null && (
        <p role="alert" className="m-0 text-[16px] font-bold text-oops-ink">
          {p.submitError}
        </p>
      )}
      <div className="flex gap-3">
        {p.index > 0 && (
          <Button
            variant="secondary"
            onClick={p.onPrev}
            icon={<ArrowLeftIcon size={20} />}
            aria-label="Soal sebelumnya"
          >
            <span className="max-sm:sr-only">Sebelumnya</span>
          </Button>
        )}
        {isLast ? (
          <Button
            variant="success"
            size="lg"
            className="flex-1"
            disabled={!p.canFinish}
            loading={p.submitting}
            loadingLabel="Aku periksa jawabanmu…"
            onClick={p.onFinish}
          >
            Selesai, lihat hasilku
          </Button>
        ) : (
          <Button size="lg" className="flex-1" disabled={p.chosen.length === 0} onClick={p.onNext}>
            Soal berikutnya
          </Button>
        )}
      </div>
    </div>
  );
}
