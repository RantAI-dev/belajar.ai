import type { ReactNode } from "react";
import { APP_NAME, LogoMark, Mascot, type MascotMood } from "../../ui";

/** The friendly frame around login and register: mascot greeting on top, form card below. */
export function AuthFrame({
  title,
  greeting,
  mood = "happy",
  children,
}: {
  readonly title: string;
  readonly greeting: string;
  readonly mood?: MascotMood;
  readonly children: ReactNode;
}) {
  return (
    <main className="mx-auto flex min-h-screen w-full max-w-[560px] flex-col gap-5 px-4 py-8 sm:py-12">
      <div className="flex items-center gap-2.5 self-center">
        <LogoMark size={36} />
        <span className="font-display text-[24px] font-semibold">{APP_NAME}</span>
      </div>
      <div className="flex items-center gap-4">
        <Mascot mood={mood} size={88} animate />
        <div className="rounded-[6px_22px_22px_22px] border-2 border-line bg-surface px-4 py-3 text-[17px] font-bold leading-snug">
          {greeting}
        </div>
      </div>
      <section
        aria-labelledby="auth-title"
        className="flex flex-col gap-5 rounded-panel border-2 border-line bg-surface p-5 sm:p-7"
      >
        <h1 id="auth-title" className="m-0 font-display text-[28px] font-semibold">
          {title}
        </h1>
        {children}
      </section>
    </main>
  );
}
