/** Joins class names, skipping empty and false values. */
export const cx = (...parts: ReadonlyArray<string | false | null | undefined>): string =>
  parts.filter((p): p is string => typeof p === "string" && p.length > 0).join(" ");
