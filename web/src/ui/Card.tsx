import type { HTMLAttributes, ReactNode } from "react";
import { cx } from "./cx";

export type CardTone = "white" | "warm" | "page";

const TONE: Record<CardTone, string> = {
  white: "bg-surface border-2 border-line",
  warm: "bg-warm border-2 border-warm-line",
  page: "bg-page border-2 border-line",
};

export type CardProps = HTMLAttributes<HTMLElement> & {
  readonly as?: "div" | "section" | "article" | "aside" | "form";
  readonly tone?: CardTone;
  readonly padded?: boolean;
  readonly children: ReactNode;
};

export function Card({
  as: Tag = "div",
  tone = "white",
  padded = true,
  className,
  children,
  ...rest
}: CardProps) {
  return (
    <Tag className={cx("rounded-card", TONE[tone], padded && "p-5 sm:p-6", className)} {...rest}>
      {children}
    </Tag>
  );
}
