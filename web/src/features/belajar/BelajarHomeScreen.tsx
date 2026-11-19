import { Mascot, SubjectCard, subjectById } from "../../ui";
import type { SubjectEntry } from "./types";

export function BelajarHomeScreen({ subjects }: { readonly subjects: readonly SubjectEntry[] }) {
  return (
    <div className="flex flex-col gap-6">
      <div className="flex items-center gap-4">
        <Mascot mood="happy" size={80} />
        <div className="flex flex-col gap-1">
          <h1 className="m-0 font-display text-[28px] font-semibold leading-tight">
            Mau belajar apa hari ini?
          </h1>
          <p className="m-0 text-[17px] font-semibold text-ink-muted">
            Pilih mapel, lalu kita jelajahi babnya satu per satu.
          </p>
        </div>
      </div>
      <div className="grid grid-cols-2 gap-3 sm:grid-cols-[repeat(auto-fill,minmax(190px,1fr))] sm:gap-4">
        {subjects.map((s) => {
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
    </div>
  );
}
