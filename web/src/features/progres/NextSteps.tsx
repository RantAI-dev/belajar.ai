import {
  ArrowRightIcon,
  ButtonLink,
  Card,
  MasteryBadge,
  StarRating,
  SubjectIcon,
  subjectById,
} from "../../ui";
import { TrendTag } from "./TrendTag";
import type { ChapterRow, TopicRow } from "./types";

function PracticeAgain({ topics }: { readonly topics: readonly TopicRow[] }) {
  return (
    <Card as="section" aria-labelledby="latih-lagi-title" className="flex flex-col gap-3">
      <div className="flex flex-col gap-1">
        <h2 id="latih-lagi-title" className="m-0 font-display text-[22px] font-semibold">
          Latih lagi
        </h2>
        <p className="m-0 text-[15px] font-bold text-ink-muted">
          {topics.length === 0
            ? "Belum ada topik yang perlu diulang. Coba topik baru di Latihan, yuk!"
            : "Topik ini akan makin mudah kalau kamu latih sekali lagi."}
        </p>
      </div>
      {topics.length > 0 && (
        <ul className="m-0 flex list-none flex-col gap-3 p-0">
          {topics.map((t) => {
            const subject = subjectById(t.subjectId);
            return (
              <li
                key={`${t.subjectId}/${t.topic}`}
                className="flex flex-wrap items-center gap-3 rounded-[18px] border-2 border-line bg-page p-3"
              >
                <SubjectIcon subjectId={t.subjectId} size={40} framed />
                <div className="flex min-w-0 flex-1 basis-[180px] flex-col gap-1">
                  <span className="text-[17px] font-extrabold leading-snug">{t.topic}</span>
                  <span className="flex flex-wrap items-center gap-2 text-[14px] font-bold text-ink-muted">
                    {subject?.name ?? t.subjectId} · {t.percent}% benar
                    <MasteryBadge level={t.level} />
                    <TrendTag trend={t.trend} label={t.trendLabel} detail={t.trendDetail} />
                  </span>
                  <span className="text-[14px] font-semibold text-ink-faint">
                    {t.encouragement}
                  </span>
                </div>
                <ButtonLink
                  href={t.practiceHref}
                  variant="success"
                  aria-label={`Latih lagi ${t.topic}`}
                  className="max-sm:w-full"
                >
                  Latih lagi
                </ButtonLink>
              </li>
            );
          })}
        </ul>
      )}
    </Card>
  );
}

function ContinueLessons({ chapters }: { readonly chapters: readonly ChapterRow[] }) {
  return (
    <Card as="section" aria-labelledby="lanjutkan-title" className="flex flex-col gap-3">
      <div className="flex flex-col gap-1">
        <h2 id="lanjutkan-title" className="m-0 font-display text-[22px] font-semibold">
          Lanjutkan
        </h2>
        <p className="m-0 text-[15px] font-bold text-ink-muted">
          {chapters.length === 0
            ? "Semua bab yang kamu mulai sudah selesai. Mantap!"
            : "Bab yang belum selesai menunggumu."}
        </p>
      </div>
      {chapters.length > 0 && (
        <ul className="m-0 flex list-none flex-col gap-2.5 p-0">
          {chapters.map((c) => (
            <li key={c.lessonId}>
              <ButtonLink
                href={c.href}
                variant="secondary"
                block
                className="justify-between text-left"
                icon={<SubjectIcon subjectId={c.subjectId} size={32} />}
              >
                <span className="flex w-full items-center justify-between gap-3">
                  <span className="flex min-w-0 flex-col">
                    <span className="truncate text-[17px]">{c.title}</span>
                    <span className="flex items-center gap-2 text-[14px] font-bold text-ink-muted">
                      {c.status}
                      <StarRating value={c.stars} size={14} />
                    </span>
                  </span>
                  <ArrowRightIcon />
                </span>
              </ButtonLink>
            </li>
          ))}
        </ul>
      )}
    </Card>
  );
}

/** What to do next: the weakest topics to practise and the chapters left half-way. */
export function NextSteps({
  practiceAgain,
  continueLessons,
}: {
  readonly practiceAgain: readonly TopicRow[];
  readonly continueLessons: readonly ChapterRow[];
}) {
  return (
    <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
      <PracticeAgain topics={practiceAgain} />
      <ContinueLessons chapters={continueLessons} />
    </div>
  );
}
