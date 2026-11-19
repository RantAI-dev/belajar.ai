import { FlameIcon } from "./icons";

export function StreakBadge({ days }: { readonly days: number }) {
  return (
    <span
      title="Hari belajar berturut-turut"
      className="inline-flex min-h-10 items-center gap-1.5 rounded-[12px] bg-streak-soft px-3 text-[15px] font-extrabold text-streak-ink"
    >
      <FlameIcon />
      {days}
      <span className="max-sm:sr-only"> hari</span>
      <span className="sr-only"> belajar berturut-turut</span>
    </span>
  );
}
