import type { ButtonHTMLAttributes, CSSProperties, ReactNode } from "react";
import { cx } from "./cx";

export type ToggleChipProps = Omit<
  ButtonHTMLAttributes<HTMLButtonElement>,
  "children" | "onClick"
> & {
  readonly pressed: boolean;
  readonly onPressedChange: (next: boolean) => void;
  /** `soft`: tinted fill with a coloured border (any accent). `solid`: filled, white text; only
   * for accents with >= 4.5:1 against white, like primary. */
  readonly look?: "soft" | "solid";
  readonly shape?: "pill" | "tile";
  readonly accent?: { readonly color: string; readonly tint: string };
  readonly children: ReactNode;
};

const PRIMARY = { color: "var(--color-primary)", tint: "var(--color-primary-soft)" };

/** A pressable chip that stays on or off (aria-pressed). */
export function ToggleChip({
  pressed,
  onPressedChange,
  look = "soft",
  shape = "pill",
  accent = PRIMARY,
  className,
  style,
  children,
  ...rest
}: ToggleChipProps) {
  const on: CSSProperties =
    look === "solid"
      ? { background: accent.color, borderColor: accent.color, color: "var(--color-on-fill)" }
      : { background: accent.tint, borderColor: accent.color };
  return (
    <button
      type="button"
      aria-pressed={pressed}
      onClick={() => onPressedChange(!pressed)}
      className={cx(
        "pressable inline-flex min-h-12 cursor-pointer items-center gap-2 border-[3px] font-extrabold text-ink",
        pressed ? "" : "border-line-strong bg-surface hover:border-line-sky hover:bg-page",
        shape === "pill"
          ? "rounded-full px-4 text-[15px]"
          : "rounded-btn p-3.5 text-left text-[17px]",
        className,
      )}
      style={pressed ? { ...on, ...style } : style}
      {...rest}
    >
      {children}
    </button>
  );
}
