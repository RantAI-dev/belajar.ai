import type { FormEvent } from "react";
import {
  Button,
  Card,
  CheckIcon,
  cx,
  FunFactCard,
  MASTERY_LABEL,
  Mascot,
  MasteryBadge,
  ProgressBar,
  SelectField,
  TextField,
  ToggleChip,
} from "../../ui";
import { generationText, slotsOf } from "./generation";
import type { PracticeKind, SetupView } from "./types";

export type LatihanSetupScreenProps = {
  readonly view: SetupView;
  readonly onSubjectChange: (subjectId: string) => void;
  readonly onChapterChange: (chapterId: string) => void;
  readonly onKindChange: (kind: PracticeKind) => void;
  readonly onTopicChange: (topic: string) => void;
  readonly onCountChange: (count: number) => void;
  readonly onGenerate: () => void;
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

function Progress({ view }: Pick<LatihanSetupScreenProps, "view">) {
  const g = view.generation;
  const text = generationText(g);
  const finished = g?.stage === "ready";
  return (
    <Card className="flex flex-col gap-4">
      <div className="flex items-center gap-3.5">
        <Mascot
          mood={finished ? "celebrate" : g === null ? "happy" : "thinking"}
          size={72}
          animate={g !== null && !finished}
        />
        <div className="flex flex-col gap-1" aria-live="polite">
          <h2 className="m-0 font-display text-[24px] font-semibold leading-tight">
            {text.headline}
          </h2>
          <span className="text-[15px] font-bold text-ink-muted">{text.detail}</span>
        </div>
      </div>
      {view.error !== null && (
        <p
          role="alert"
          className="m-0 rounded-[14px] bg-oops-soft px-4 py-3 text-[16px] font-bold text-oops-ink"
        >
          {view.error}
        </p>
      )}
      {g !== null && (
        <>
          <div className="flex items-center gap-3">
            <ProgressBar
              value={g.ready}
              max={g.total}
              label="Soal yang sudah siap"
              valueText={`${g.ready} dari ${g.total} soal siap`}
              size="lg"
              className="flex-1"
            />
            <span aria-hidden="true" className="font-display text-[24px] font-semibold">
              {g.ready}/{g.total}
            </span>
          </div>
          {slotsOf(g).length > 0 && (
            <div aria-hidden="true" className="flex gap-2">
              {slotsOf(g).map((slot) => (
                <span
                  key={slot.n}
                  className={cx(
                    "flex min-h-[52px] flex-1 items-center justify-center rounded-chip border-2 font-display text-[20px] font-semibold",
                    slot.done
                      ? "border-success bg-success-soft text-success-edge"
                      : "border-line-strong bg-page text-ink-faint",
                  )}
                >
                  {slot.done ? <CheckIcon size={22} /> : slot.n}
                </span>
              ))}
            </div>
          )}
        </>
      )}
    </Card>
  );
}

function Mastery({ rows }: { readonly rows: SetupView["mastery"] }) {
  if (rows.length === 0) return null;
  return (
    <Card as="section" aria-labelledby="mastery-title" className="flex flex-col gap-3">
      <h2 id="mastery-title" className="m-0 font-display text-[20px] font-semibold">
        Penguasaanmu
      </h2>
      <ul className="m-0 flex list-none flex-col gap-3 p-0">
        {rows.slice(0, 4).map((r) => (
          <li key={r.topic} className="flex flex-col gap-1.5">
            <span className="flex items-center justify-between gap-2 text-[15px] font-bold">
              <span className="min-w-0 truncate">{r.topic}</span>
              <span className="flex shrink-0 items-center gap-2">
                <MasteryBadge level={r.level} />
                <span className="text-ink-muted">{r.percent}%</span>
              </span>
            </span>
            <ProgressBar
              value={r.percent}
              max={100}
              label={`Penguasaan ${r.topic}`}
              valueText={`${r.percent}%, ${MASTERY_LABEL[r.level]}`}
            />
          </li>
        ))}
      </ul>
    </Card>
  );
}

export function LatihanSetupScreen(props: LatihanSetupScreenProps) {
  const { view, onSubjectChange, onChapterChange, onKindChange, onCountChange, onGenerate } = props;
  const generating = view.generation !== null && view.generation.stage !== "ready";
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
        <TextField
          label="Topik"
          hint="Boleh kamu ubah, misalnya jadi bagian bab yang mau dilatih."
          value={view.topic}
          maxLength={100}
          required
          onValueChange={props.onTopicChange}
        />
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
        <Button
          type="submit"
          size="lg"
          block
          loading={generating}
          loadingLabel="Sedang dibuat…"
          disabled={view.topic.trim() === ""}
        >
          Buatkan soal
        </Button>
      </Card>
      <section
        aria-label="Pembuatan soal"
        className="flex min-w-0 flex-1 basis-[420px] flex-col gap-4"
      >
        <Progress view={view} />
        <FunFactCard
          text={view.funFact.text}
          {...(view.funFact.source === undefined ? {} : { source: view.funFact.source })}
        />
        <Mastery rows={view.mastery} />
      </section>
    </div>
  );
}
