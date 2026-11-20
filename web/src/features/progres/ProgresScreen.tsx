import { type ReactNode, useState } from "react";
import {
  AppLink,
  BookIcon,
  ButtonLink,
  Card,
  ChatIcon,
  CheckSquareIcon,
  Mascot,
  subjectById,
} from "../../ui";
import { ActivitySection } from "./ActivitySection";
import { MasterySection } from "./MasterySection";
import { NextSteps } from "./NextSteps";
import { SummarySection } from "./SummarySection";
import type { ActivityRange, Period, ProgresView, SessionKind, SessionRow } from "./types";

export type ProgresScreenProps = { readonly view: ProgresView };

const KIND_ICON: Readonly<Record<SessionKind, ReactNode>> = {
  practice: <CheckSquareIcon />,
  lesson: <BookIcon />,
  ask: <ChatIcon />,
};

function RecentSessions({ sessions }: { readonly sessions: readonly SessionRow[] }) {
  return (
    <Card as="section" aria-labelledby="sesi-title" className="flex flex-col gap-3">
      <h2 id="sesi-title" className="m-0 font-display text-[22px] font-semibold">
        Sesi terakhir
      </h2>
      {sessions.length === 0 ? (
        <p className="m-0 text-[16px] font-bold text-ink-muted">Belum ada sesi.</p>
      ) : (
        <ul className="m-0 flex list-none flex-col gap-2 p-0">
          {sessions.map((s) => {
            const subject = s.subjectId === null ? undefined : subjectById(s.subjectId);
            return (
              <li key={`${s.kind}-${s.id}`}>
                <AppLink
                  href={s.href}
                  className="flex min-h-14 items-center gap-3 rounded-[16px] border-2 border-line px-3.5 py-2 text-ink hover:border-line-sky hover:bg-page"
                >
                  <span className="flex size-10 shrink-0 items-center justify-center rounded-full bg-primary-soft text-primary-ink">
                    {KIND_ICON[s.kind]}
                  </span>
                  <span className="flex min-w-0 flex-1 flex-col">
                    <span className="truncate text-[16px] font-extrabold">{s.title}</span>
                    <span className="text-[14px] font-bold text-ink-muted">
                      {s.kindLabel}
                      {subject === undefined ? "" : ` · ${subject.shortName}`} · {s.detail}
                    </span>
                  </span>
                  <span className="shrink-0 text-[14px] font-bold text-ink-faint">{s.when}</span>
                </AppLink>
              </li>
            );
          })}
        </ul>
      )}
    </Card>
  );
}

function EmptyProgres() {
  return (
    <Card
      as="section"
      aria-labelledby="progres-kosong"
      className="flex flex-col items-center gap-4 py-8 text-center"
    >
      <Mascot mood="happy" size={120} animate />
      <h1 id="progres-kosong" className="m-0 font-display text-[28px] font-semibold">
        Petualanganmu baru dimulai!
      </h1>
      <p className="m-0 max-w-[480px] text-[17px] font-bold text-ink-muted">
        Di sini nanti kamu bisa melihat bintang, runtunan belajar, dan topik yang sudah kamu kuasai.
        Yuk mulai dengan satu latihan atau satu bab.
      </p>
      <div className="flex flex-wrap justify-center gap-3">
        <ButtonLink href="/latihan" variant="success">
          Mulai latihan
        </ButtonLink>
        <ButtonLink href="/belajar" variant="secondary">
          Pilih bab
        </ButtonLink>
      </div>
    </Card>
  );
}

/** Progres: summary, what to do next, mastery per subject, activity and recent sessions. */
export function ProgresScreen({ view }: ProgresScreenProps) {
  const [period, setPeriod] = useState<Period>("week");
  const [range, setRange] = useState<ActivityRange>(7);
  if (view.isEmpty) return <EmptyProgres />;
  return (
    <div className="flex flex-col gap-6">
      <SummarySection view={view} period={period} onPeriodChange={setPeriod} />
      <NextSteps practiceAgain={view.practiceAgain} continueLessons={view.continueLessons} />
      <MasterySection subjects={view.subjects} />
      <div className="grid grid-cols-1 items-start gap-6 lg:grid-cols-[3fr_2fr]">
        <ActivitySection views={view.activity} range={range} onRangeChange={setRange} />
        <RecentSessions sessions={view.recent} />
      </div>
    </div>
  );
}
