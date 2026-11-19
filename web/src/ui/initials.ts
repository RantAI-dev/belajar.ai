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

// Pastel fills with an ink that reads at 4.5:1 on them.
export const AVATAR_COLORS: ReadonlyArray<{ readonly bg: string; readonly fg: string }> = [
  { bg: "#ffe1ec", fg: "#8a1f4b" },
  { bg: "#e0f0fd", fg: "#00598a" },
  { bg: "#e8f5f0", fg: "#1f5c49" },
  { bg: "#fff4e5", fg: "#8a4b0f" },
  { bg: "#f0edf9", fg: "#3d2c75" },
];

/** The same name always gets the same colour. */
export const avatarColorIndex = (name: string): number => {
  let hash = 0;
  for (const ch of name) hash = (hash * 31 + (ch.codePointAt(0) ?? 0)) % 9973;
  return hash % AVATAR_COLORS.length;
};
