import type { ButtonHTMLAttributes, ReactNode } from "react";
import { cx } from "./cx";
import { Spinner } from "./icons";
import { AppLink, type LinkProps } from "./link";

export type ButtonVariant = "primary" | "secondary" | "success" | "ghost" | "sun";
export type ButtonSize = "md" | "lg";

// White text on success green is 3.3:1, so every size uses bold text >= 19px (WCAG large text).
const VARIANT: Record<ButtonVariant, string> = {
  primary:
    "bg-primary text-on-fill shadow-[0_5px_0_var(--color-primary-edge)] hover:bg-primary-strong",
  secondary:
    "bg-surface text-primary-ink border-2 border-line-sky shadow-[0_4px_0_var(--color-line-sky)] hover:bg-primary-soft",
  success: "bg-success text-on-fill shadow-[0_5px_0_var(--color-success-edge)] hover:brightness-95",
  sun: "bg-sun text-sun-ink shadow-[0_4px_0_var(--color-sun-edge)] hover:brightness-95",
  ghost: "bg-transparent text-primary-ink hover:bg-primary-soft",
};

const SIZE: Record<ButtonSize, string> = {
  md: "min-h-[52px] px-5 text-[19px] rounded-[16px]",
  lg: "min-h-[60px] px-7 text-[20px] rounded-btn",
};

const PRESS = "active:translate-y-[4px] active:shadow-none";

export const buttonClass = ({
  variant = "primary",
  size = "md",
  block = false,
  disabled = false,
}: {
  readonly variant?: ButtonVariant | undefined;
  readonly size?: ButtonSize | undefined;
  readonly block?: boolean | undefined;
  readonly disabled?: boolean | undefined;
}): string =>
  cx(
    "pressable inline-flex items-center justify-center gap-2 font-extrabold leading-tight text-center no-underline select-none",
    VARIANT[variant],
    SIZE[size],
    block && "w-full",
    disabled
      ? "cursor-not-allowed opacity-55 shadow-none"
      : cx("cursor-pointer", variant !== "ghost" && PRESS),
  );

type Common = {
  readonly variant?: ButtonVariant;
  readonly size?: ButtonSize;
  readonly block?: boolean;
  readonly icon?: ReactNode;
  readonly children: ReactNode;
};

export type ButtonProps = Common &
  Omit<ButtonHTMLAttributes<HTMLButtonElement>, "children"> & {
    readonly loading?: boolean;
    /** Read out instead of the label while loading. */
    readonly loadingLabel?: string;
  };

export function Button({
  variant,
  size,
  block,
  icon,
  loading = false,
  loadingLabel,
  disabled = false,
  className,
  children,
  type = "button",
  ...rest
}: ButtonProps) {
  const off = disabled || loading;
  return (
    <button
      type={type}
      disabled={off}
      aria-busy={loading || undefined}
      className={cx(buttonClass({ variant, size, block, disabled: off }), className)}
      {...rest}
    >
      {loading ? <Spinner /> : icon}
      <span>{loading && loadingLabel !== undefined ? loadingLabel : children}</span>
    </button>
  );
}

export type ButtonLinkProps = Common & LinkProps;

/** A link that looks like a button: use it whenever the action is navigation. */
export function ButtonLink({
  variant,
  size,
  block,
  icon,
  className,
  children,
  ...rest
}: ButtonLinkProps) {
  return (
    <AppLink className={cx(buttonClass({ variant, size, block }), className)} {...rest}>
      {icon}
      <span>{children}</span>
    </AppLink>
  );
}
