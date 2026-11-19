import { cx } from "../../ui";
import { GRADE_GROUPS, type Grade } from "./grades";

/** Big tappable grade buttons, grouped SD / SMP / SMA. A radio group under the hood. */
export function GradePicker({
  value,
  onChange,
  error,
}: {
  readonly value: Grade | null;
  readonly onChange: (grade: Grade) => void;
  readonly error?: string;
}) {
  return (
    <fieldset
      className="m-0 flex flex-col gap-3 border-0 p-0"
      aria-describedby={error !== undefined ? "grade-error" : undefined}
    >
      <legend className="mb-1 text-[15px] font-extrabold text-ink-muted">Kamu kelas berapa?</legend>
      {GRADE_GROUPS.map((group) => (
        <div key={group.level} className="flex flex-col gap-2">
          <span className="text-[14px] font-extrabold text-ink-muted">{group.label}</span>
          <div className="grid grid-cols-3 gap-2 sm:grid-cols-6">
            {group.grades.map((grade) => {
              const on = grade === value;
              return (
                <label
                  key={grade}
                  className={cx(
                    "pressable ring-within flex min-h-[60px] cursor-pointer items-center justify-center rounded-[16px] border-[3px] font-display text-[26px] font-semibold",
                    on
                      ? "border-primary bg-primary text-white shadow-[0_4px_0_var(--color-primary-edge)]"
                      : "border-line-strong bg-white text-ink hover:bg-page",
                  )}
                >
                  <input
                    type="radio"
                    name="grade"
                    value={grade}
                    checked={on}
                    onChange={() => onChange(grade)}
                    className="sr-only"
                  />
                  {grade}
                  <span className="sr-only"> {group.level}</span>
                </label>
              );
            })}
          </div>
        </div>
      ))}
      {error !== undefined && (
        <span id="grade-error" className="text-[15px] font-bold text-oops-ink">
          {error}
        </span>
      )}
    </fieldset>
  );
}
