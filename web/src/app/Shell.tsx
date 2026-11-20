import { useEffect, useRef } from "react";
import { Outlet, useLocation } from "react-router";
import { AppHeader, type NavKey } from "../ui";
import { useHeaderStats } from "./headerStats";
import { useAuth } from "./session";

const LINKS = { beranda: "/", tanya: "/tanya", belajar: "/belajar", latihan: "/latihan" } as const;

const navKeyOf = (path: string): NavKey | null => {
  if (path === "/") return "beranda";
  if (path.startsWith("/tanya")) return "tanya";
  if (path.startsWith("/belajar")) return "belajar";
  if (path.startsWith("/latihan")) return "latihan";
  return null;
};

/** Header, bottom tab bar on phones, and the page. */
export function Shell() {
  const { pathname } = useLocation();
  const { student } = useAuth();
  const stats = useHeaderStats();
  const shownPath = useRef(pathname);
  useEffect(() => {
    if (shownPath.current === pathname) return;
    shownPath.current = pathname;
    window.scrollTo(0, 0);
  }, [pathname]);
  return (
    <div className="flex min-h-screen flex-col">
      <a
        href="#isi"
        className="sr-only focus:not-sr-only focus:absolute focus:top-2 focus:left-2 focus:z-30 focus:rounded-chip focus:bg-surface focus:px-4 focus:py-3 focus:font-extrabold"
      >
        Langsung ke isi
      </a>
      <AppHeader
        links={LINKS}
        current={navKeyOf(pathname)}
        studentName={student?.nickname ?? ""}
        stars={stats.stars}
        streakDays={stats.streakDays}
        accountHref="/akun"
      />
      <main
        id="isi"
        className="mx-auto flex w-full max-w-[1240px] flex-1 flex-col px-4 pt-6 pb-[104px] sm:px-7 md:pb-14"
      >
        <Outlet />
      </main>
    </div>
  );
}
