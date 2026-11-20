import { useId } from "react";
import { APP_NAME } from "./brand";
import { cx } from "./cx";
import { AppLink } from "./link";

/**
 * The logo mark: a rounded "b" in the brand gradient (the same in both themes) with a white
 * counter and a yellow sparkle. web/public/favicon.svg is the same drawing with literal colours.
 */
export function LogoMark({ size = 40 }: { readonly size?: number }) {
  const id = `${useId()}-mark`;
  return (
    <svg aria-hidden="true" width={size} height={size} viewBox="0 0 64 64" className="shrink-0">
      <defs>
        <linearGradient id={id} x1="0" y1="0" x2="0.8" y2="1">
          <stop offset="0" style={{ stopColor: "var(--color-brand-teal)" }} />
          <stop offset="0.5" style={{ stopColor: "var(--color-brand-blue)" }} />
          <stop offset="1" style={{ stopColor: "var(--color-brand-purple)" }} />
        </linearGradient>
      </defs>
      <path
        d="M17 6a8 8 0 0 1 8 8v12.2A19 19 0 1 1 9 44.8V14a8 8 0 0 1 8-8Z"
        style={{ fill: `url(#${id})` }}
      />
      <circle cx="28" cy="44" r="8" style={{ fill: "var(--color-snow)" }} />
      <path
        d="M52 3Q53.2 9.8 60 11Q53.2 12.2 52 19Q50.8 12.2 44 11Q50.8 9.8 52 3Z"
        style={{ fill: "var(--color-brand-sparkle)" }}
      />
    </svg>
  );
}

/** "belajar.ai" set in the bundled display font (Fredoka), navy in light, periwinkle in dark. */
export function Wordmark({ className }: { readonly className?: string }) {
  return (
    <span
      className={cx("font-display font-semibold tracking-tight", className)}
      style={{ color: "var(--color-brand-wordmark)" }}
    >
      {APP_NAME}
    </span>
  );
}

/** Mark plus wordmark. With `href` it links (to Beranda in the header); on phones only the mark shows. */
export function Logo({
  href,
  compact = false,
  className,
}: {
  readonly href?: string;
  readonly compact?: boolean;
  readonly className?: string;
}) {
  const inner = (
    <>
      <LogoMark size={compact ? 36 : 40} />
      <Wordmark
        className={cx(
          href !== undefined && "max-[419px]:sr-only",
          compact ? "text-[22px]" : "text-[26px]",
        )}
      />
    </>
  );
  const box = cx("flex min-h-12 items-center gap-2 rounded-chip", className);
  return href === undefined ? (
    <div className={box}>{inner}</div>
  ) : (
    <AppLink href={href} aria-label={`${APP_NAME}, beranda`} className={box}>
      {inner}
    </AppLink>
  );
}
