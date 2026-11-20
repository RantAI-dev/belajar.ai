import { Card, cx, ToggleChip } from "../../ui";
import type { ActivityRange, ActivityView } from "./types";

const RANGES: readonly ActivityRange[] = [7, 30];
const CHART_HEIGHT = 120;
const LABEL_SPACE = 28;
const TOP_SPACE = 22;

/** Questions answered per day as bars; a dot marks a day active without questions. */
function DayChart({ view }: { readonly view: ActivityView }) {
  const slot = view.range === 7 ? 56 : 20;
  const barWidth = view.range === 7 ? 30 : 12;
  const width = slot * view.bars.length;
  const captionId = `aktivitas-${view.range}-caption`;
  return (
    <figure className="m-0 flex flex-col gap-2">
      <svg
        role="img"
        aria-labelledby={captionId}
        viewBox={`0 0 ${width} ${TOP_SPACE + CHART_HEIGHT + LABEL_SPACE}`}
        className="h-auto w-full"
      >
        <title>{view.summary}</title>
        <line
          x1={0}
          x2={width}
          y1={TOP_SPACE + CHART_HEIGHT}
          y2={TOP_SPACE + CHART_HEIGHT}
          className="stroke-line-strong"
          strokeWidth={2}
        />
        {view.bars.map((b, i) => {
          const h = b.questions === 0 ? 0 : Math.max(4, (b.questions / view.max) * CHART_HEIGHT);
          const x = i * slot + (slot - barWidth) / 2;
          const mid = i * slot + slot / 2;
          const showLabel = view.range === 7 || i % 5 === 0 || b.isToday;
          return (
            <g key={b.on}>
              {h > 0 && (
                <rect
                  x={x}
                  y={TOP_SPACE + CHART_HEIGHT - h}
                  width={barWidth}
                  height={h}
                  rx={view.range === 7 ? 8 : 4}
                  className={b.isToday ? "fill-primary-strong" : "fill-primary"}
                />
              )}
              {h === 0 && b.isActive && (
                <circle
                  cx={mid}
                  cy={TOP_SPACE + CHART_HEIGHT - 8}
                  r={view.range === 7 ? 6 : 4}
                  className="fill-success"
                />
              )}
              {view.range === 7 && b.questions > 0 && (
                <text
                  x={mid}
                  y={TOP_SPACE + CHART_HEIGHT - h - 6}
                  textAnchor="middle"
                  className="fill-ink text-[15px] font-extrabold"
                >
                  {b.questions}
                </text>
              )}
              {showLabel && (
                <text
                  x={mid}
                  y={TOP_SPACE + CHART_HEIGHT + 20}
                  textAnchor="middle"
                  className={cx(
                    "text-[14px]",
                    b.isToday ? "fill-ink font-extrabold" : "fill-ink-muted font-bold",
                  )}
                >
                  {b.isToday ? "Hari ini" : b.shortLabel}
                </text>
              )}
            </g>
          );
        })}
      </svg>
      <figcaption id={captionId} className="text-[14px] font-semibold text-ink-muted">
        {view.summary} Batang biru: soal yang dijawab. Titik hijau: hari aktif tanpa soal.
      </figcaption>
    </figure>
  );
}

function DayTable({ view }: { readonly view: ActivityView }) {
  return (
    <details className="rounded-[16px] border-2 border-line bg-page px-4 py-2.5">
      <summary className="cursor-pointer text-[15px] font-extrabold text-primary">
        Lihat sebagai tabel
      </summary>
      <div className="mt-2 overflow-x-auto">
        <table className="w-full min-w-[480px] border-collapse text-left text-[14px]">
          <caption className="sr-only">Kegiatan {view.range} hari terakhir per hari</caption>
          <thead>
            <tr className="text-ink-muted">
              <th scope="col" className="py-1.5 pr-3 font-extrabold">
                Hari
              </th>
              <th scope="col" className="py-1.5 pr-3 font-extrabold">
                Aktif
              </th>
              <th scope="col" className="py-1.5 pr-3 font-extrabold">
                Soal (benar)
              </th>
              <th scope="col" className="py-1.5 pr-3 font-extrabold">
                Bab selesai
              </th>
              <th scope="col" className="py-1.5 font-extrabold">
                Pertanyaan
              </th>
            </tr>
          </thead>
          <tbody>
            {[...view.bars].reverse().map((b) => (
              <tr key={b.on} className="border-t-2 border-line">
                <th scope="row" className="py-1.5 pr-3 font-bold">
                  {b.isToday ? `Hari ini (${b.longLabel})` : b.longLabel}
                </th>
                <td className="py-1.5 pr-3 font-semibold">{b.isActive ? "Ya" : "Tidak"}</td>
                <td className="py-1.5 pr-3 font-semibold">
                  {b.questions} ({b.correct})
                </td>
                <td className="py-1.5 pr-3 font-semibold">{b.lessons}</td>
                <td className="py-1.5 font-semibold">{b.asked}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </details>
  );
}

/** Days active, questions and lessons over the last 7 or 30 days, as a chart and a table. */
export function ActivitySection({
  views,
  range,
  onRangeChange,
}: {
  readonly views: Readonly<Record<ActivityRange, ActivityView>>;
  readonly range: ActivityRange;
  readonly onRangeChange: (range: ActivityRange) => void;
}) {
  const view = views[range];
  const stats = [
    { label: "Hari aktif", value: `${view.daysActive} dari ${view.range}` },
    { label: "Soal dijawab", value: String(view.questions) },
    { label: "Bab selesai", value: String(view.lessons) },
    { label: "Pertanyaan", value: String(view.asked) },
  ];
  return (
    <Card as="section" aria-labelledby="aktivitas-title" className="flex flex-col gap-4">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <h2 id="aktivitas-title" className="m-0 font-display text-[22px] font-semibold">
          Aktivitas
        </h2>
        <fieldset className="m-0 flex gap-2 border-0 p-0">
          <legend className="sr-only">Rentang waktu</legend>
          {RANGES.map((r) => (
            <ToggleChip key={r} pressed={range === r} onPressedChange={() => onRangeChange(r)}>
              {r} hari
            </ToggleChip>
          ))}
        </fieldset>
      </div>
      <dl className="m-0 grid grid-cols-2 gap-3 sm:grid-cols-4">
        {stats.map((s) => (
          <div key={s.label} className="flex flex-col gap-0.5 rounded-[16px] bg-page px-3.5 py-2.5">
            <dt className="text-[14px] font-extrabold text-ink-muted">{s.label}</dt>
            <dd className="m-0 font-display text-[24px] font-semibold">{s.value}</dd>
          </div>
        ))}
      </dl>
      <DayChart view={view} />
      <DayTable view={view} />
    </Card>
  );
}
