import { type InputHTMLAttributes, type SelectHTMLAttributes, useId } from "react";
import { cx } from "./cx";

const CONTROL =
  "min-h-[52px] w-full rounded-chip border-2 bg-white px-3.5 text-[17px] font-bold text-ink placeholder:text-ink-faint placeholder:font-semibold";

type Shared = {
  readonly label: string;
  readonly hint?: string;
  readonly error?: string;
  readonly hideLabel?: boolean;
};

function Help({
  id,
  hint,
  error,
}: {
  readonly id: string;
  readonly hint?: string;
  readonly error?: string;
}) {
  if (error !== undefined) {
    return (
      <span id={id} className="text-[15px] font-bold text-oops-ink">
        {error}
      </span>
    );
  }
  if (hint !== undefined) {
    return (
      <span id={id} className="text-[14px] font-semibold text-ink-muted">
        {hint}
      </span>
    );
  }
  return null;
}

export type TextFieldProps = Shared &
  Omit<InputHTMLAttributes<HTMLInputElement>, "onChange" | "value"> & {
    readonly value: string;
    readonly onValueChange: (value: string) => void;
  };

export function TextField({
  label,
  hint,
  error,
  hideLabel = false,
  value,
  onValueChange,
  className,
  id,
  ...rest
}: TextFieldProps) {
  const auto = useId();
  const inputId = id ?? auto;
  const helpId = `${inputId}-help`;
  const hasHelp = error !== undefined || hint !== undefined;
  return (
    <div className={cx("flex flex-col gap-1.5", className)}>
      <label
        htmlFor={inputId}
        className={cx("text-[15px] font-extrabold text-ink-muted", hideLabel && "sr-only")}
      >
        {label}
      </label>
      <input
        id={inputId}
        value={value}
        onChange={(e) => onValueChange(e.target.value)}
        aria-invalid={error !== undefined || undefined}
        aria-describedby={hasHelp ? helpId : undefined}
        className={cx(CONTROL, error !== undefined ? "border-oops" : "border-line-strong")}
        {...rest}
      />
      <Help
        id={helpId}
        {...(hint !== undefined ? { hint } : {})}
        {...(error !== undefined ? { error } : {})}
      />
    </div>
  );
}

export type SelectOption = { readonly value: string; readonly label: string };

export type SelectFieldProps = Shared &
  Omit<SelectHTMLAttributes<HTMLSelectElement>, "onChange" | "value"> & {
    readonly value: string;
    readonly options: readonly SelectOption[];
    readonly onValueChange: (value: string) => void;
  };

export function SelectField({
  label,
  hint,
  error,
  hideLabel = false,
  value,
  options,
  onValueChange,
  className,
  id,
  ...rest
}: SelectFieldProps) {
  const auto = useId();
  const selectId = id ?? auto;
  const helpId = `${selectId}-help`;
  const hasHelp = error !== undefined || hint !== undefined;
  return (
    <div className={cx("flex flex-col gap-1.5", className)}>
      <label
        htmlFor={selectId}
        className={cx("text-[15px] font-extrabold text-ink-muted", hideLabel && "sr-only")}
      >
        {label}
      </label>
      <select
        id={selectId}
        value={value}
        onChange={(e) => onValueChange(e.target.value)}
        aria-invalid={error !== undefined || undefined}
        aria-describedby={hasHelp ? helpId : undefined}
        className={cx(
          CONTROL,
          "cursor-pointer",
          error !== undefined ? "border-oops" : "border-line-strong",
        )}
        {...rest}
      >
        {options.map((o) => (
          <option key={o.value} value={o.value}>
            {o.label}
          </option>
        ))}
      </select>
      <Help
        id={helpId}
        {...(hint !== undefined ? { hint } : {})}
        {...(error !== undefined ? { error } : {})}
      />
    </div>
  );
}
