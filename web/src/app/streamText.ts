/** The first `words` words of `text`, keeping its spacing: how the fake answer "streams". */
export const takeWords = (text: string, words: number): string => {
  if (words <= 0) return "";
  const parts = text.split(/(\s+)/);
  return parts.slice(0, words * 2 - 1).join("");
};

export const wordCount = (text: string): number =>
  text.split(/\s+/).filter((w) => w.length > 0).length;
