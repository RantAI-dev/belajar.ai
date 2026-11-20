import { type CSSProperties, type FormEvent, useState } from "react";
import {
  AppLink,
  Mascot,
  MissionCard,
  Stepper,
  SubjectCard,
  subjectById,
  subjectPaint,
} from "../../ui";
import { LESSON_STEPS } from "../belajar";
import type { BerandaView, ContinueLesson } from "./types";

export type BerandaScreenProps = {
  readonly view: BerandaView;
  readonly onAsk: (question: string) => void;
  /** The quick question is on its way to Tanya. */
  readonly asking?: boolean;
  /** Why the quick question could not be sent. */
  readonly askError?: string;
};

function ContinueCard({ lesson }: { readonly lesson: ContinueLesson }) {
  const subject = subjectById(lesson.subjectId);
  if (subject === undefined) return null;
  const paint = subjectPaint(subject);
  const steps = Array.from(
    { length: lesson.totalSteps },
    (_, i) => LESSON_STEPS[i] ?? `Langkah ${i + 1}`,
  );
  return (
    <AppLink
      href={lesson.href}
      className="tile flex min-w-0 flex-1 basis-[340px] flex-col gap-3.5 rounded-card border-2 p-5 text-ink sm:p-[22px]"
      style={
        {
          "--edge": paint.shadow,
          background: paint.tint,
          borderColor: paint.edge,
        } as CSSProperties
      }
    >
      <span
        className="inline-flex min-h-[30px] items-center self-start rounded-full px-3 text-[14px] font-extrabold text-on-fill"
        style={{ background: paint.solid }}
      >
        {subject.shortName}
      </span>
      <span className="font-display text-[24px] font-semibold leading-tight">
        Lanjutkan petualangan: {lesson.chapterTitle}
      </span>
      <Stepper
        steps={steps}
        current={lesson.step}
        color={paint.color}
        edge={paint.edge}
        ink={paint.ink}
      />
      <span
        className="inline-flex min-h-12 items-center self-start rounded-[14px] px-[22px] text-[19px] font-extrabold text-on-fill"
        style={{ background: paint.color, boxShadow: `0 4px 0 ${paint.solid}` }}
      >
        Lanjut belajar
      </span>
    </AppLink>
  );
}

export function BerandaScreen({ view, onAsk, asking = false, askError }: BerandaScreenProps) {
  const [question, setQuestion] = useState("");
  const submit = (e: FormEvent) => {
    e.preventDefault();
    if (question.trim().length > 0 && !asking) onAsk(question.trim());
  };
  return (
    <div className="flex flex-col gap-6">
      <section className="on-dark flex flex-wrap items-center gap-6 rounded-panel bg-hero p-5 text-on-fill sm:p-7">
        <Mascot
          mood="happy"
          size={132}
          backdrop="var(--color-hero-glow)"
          inverse
          animate
          className="max-sm:size-24"
        />
        <div className="flex min-w-0 flex-1 basis-[300px] flex-col gap-3.5">
          <h1 className="m-0 font-display text-[30px] font-semibold leading-tight sm:text-[36px]">
            Halo, {view.studentName}! Mau tahu apa hari ini?
          </h1>
          <form onSubmit={submit} className="flex flex-wrap gap-2.5">
            <label htmlFor="ask-home" className="sr-only">
              Pertanyaanmu
            </label>
            <input
              id="ask-home"
              type="text"
              value={question}
              onChange={(e) => setQuestion(e.target.value)}
              placeholder="Contoh: kenapa rel kereta diberi celah?"
              className="min-h-14 min-w-0 flex-1 basis-[240px] rounded-[16px] border-0 bg-surface px-5 text-[18px] font-semibold text-ink placeholder:text-ink-faint"
            />
            <button
              type="submit"
              disabled={asking}
              aria-busy={asking || undefined}
              className="pressable inline-flex min-h-14 cursor-pointer items-center gap-2 rounded-[16px] bg-sun px-[26px] text-[19px] font-extrabold text-sun-ink shadow-[0_4px_0_var(--color-sun-edge)] active:translate-y-[4px] active:shadow-none max-sm:w-full max-sm:justify-center"
            >
              Tanya
            </button>
          </form>
          {askError !== undefined && (
            <p
              role="alert"
              className="m-0 rounded-[12px] bg-surface px-3.5 py-2 text-[16px] font-bold text-oops-ink"
            >
              {askError}
            </p>
          )}
          <p className="m-0 text-[15px] font-semibold text-hero-ink-soft">
            Jawabanku selalu dari buku pelajaranmu, lengkap dengan halamannya.
          </p>
        </div>
      </section>

      <div className="flex flex-wrap items-stretch gap-5">
        <div className="min-w-0 flex-1 basis-[340px]">
          <MissionCard missions={view.missions} />
        </div>
        {view.continueLesson !== null && <ContinueCard lesson={view.continueLesson} />}
      </div>

      <section aria-labelledby="mapel-title" className="flex flex-col gap-3.5">
        <h2 id="mapel-title" className="m-0 font-display text-[22px] font-semibold">
          Pilih mapel
        </h2>
        <div className="grid grid-cols-2 gap-3 sm:grid-cols-[repeat(auto-fill,minmax(170px,1fr))] sm:gap-4">
          {view.subjects.map((s) => {
            const subject = subjectById(s.subjectId);
            if (subject === undefined) return null;
            return (
              <SubjectCard
                key={s.subjectId}
                subject={subject}
                href={s.href}
                progressLabel={s.progressLabel}
                progress={s.progress}
              />
            );
          })}
        </div>
      </section>
    </div>
  );
}
