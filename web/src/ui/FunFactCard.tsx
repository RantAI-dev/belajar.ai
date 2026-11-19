import { cx } from "./cx";
import { BulbIcon } from "./icons";

/** "Tahukah kamu?": a warm card with one fact from the student's book. */
export function FunFactCard({
  text,
  source,
  title = "Tahukah kamu?",
  className,
}: {
  readonly text: string;
  readonly source?: string;
  readonly title?: string;
  readonly className?: string;
}) {
  return (
    <aside
      aria-label={title}
      className={cx(
        "flex items-start gap-3.5 rounded-card border-2 border-warm-line bg-warm p-5",
        className,
      )}
    >
      <BulbIcon size={36} className="shrink-0" />
      <div className="flex min-w-0 flex-col gap-1.5 text-warm-ink">
        <span className="font-display text-[20px] font-semibold">{title}</span>
        <p className="m-0 text-[17px] font-bold leading-relaxed">{text}</p>
        {source !== undefined && <span className="text-[14px] font-bold">{source}</span>}
      </div>
    </aside>
  );
}
