import type { MascotExpression } from "../../ui";

/** What the Beranda mascot shows and says before the student taps it. */
export type HeroMood = {
  readonly expression: MascotExpression;
  readonly line: string;
};

export type HeroMoodInput = {
  readonly missionsDone: number;
  readonly missionsTotal: number;
  readonly streakDays: number;
  /** The newest unfinished lesson's last change, the only "last seen" Beranda loads. */
  readonly lastActivityAt: string | null;
  readonly now: Date;
};

const DAY_MS = 24 * 60 * 60 * 1000;
/** Away this long with no streak and nothing done today: the mascot has dozed off. */
export const SLEEPY_AFTER_DAYS = 7;

const isMilestone = (days: number): boolean => days === 3 || (days >= 7 && days % 7 === 0);

const daysSince = (at: string | null, now: Date): number | null => {
  if (at === null) return null;
  const t = Date.parse(at);
  return Number.isNaN(t) ? null : (now.getTime() - t) / DAY_MS;
};

/** The mascot's mood from today's missions, the streak and when the student last studied. */
export function heroMood(input: HeroMoodInput): HeroMood {
  const { missionsDone: done, missionsTotal: total, streakDays: streak } = input;
  if (total > 0 && done >= total)
    return { expression: "celebration", line: "Semua misi hari ini selesai! Hebat sekali!" };
  if (done > 0 && isMilestone(streak))
    return { expression: "excited", line: `Runtunan ${streak} hari! Kamu keren, terus lanjut!` };
  if (done > 0)
    return {
      expression: "happy",
      line: `Mantap! ${done} dari ${total} misi selesai. Lanjut lagi, yuk!`,
    };
  if (streak > 0)
    return {
      expression: "idea",
      line: `Runtunanmu ${streak} hari. Yuk jaga dengan belajar hari ini!`,
    };
  const away = daysSince(input.lastActivityAt, input.now);
  if (away !== null && away >= SLEEPY_AFTER_DAYS)
    return { expression: "sleepy", line: "Hoaam... lama tak jumpa! Yuk belajar lagi hari ini." };
  return { expression: "cheerful", line: "Yuk mulai belajar hari ini!" };
}

const REACTION_LINES: readonly string[] = [
  "Hihi, geli! Ayo kita belajar bareng!",
  "Wah, kamu menemukanku! Siap jadi lebih pintar?",
  "Tahukah kamu? Otak makin kuat kalau sering dilatih!",
  "Aku siap menjawab pertanyaanmu!",
  "Langkah kecil hari ini, lompatan besar besok!",
];

/** The one-liner for the student's n-th tap (0-based), taking turns through the list. */
export const reactionLine = (n: number): string =>
  REACTION_LINES[Math.abs(Math.trunc(n)) % REACTION_LINES.length] ?? "";

/** How long a tap's reaction lasts before the mascot returns to its mood. */
export const REACTION_MS = 1500;

/** The faces a tap shows and when each starts (ms); one held face under reduced motion. */
export const reactionFrames = (
  reducedMotion: boolean,
): readonly { readonly expression: MascotExpression; readonly at: number }[] =>
  reducedMotion
    ? [{ expression: "excited", at: 0 }]
    : [
        { expression: "excited", at: 0 },
        { expression: "surprised", at: 500 },
        { expression: "happy", at: 1000 },
      ];
