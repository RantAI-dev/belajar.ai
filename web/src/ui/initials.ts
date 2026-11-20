/** One or two capital initials from a name: "Siswa A" is "SA", "budi" is "B". */
export const initialsOf = (name: string): string => {
  const words = name
    .trim()
    .split(/\s+/)
    .filter((w) => w.length > 0);
  return words
    .slice(0, 2)
    .map((w) => w.charAt(0).toUpperCase())
    .join("");
};

// Pastel fills with an ink that reads at 4.5:1 on them, in both themes (theme tokens; the pairs
// are checked in tokenPairs.ts).
export const AVATAR_COLORS: ReadonlyArray<{ readonly bg: string; readonly fg: string }> = [
  1, 2, 3, 4, 5,
].map((n) => ({ bg: `var(--color-avatar-${n}-bg)`, fg: `var(--color-avatar-${n}-fg)` }));

/** The same name always gets the same colour. */
export const avatarColorIndex = (name: string): number => {
  let hash = 0;
  for (const ch of name) hash = (hash * 31 + (ch.codePointAt(0) ?? 0)) % 9973;
  return hash % AVATAR_COLORS.length;
};
