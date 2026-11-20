import { type ReactNode, useEffect, useRef } from "react";
import "./fonts";
import "./auth.css";
import { MascotStage, type StageOutcome } from "./MascotStage";

/**
 * The sign-in pages (Masuk, Daftar, the Google step): the mascot stage on the left (on top on a
 * phone) and the form card on the right. The card comes first in the document so the keyboard and
 * screen readers reach the form before the mascot; CSS puts the stage first on screen.
 */
export function AuthFrame({
  title,
  subtitle,
  outcome,
  children,
}: {
  readonly title: string;
  readonly subtitle: string;
  /** The latest sign-in result: the mascot reacts, and a failure shakes the card. */
  readonly outcome?: StageOutcome | undefined;
  readonly children: ReactNode;
}) {
  const card = useRef<HTMLElement>(null);
  useEffect(() => {
    const el = card.current;
    if (el === null || outcome?.kind !== "failure") return;
    el.classList.remove("auth-shake");
    // Reading a layout property restarts the animation for a second wrong password.
    void el.offsetWidth;
    el.classList.add("auth-shake");
  }, [outcome]);

  return (
    <div className="font-landing grid min-h-screen bg-page text-ink min-[861px]:grid-cols-[minmax(0,1.15fr)_minmax(0,1fr)]">
      <main className="grid place-items-center px-[max(5vw,16px)] py-[clamp(24px,4vw,56px)]">
        <section
          ref={card}
          aria-labelledby="auth-title"
          onAnimationEnd={() => card.current?.classList.remove("auth-shake")}
          className="flex w-full max-w-[440px] flex-col gap-5 rounded-card border-2 border-line bg-surface p-[clamp(22px,3vw,36px)]"
        >
          <div>
            <h1
              id="auth-title"
              className="m-0 mb-1.5 text-[32px] leading-tight font-extrabold tracking-tight"
            >
              {title}
            </h1>
            <p className="m-0 text-[16px] font-medium text-ink-muted">{subtitle}</p>
          </div>
          {children}
        </section>
      </main>
      <MascotStage outcome={outcome} />
    </div>
  );
}
