import {
  AppLink,
  Card,
  MASTERY_LABEL,
  MasteryBadge,
  type MasteryLevel,
  ProgressBar,
  StarRating,
  SubjectIcon,
} from "../../ui";
import { TrendTag } from "./TrendTag";
import type { SubjectBlock } from "./types";

const LEGEND: ReadonlyArray<{ readonly level: MasteryLevel; readonly meaning: string }> = [
  { level: "not_started", meaning: "belum ada soal yang dijawab" },
  { level: "learning", meaning: "di bawah 60% benar" },
  { level: "almost", meaning: "60% benar atau lebih" },
  { level: "mastered", meaning: "80% benar atau lebih, dari minimal 5 soal" },
];

function Legend() {
  return (
    <details className="rounded-[16px] border-2 border-line bg-page px-4 py-2.5">
      <summary className="cursor-pointer text-[15px] font-extrabold text-primary-ink">
        Arti tingkat penguasaan
      </summary>
      <ul className="m-0 mt-2 flex list-none flex-col gap-2 p-0">
        {LEGEND.map((l) => (
          <li key={l.level} className="flex flex-wrap items-center gap-2 text-[15px] font-semibold">
            <MasteryBadge level={l.level} />
            {l.meaning}
          </li>
        ))}
      </ul>
    </details>
  );
}

function SubjectDetails({ subject }: { readonly subject: SubjectBlock }) {
  const titleId = `mapel-${subject.subjectId}`;
  return (
    <li>
      <details className="group rounded-card border-2 border-line bg-page">
        <summary className="flex cursor-pointer flex-wrap items-center gap-3 p-4">
          <SubjectIcon subjectId={subject.subjectId} size={44} framed />
          <span className="flex min-w-0 flex-1 basis-[200px] flex-col gap-1.5">
            <span className="flex flex-wrap items-center gap-2">
              <span id={titleId} className="font-display text-[20px] font-semibold">
                {subject.name}
              </span>
              <MasteryBadge level={subject.level} />
            </span>
            <ProgressBar
              value={subject.percent}
              max={100}
              label={`Penguasaan ${subject.name}`}
              valueText={`${subject.percent}% benar, ${MASTERY_LABEL[subject.level]}`}
              color="var(--color-primary)"
            />
            <span className="text-[14px] font-bold text-ink-muted">
              {subject.attempts === 0
                ? "Belum ada soal latihan"
                : `${subject.percent}% benar dari ${subject.attempts} soal`}{" "}
              · {subject.lessonsLabel}
            </span>
          </span>
          <span aria-hidden="true" className="text-[15px] font-extrabold text-primary-ink">
            <span className="group-open:hidden">Lihat</span>
            <span className="hidden group-open:inline">Tutup</span>
          </span>
        </summary>
        <div className="flex flex-col gap-4 px-4 pb-4">
          {subject.topics.length > 0 && (
            <div className="overflow-x-auto">
              <table className="w-full min-w-[520px] border-collapse text-left text-[15px]">
                <caption className="sr-only">Penguasaan topik {subject.name}</caption>
                <thead>
                  <tr className="text-[14px] text-ink-muted">
                    <th scope="col" className="py-2 pr-3 font-extrabold">
                      Topik
                    </th>
                    <th scope="col" className="py-2 pr-3 font-extrabold">
                      Tingkat
                    </th>
                    <th scope="col" className="py-2 pr-3 font-extrabold">
                      Benar
                    </th>
                    <th scope="col" className="py-2 font-extrabold">
                      Dibanding minggu lalu
                    </th>
                  </tr>
                </thead>
                <tbody>
                  {subject.topics.map((t) => (
                    <tr key={t.topic} className="border-t-2 border-line">
                      <th scope="row" className="py-2.5 pr-3 font-extrabold">
                        {t.topic}
                      </th>
                      <td className="py-2.5 pr-3">
                        <MasteryBadge level={t.level} />
                      </td>
                      <td className="py-2.5 pr-3 font-bold">
                        {t.percent}%{" "}
                        <span className="text-[13px] text-ink-faint">
                          ({t.correct}/{t.attempts})
                        </span>
                      </td>
                      <td className="py-2.5">
                        <span className="flex flex-col">
                          <TrendTag trend={t.trend} label={t.trendLabel} detail={t.trendDetail} />
                          <span
                            aria-hidden="true"
                            className="text-[13px] font-semibold text-ink-faint"
                          >
                            {t.trendDetail}
                          </span>
                        </span>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
          {subject.chapters.length > 0 && (
            <div className="flex flex-col gap-2">
              <h3 className="m-0 text-[16px] font-extrabold">Bab yang kamu pelajari</h3>
              <ul className="m-0 flex list-none flex-col gap-2 p-0">
                {subject.chapters.map((c) => (
                  <li key={c.lessonId}>
                    <AppLink
                      href={c.href}
                      className="flex min-h-12 flex-wrap items-center justify-between gap-2 rounded-[14px] border-2 border-line px-3.5 py-2 text-ink hover:border-line-sky"
                    >
                      <span className="font-bold">{c.title}</span>
                      <span className="flex items-center gap-2 text-[14px] font-bold text-ink-muted">
                        {c.status}
                        <StarRating value={c.stars} size={15} />
                      </span>
                    </AppLink>
                  </li>
                ))}
              </ul>
            </div>
          )}
        </div>
      </details>
    </li>
  );
}

/** Mastery per subject, opening into its topics (with this week's trend) and its chapters. */
export function MasterySection({ subjects }: { readonly subjects: readonly SubjectBlock[] }) {
  return (
    <Card as="section" aria-labelledby="penguasaan-title" className="flex flex-col gap-4">
      <div className="flex flex-col gap-1">
        <h2 id="penguasaan-title" className="m-0 font-display text-[22px] font-semibold">
          Penguasaan per mapel
        </h2>
        <p className="m-0 text-[15px] font-bold text-ink-muted">
          Buka mapel untuk melihat topik dan bab-nya.
        </p>
      </div>
      <Legend />
      {subjects.length === 0 ? (
        <p className="m-0 text-[16px] font-bold text-ink-muted">
          Belum ada mapel yang kamu latih. Semua topik masih {MASTERY_LABEL.not_started}.
        </p>
      ) : (
        <ul className="m-0 flex list-none flex-col gap-3 p-0">
          {subjects.map((s) => (
            <SubjectDetails key={s.subjectId} subject={s} />
          ))}
        </ul>
      )}
    </Card>
  );
}
