import {
  type InputHTMLAttributes,
  type SelectHTMLAttributes,
  useId,
  useRef,
  useState,
} from "react";
import { cx } from "./cx";

const CONTROL =
  "min-h-[52px] w-full rounded-chip border-2 bg-surface px-3.5 text-[17px] font-bold text-ink placeholder:text-ink-faint placeholder:font-semibold";

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

export type PasswordFieldProps = Omit<TextFieldProps, "type">;

function EyeIcon({ crossed }: { readonly crossed: boolean }) {
  return (
    <svg
      aria-hidden="true"
      width="24"
      height="24"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
    >
      <path d="M2 12s3.6-7 10-7 10 7 10 7-3.6 7-10 7S2 12 2 12z" />
      <circle cx="12" cy="12" r="3" />
      {crossed && <path d="M4 4l16 16" />}
    </svg>
  );
}

/**
 * A password input with a show/hide eye. `data-password` marks it for the login mascot, which
 * covers its eyes while it is focused and hidden.
 */
export function PasswordField({
  label,
  hint,
  error,
  hideLabel = false,
  value,
  onValueChange,
  className,
  id,
  ...rest
}: PasswordFieldProps) {
  const auto = useId();
  const inputId = id ?? auto;
  const helpId = `${inputId}-help`;
  const hasHelp = error !== undefined || hint !== undefined;
  const [shown, setShown] = useState(false);
  const input = useRef<HTMLInputElement>(null);
  return (
    <div className={cx("flex flex-col gap-1.5", className)}>
      <label
        htmlFor={inputId}
        className={cx("text-[15px] font-extrabold text-ink-muted", hideLabel && "sr-only")}
      >
        {label}
      </label>
      <div className="relative">
        <input
          ref={input}
          id={inputId}
          type={shown ? "text" : "password"}
          data-password=""
          value={value}
          onChange={(e) => onValueChange(e.target.value)}
          aria-invalid={error !== undefined || undefined}
          aria-describedby={hasHelp ? helpId : undefined}
          className={cx(
            CONTROL,
            "pr-14",
            error !== undefined ? "border-oops" : "border-line-strong",
          )}
          {...rest}
        />
        <button
          type="button"
          aria-pressed={shown}
          aria-label={shown ? "Sembunyikan kata sandi" : "Tampilkan kata sandi"}
          // Keeps the caret in the input: the press must not steal focus from it.
          onPointerDown={(e) => e.preventDefault()}
          onClick={() => {
            setShown((s) => !s);
            input.current?.focus();
          }}
          className="absolute top-1 right-1 grid size-11 cursor-pointer place-items-center rounded-[10px] text-ink-muted hover:bg-primary-soft hover:text-primary-ink"
        >
          <EyeIcon crossed={!shown} />
        </button>
      </div>
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
