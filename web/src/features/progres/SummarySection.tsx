import type { ReactNode } from "react";
import {
  BookIcon,
  Card,
  ChartIcon,
  CheckIcon,
  CheckSquareIcon,
  FlameIcon,
  Mascot,
  StarIcon,
  ToggleChip,
} from "../../ui";
import type { Period, ProgresView, StatTile } from "./types";

const ICON: Readonly<Record<StatTile["id"], ReactNode>> = {
  streak: <FlameIcon size={26} />,
  stars: <StarIcon size={26} />,
  lessons: <BookIcon size={26} className="text-primary" />,
  questions: <CheckSquareIcon size={26} className="text-primary" />,
  accuracy: <ChartIcon size={26} className="text-success-ink" />,
  days: <CheckIcon size={26} className="text-success-ink" />,
};

const PERIODS: ReadonlyArray<{ readonly id: Period; readonly label: string }> = [
  { id: "week", label: "Minggu ini" },
  { id: "all", label: "Semua" },
];

export function SummarySection({
  view,
  period,
  onPeriodChange,
}: {
  readonly view: ProgresView;
  readonly period: Period;
  readonly onPeriodChange: (period: Period) => void;
}) {
  return (
    <section aria-labelledby="ringkasan-title" className="flex flex-col gap-4">
      <div className="flex flex-wrap items-center gap-4 rounded-panel border-2 border-line bg-primary-soft p-5 sm:p-6">
        <Mascot mood="celebrate" size={96} className="max-sm:size-20" />
        <div className="flex min-w-0 flex-1 basis-[260px] flex-col gap-1.5">
          <h1 className="m-0 font-display text-[30px] font-semibold leading-tight">
            Progres belajarmu
          </h1>
          <p className="m-0 text-[17px] font-bold text-ink-muted" aria-live="polite">
            {view.cheer[period]}
          </p>
        </div>
      </div>
      <div className="flex flex-wrap items-center justify-between gap-3">
        <h2 id="ringkasan-title" className="m-0 font-display text-[22px] font-semibold">
          Ringkasan{" "}
          <span className="text-[16px] font-bold text-ink-muted">
            {period === "week" ? `(${view.weekLabel})` : "(sejak awal)"}
          </span>
        </h2>
        <fieldset className="m-0 flex gap-2 border-0 p-0">
          <legend className="sr-only">Tampilkan ringkasan untuk</legend>
          {PERIODS.map((p) => (
            <ToggleChip
              key={p.id}
              pressed={period === p.id}
              onPressedChange={() => onPeriodChange(p.id)}
            >
              {p.label}
            </ToggleChip>
          ))}
        </fieldset>
      </div>
      <ul className="m-0 grid list-none grid-cols-2 gap-3 p-0 sm:grid-cols-3 lg:grid-cols-6">
        {view.summary[period].map((t) => (
          <li key={t.id}>
            <Card className="flex h-full flex-col gap-1.5" padded={false}>
              <div className="flex flex-col gap-1.5 p-4">
                <span className="flex items-center gap-2 text-[15px] font-extrabold text-ink-muted">
                  {ICON[t.id]}
                  {t.label}
                </span>
                <span className="font-display text-[28px] font-semibold leading-none">
                  {t.value}
                </span>
                <span className="text-[14px] font-semibold text-ink-faint">{t.hint}</span>
              </div>
            </Card>
          </li>
        ))}
      </ul>
    </section>
  );
}
