import type { ReactNode } from "react";
import { Avatar } from "./Avatar";
import { cx } from "./cx";
import { BookIcon, ChatIcon, CheckSquareIcon, HomeIcon } from "./icons";
import { AppLink } from "./link";
import { Logo } from "./Mascot";
import { StarCount } from "./Stars";
import { StreakBadge } from "./StreakBadge";

export type NavKey = "beranda" | "tanya" | "belajar" | "latihan";

const NAV: ReadonlyArray<{
  readonly key: NavKey;
  readonly label: string;
  readonly icon: ReactNode;
}> = [
  { key: "beranda", label: "Beranda", icon: <HomeIcon /> },
  { key: "tanya", label: "Tanya", icon: <ChatIcon /> },
  { key: "belajar", label: "Belajar", icon: <BookIcon /> },
  { key: "latihan", label: "Latihan", icon: <CheckSquareIcon /> },
];

export type AppHeaderProps = {
  readonly links: Readonly<Record<NavKey, string>>;
  readonly current: NavKey | null;
  readonly studentName: string;
  readonly stars: number;
  readonly streakDays: number;
  readonly accountHref: string;
};

/** Logo, main menu, stars and the account. On phones the menu becomes a bottom tab bar. */
export function AppHeader({
  links,
  current,
  studentName,
  stars,
  streakDays,
  accountHref,
}: AppHeaderProps) {
  return (
    <header className="border-b-2 border-line bg-surface">
      <div className="mx-auto flex min-h-[64px] max-w-[1240px] items-center justify-between gap-x-5 gap-y-3 px-4 py-2 sm:min-h-[76px] sm:px-7">
        <Logo href={links.beranda} />
        <nav
          aria-label="Menu utama"
          className="fixed inset-x-0 bottom-0 z-20 border-t-2 border-line bg-surface px-2 pb-[env(safe-area-inset-bottom)] md:static md:border-0 md:bg-transparent md:p-0"
        >
          <ul className="m-0 flex list-none justify-around gap-1.5 p-0 md:justify-start">
            {NAV.map((item) => {
              const on = item.key === current;
              return (
                <li key={item.key} className="flex-1 md:flex-none">
                  <AppLink
                    href={links[item.key]}
                    aria-current={on ? "page" : undefined}
                    className={cx(
                      "flex min-h-[60px] flex-col items-center justify-center gap-0.5 rounded-chip px-2 text-[13px] md:min-h-12 md:flex-row md:gap-2 md:px-[18px] md:text-[16px]",
                      on
                        ? "bg-primary-soft font-extrabold text-primary-ink"
                        : "font-bold text-ink-muted hover:bg-page",
                    )}
                  >
                    {item.icon}
                    {item.label}
                  </AppLink>
                </li>
              );
            })}
          </ul>
        </nav>
        <div className="flex items-center gap-2">
          <StreakBadge days={streakDays} />
          <StarCount count={stars} />
          <AppLink
            href={accountHref}
            aria-label={`Akun ${studentName}`}
            className="flex size-12 items-center justify-center rounded-full"
          >
            <Avatar name={studentName} />
          </AppLink>
        </div>
      </div>
    </header>
  );
}
