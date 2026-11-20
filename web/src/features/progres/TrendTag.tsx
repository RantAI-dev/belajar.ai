import type { ReactNode } from "react";
import { cx, DashIcon, TrendDownIcon, TrendFlatIcon, TrendUpIcon } from "../../ui";
import type { Trend } from "./types";

const ICON: Readonly<Record<Trend, ReactNode>> = {
  up: <TrendUpIcon size={18} />,
  down: <TrendDownIcon size={18} />,
  same: <TrendFlatIcon size={18} />,
  no_data: <DashIcon size={18} />,
};

// "Turun" stays in a calm tone: a dip is something to practise, not a warning.
const TONE: Readonly<Record<Trend, string>> = {
  up: "text-success-edge",
  down: "text-oops-ink",
  same: "text-primary-strong",
  no_data: "text-ink-faint",
};

/** This week against last week: an arrow and a word, never colour alone. */
export function TrendTag({
  trend,
  label,
  detail,
}: {
  readonly trend: Trend;
  readonly label: string;
  readonly detail: string;
}) {
  return (
    <span className={cx("inline-flex items-center gap-1 text-[14px] font-extrabold", TONE[trend])}>
      {ICON[trend]}
      <span>{label}</span>
      <span className="sr-only">. {detail}</span>
    </span>
  );
}
