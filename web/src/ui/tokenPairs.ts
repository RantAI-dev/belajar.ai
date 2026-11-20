import { SUBJECTS } from "./subjects";

/**
 * Every foreground/background token pair the components draw, with the WCAG AA minimum it must
 * meet in both themes: `text` 4.5:1, `large` 3:1 (bold >= 19px or >= 24px), `ui` 3:1 (focus rings,
 * fills and icons that carry meaning). A new component pairing goes here; theme.test.ts checks it.
 * Decorative edges (card borders, drop shadows) are not pairs.
 */
export type Need = "text" | "large" | "ui";
export type TokenPair = {
  readonly fg: string;
  readonly bg: string;
  readonly need: Need;
  readonly use: string;
};

export const MINIMUM: Readonly<Record<Need, number>> = { text: 4.5, large: 3, ui: 3 };

const p = (fg: string, bg: string, need: Need, use: string): TokenPair => ({
  fg: fg.startsWith("focus") ? fg : `color-${fg}`,
  bg: `color-${bg}`,
  need,
  use,
});

const BASE: readonly TokenPair[] = [
  p("ink", "page", "text", "body text on the page"),
  p("ink", "surface", "text", "body text on cards and inputs"),
  p("ink", "surface-alt", "text", "mission rows"),
  p("ink", "primary-soft", "text", "hovered rows, selected answers"),
  p("ink", "success-soft", "text", "done missions, correct answers"),
  p("ink", "oops-soft", "text", "wrong answers"),
  p("ink-muted", "page", "text", "secondary text on the page"),
  p("ink-muted", "surface", "text", "secondary text on cards"),
  p("ink-muted", "surface-alt", "text", "secondary text in mission rows"),
  p("ink-muted", "primary-soft", "text", "hint in the chosen theme option"),
  p("ink-faint", "surface", "text", "placeholders and hints"),
  p("ink-faint", "page", "text", "faint labels on the page"),
  p("primary-ink", "surface", "text", "links, secondary buttons"),
  p("primary-ink", "page", "text", "links on the page"),
  p("primary-ink", "primary-soft", "text", "active nav item, ghost button hover"),
  p("on-fill", "primary", "text", "primary button, user chat bubble"),
  p("on-fill", "primary-strong", "text", "primary button hover"),
  p("on-fill", "hero", "text", "Beranda greeting panel"),
  p("hero-ink-soft", "hero", "text", "greeting panel hint"),
  p("on-fill", "success-solid", "text", "correct answer marker, result number"),
  p("on-fill", "oops-solid", "text", "wrong answer marker, result number"),
  p("on-fill", "success", "large", "success button (bold 19px+)"),
  p("on-fill", "celebrate", "large", "celebration panel headline"),
  p("success-ink", "surface", "text", "stars-earned chip"),
  p("success-ink", "success-soft", "text", "success notice"),
  p("oops-ink", "surface", "text", "field errors"),
  p("oops-ink", "page", "text", "inline errors on the page"),
  p("oops-ink", "oops-soft", "text", "error notice"),
  p("warm-ink", "warm", "text", "fun fact, story card"),
  p("star-ink", "warm", "text", "star count"),
  p("star-ink", "surface-alt", "text", "mission reward"),
  p("star-ink", "success-soft", "text", "done mission reward"),
  p("streak-ink", "streak-soft", "text", "streak badge"),
  p("sun-ink", "sun", "text", "sun button"),
  p("focus", "page", "ui", "focus ring on the page"),
  p("focus", "surface", "ui", "focus ring on cards"),
  p("sun", "hero", "ui", "focus ring inside the greeting panel (.on-dark)"),
  p("primary", "surface", "ui", "selected chip and grade fills"),
  p("primary", "page", "ui", "selected fills on the page"),
  p("success", "surface", "ui", "correct-answer border, progress"),
  p("warm-icon", "warm", "ui", "fun fact bulb"),
  p("ink", "cite-soft", "text", "citation pill label"),
  p("on-fill", "cite-strong", "text", "pressed citation pill"),
  p("cite-line", "surface", "ui", "citation pill outline in an answer"),
  p("ink", "cite-marker", "text", "marked cited sentence"),
  p("cite-marker-ink", "cite-marker", "text", "emphasis inside a marked sentence"),
  p("brand-wordmark", "surface", "large", "wordmark in the header (bold 22px+)"),
  p("brand-wordmark", "page", "large", "wordmark on the page"),
  p("mascot-accent", "page", "ui", "mascot marks (zzz, ?) on the page"),
  p("mascot-accent", "surface", "ui", "mascot marks (zzz, ?) on cards"),
  // The sign-in pages' mascot stage (features/auth/MascotStage).
  p("brand-wordmark", "surface-alt", "large", "wordmark on the sign-in stage"),
  p("focus", "surface-alt", "ui", "focus ring on the 3D mascot canvas"),
  p("mascot-accent", "surface-alt", "ui", "mascot marks (zzz, ?) on the sign-in stage"),
  p("mascot-accent", "mascot-backdrop", "ui", "mascot marks over the stage's circle"),
];

const SUBJECT_PAIRS: readonly TokenPair[] = SUBJECTS.flatMap((s) => {
  const t = `subject-${s.id}`;
  return [
    p(`${t}-ink`, `${t}-tint`, "text", `${s.shortName}: label on its card`),
    p(`${t}-ink`, "surface", "text", `${s.shortName}: label on white`),
    p(`${t}-ink`, "page", "text", `${s.shortName}: label on the page`),
    p("ink", `${t}-tint`, "text", `${s.shortName}: body text on its card`),
    p("on-fill", t, "large", `${s.shortName}: white on the subject fill`),
    p("on-fill", `${t}-solid`, "text", `${s.shortName}: label pill`),
    p(t, "surface", "ui", `${s.shortName}: icon on its tile`),
    p(t, "track", "ui", `${s.shortName}: progress on its track`),
  ];
});

const AVATAR_PAIRS: readonly TokenPair[] = [1, 2, 3, 4, 5].map((n) =>
  p(`avatar-${n}-fg`, `avatar-${n}-bg`, "text", `avatar ${n} initials`),
);

export const TOKEN_PAIRS: readonly TokenPair[] = [...BASE, ...SUBJECT_PAIRS, ...AVATAR_PAIRS];
