import type { CSSProperties } from "react";
import { AppLink } from "./link";
import { ProgressBar } from "./ProgressBar";
import { SubjectIcon } from "./SubjectIcon";
import type { Subject } from "./subjects";

export type SubjectCardProps = {
  readonly subject: Subject;
  readonly href: string;
  /** e.g. "Bab 3 dari 5" or "Belum mulai". */
  readonly progressLabel: string;
  /** 0..100 */
  readonly progress: number;
};

/** A subject tile that lifts on hover; the icon wiggles. */
export function SubjectCard({ subject, href, progressLabel, progress }: SubjectCardProps) {
  const style = {
    "--edge": subject.shadow,
    background: subject.tint,
    borderColor: subject.edge,
  } as CSSProperties;
  return (
    <AppLink
      href={href}
      className="tile flex flex-col gap-2.5 rounded-[22px] border-2 p-[18px] text-ink"
      style={style}
    >
      <SubjectIcon subjectId={subject.id} framed />
      <span className="text-[18px] font-extrabold leading-tight">{subject.name}</span>
      <span className="text-[14px] font-bold" style={{ color: subject.ink }}>
        {progressLabel}
      </span>
      <ProgressBar
        value={progress}
        max={100}
        label={`Kemajuan ${subject.name}`}
        color={subject.color}
      />
    </AppLink>
  );
}
