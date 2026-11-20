import { useState } from "react";
import type { FormMessage } from "../../contracts/messages";
import type { Result } from "../../contracts/result";
import { Card, cx, FormAlert, type ThemePreference } from "../../ui";

export const THEME_OPTIONS: ReadonlyArray<{
  readonly value: ThemePreference;
  readonly label: string;
  readonly hint: string;
}> = [
  { value: "system", label: "Sistem", hint: "Ikuti pengaturan perangkatmu" },
  { value: "light", label: "Terang", hint: "Selalu terang" },
  { value: "dark", label: "Gelap", hint: "Selalu gelap, nyaman di malam hari" },
];

export type ThemeSettingProps = {
  readonly value: ThemePreference;
  readonly onChange: (theme: ThemePreference) => Promise<Result<void, FormMessage>>;
};

/** Sistem / Terang / Gelap as a radio group; the change shows at once and is saved after. */
export function ThemeSetting({ value, onChange }: ThemeSettingProps) {
  const [problem, setProblem] = useState<string | undefined>(undefined);

  const choose = async (theme: ThemePreference) => {
    setProblem(undefined);
    const r = await onChange(theme);
    if (!r.ok) setProblem(r.error.form ?? undefined);
  };

  return (
    <Card as="section" aria-labelledby="tampilan-title" className="flex flex-col gap-4">
      <h2 id="tampilan-title" className="m-0 font-display text-[22px] font-semibold">
        Tampilan
      </h2>
      {problem !== undefined && <FormAlert tone="error">{problem}</FormAlert>}
      <fieldset className="m-0 flex flex-col gap-2 border-0 p-0">
        <legend className="mb-2 text-[15px] font-extrabold text-ink-muted">Tema warna</legend>
        <div className="grid gap-2 sm:grid-cols-3">
          {THEME_OPTIONS.map((option) => {
            const on = option.value === value;
            return (
              <label
                key={option.value}
                className={cx(
                  "pressable ring-within flex min-h-[60px] cursor-pointer flex-col justify-center rounded-[16px] border-[3px] px-4 py-2",
                  on
                    ? "border-primary bg-primary-soft text-ink"
                    : "border-line-strong bg-surface text-ink hover:bg-page",
                )}
              >
                <input
                  type="radio"
                  name="theme"
                  value={option.value}
                  checked={on}
                  onChange={() => void choose(option.value)}
                  className="sr-only"
                />
                <span className="text-[18px] font-extrabold">{option.label}</span>
                <span className="text-[14px] font-bold text-ink-muted">{option.hint}</span>
              </label>
            );
          })}
        </div>
      </fieldset>
    </Card>
  );
}
