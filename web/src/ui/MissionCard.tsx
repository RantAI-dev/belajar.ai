import { Card } from "./Card";
import { CheckIcon, StarIcon } from "./icons";
import { AppLink } from "./link";

export type Mission = {
  readonly id: string;
  readonly text: string;
  readonly done: boolean;
  readonly stars: number;
  readonly href: string;
};

/** "Misi hari ini": a short list of today's goals with the stars each one earns. */
export function MissionCard({
  title = "Misi hari ini",
  missions,
}: {
  readonly title?: string;
  readonly missions: readonly Mission[];
}) {
  const done = missions.filter((m) => m.done).length;
  return (
    <Card as="section" aria-labelledby="misi-title" className="flex min-w-0 flex-col gap-3.5">
      <div className="flex items-center justify-between gap-2.5">
        <h2 id="misi-title" className="m-0 font-display text-[22px] font-semibold">
          {title}
        </h2>
        <span className="text-[15px] font-extrabold text-primary-ink">
          {done}/{missions.length} selesai
        </span>
      </div>
      <ul className="m-0 flex list-none flex-col gap-2.5 p-0">
        {missions.map((m) => (
          <li key={m.id}>
            <AppLink
              href={m.href}
              className={`flex min-h-[60px] items-center gap-3.5 rounded-[16px] px-3.5 text-ink ${m.done ? "bg-success-soft" : "bg-surface-alt hover:bg-primary-soft"}`}
            >
              <span
                aria-hidden="true"
                className={`flex size-8 shrink-0 items-center justify-center rounded-full border-[3px] ${m.done ? "border-success bg-success text-on-fill" : "border-line-dashed bg-surface"}`}
              >
                {m.done && <CheckIcon size={16} />}
              </span>
              <span className="flex-1 text-[17px] font-bold">
                {m.text}
                {m.done && <span className="sr-only"> (selesai)</span>}
              </span>
              <span className="inline-flex items-center gap-1 font-extrabold text-star-ink">
                <StarIcon size={16} />+{m.stars}
                <span className="sr-only"> bintang</span>
              </span>
            </AppLink>
          </li>
        ))}
      </ul>
    </Card>
  );
}
