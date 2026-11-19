import type { FormEvent } from "react";
import {
  Button,
  ButtonLink,
  Card,
  CheckIcon,
  cx,
  FunFactCard,
  Mascot,
  ProgressBar,
  SelectField,
  ToggleChip,
} from "../../ui";
import type { PracticeKind, SetupView } from "./types";

export type LatihanSetupScreenProps = {
  readonly view: SetupView;
  readonly onSubjectChange: (subjectId: string) => void;
  readonly onChapterChange: (chapterId: string) => void;
  readonly onKindChange: (kind: PracticeKind) => void;
  readonly onCountChange: (count: number) => void;
  readonly onGenerate: () => void;
  /** Where "start with the questions that are ready" goes. */
  readonly startHref: string;
};

const KINDS: ReadonlyArray<{
  readonly id: PracticeKind;
  readonly name: string;
  readonly desc: string;
}> = [
  { id: "single", name: "Pilihan ganda", desc: "Pilih 1 jawaban benar" },
  { id: "multiple", name: "Benar lebih dari satu", desc: "Bisa ada 2 jawaban benar" },
  { id: "story", name: "Soal cerita", desc: "Soal dengan cerita pendek" },
  { id: "mixed", name: "Campuran", desc: "Semua jenis dicampur" },
];

function Progress({ view, startHref }: Pick<LatihanSetupScreenProps, "view" | "startHref">) {
  const g = view.generation;
  const ready = g?.ready ?? 0;
  const total = g?.total ?? view.count;
  const finished = g !== null && ready >= total;
  const headline =
    g === null
      ? "Aku siap membuatkan soal!"
      : finished
        ? "Soalmu sudah siap!"
        : "Sebentar ya, soalmu sedang dibuat…";
  return (
    <Card className="flex flex-col gap-4">
      <div className="flex items-center gap-3.5">
        <Mascot
          mood={finished ? "celebrate" : g === null ? "happy" : "thinking"}
          size={72}
          animate={g !== null && !finished}
        />
        <div className="flex flex-col gap-1" aria-live="polite">
          <h2 className="m-0 font-display text-[24px] font-semibold leading-tight">{headline}</h2>
          <span className="text-[15px] font-bold text-ink-muted">
            {g === null
              ? "Pilih mapel dan jenis soal, lalu tekan Buatkan soal."
              : "Aku cek dulu setiap soal supaya jawabannya benar."}
          </span>
        </div>
      </div>
      {g !== null && (
        <>
          <div className="flex items-center gap-3">
            <ProgressBar
              value={ready}
              max={total}
              label="Soal yang sudah siap"
              valueText={`${ready} dari ${total} soal siap`}
              size="lg"
              className="flex-1"
            />
            <span aria-hidden="true" className="font-display text-[24px] font-semibold">
              {ready}/{total}
            </span>
          </div>
          {total <= 5 && (
            <div aria-hidden="true" className="flex gap-2">
              {Array.from({ length: total }, (_, i) => i + 1).map((slot) => (
                <span
                  key={slot}
                  className={cx(
                    "flex min-h-[52px] flex-1 items-center justify-center rounded-chip border-2 font-display text-[20px] font-semibold",
                    slot <= ready
                      ? "border-success bg-success-soft text-success-edge"
                      : "border-line-strong bg-page text-ink-faint",
                  )}
                >
                  {slot <= ready ? <CheckIcon size={22} /> : slot}
                </span>
              ))}
            </div>
          )}
          {ready > 0 && (
            <ButtonLink href={startHref} variant="success" size="lg" block>
              {finished ? "Mulai kerjakan" : `Mulai kerjakan ${ready} soal yang siap`}
            </ButtonLink>
          )}
        </>
      )}
    </Card>
  );
}

export function LatihanSetupScreen(props: LatihanSetupScreenProps) {
  const { view, onSubjectChange, onChapterChange, onKindChange, onCountChange, onGenerate } = props;
  const generating = view.generation !== null && view.generation.ready < view.generation.total;
  const submit = (e: FormEvent) => {
    e.preventDefault();
    onGenerate();
  };
  return (
    <div className="flex flex-wrap items-start gap-6">
      <Card
        as="form"
        onSubmit={submit}
        aria-labelledby="setup-title"
        className="flex min-w-0 flex-1 basis-[380px] flex-col gap-[18px]"
      >
        <h1 id="setup-title" className="m-0 font-display text-[28px] font-semibold">
          Siap latihan?
        </h1>
        <div className="flex flex-wrap gap-2.5">
          <SelectField
            label="Mapel"
            className="flex-1 basis-[160px]"
            value={view.subjectId}
            onValueChange={onSubjectChange}
            options={view.subjects.map((s) => ({ value: s.id, label: s.label }))}
          />
          <SelectField
            label="Bab"
            className="flex-[2] basis-[220px]"
            value={view.chapterId}
            onValueChange={onChapterChange}
            options={view.chapters.map((c) => ({ value: c.id, label: c.label }))}
          />
        </div>
        <fieldset className="m-0 flex min-w-0 flex-col gap-2 border-0 p-0">
          <legend className="mb-2 p-0 text-[15px] font-extrabold text-ink-muted">Jenis soal</legend>
          <div className="grid grid-cols-1 gap-2.5 min-[420px]:grid-cols-2">
            {KINDS.map((k) => (
              <ToggleChip
                key={k.id}
                shape="tile"
                pressed={view.kind === k.id}
                onPressedChange={() => onKindChange(k.id)}
                className="flex-col items-start gap-1.5"
              >
                <span className="text-[17px] font-extrabold">{k.name}</span>
                <span className="text-[14px] font-semibold leading-snug text-ink-muted">
                  {k.desc}
                </span>
              </ToggleChip>
            ))}
          </div>
        </fieldset>
        <fieldset className="m-0 flex min-w-0 flex-col gap-2 border-0 p-0">
          <legend className="mb-2 p-0 text-[15px] font-extrabold text-ink-muted">
            Berapa soal?
          </legend>
          <div className="flex gap-2.5">
            {view.counts.map((n) => (
              <ToggleChip
                key={n}
                shape="tile"
                look="solid"
                pressed={view.count === n}
                onPressedChange={() => onCountChange(n)}
                className="min-h-14 flex-1 justify-center font-display text-[24px] font-semibold"
              >
                {n}
                <span className="sr-only"> soal</span>
              </ToggleChip>
            ))}
          </div>
        </fieldset>
        <Button type="submit" size="lg" block loading={generating} loadingLabel="Sedang dibuat…">
          Buatkan soal
        </Button>
      </Card>
      <section
        aria-label="Pembuatan soal"
        className="flex min-w-0 flex-1 basis-[420px] flex-col gap-4"
      >
        <Progress view={view} startHref={props.startHref} />
        <FunFactCard text={view.funFact.text} source={view.funFact.source} />
      </section>
    </div>
  );
}
