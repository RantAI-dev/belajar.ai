import { beforeAll, describe, expect, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import { loadMath } from "../../ui";
import type { TanyaMessage } from "../tanya";
import { LessonScreen, type LessonScreenProps } from "./LessonScreen";

const lesson = (text: string): LessonScreenProps => {
  const messages: TanyaMessage[] = [
    {
      id: "m1",
      kind: "answer",
      status: "done",
      parts: [{ text }],
      sources: [],
    },
  ];
  return {
    subjectId: "mtk",
    chapterTitle: "Bab 2: Pecahan",
    step: "explain",
    messages,
    busy: false,
    loading: false,
    onSend: () => {},
    onNext: () => {},
    moving: false,
    onComplete: () => {},
    completing: false,
    actionError: null,
    result: null,
    bestStars: 0,
    mapHref: "/belajar/mtk",
    practiceHref: "/latihan",
  };
};

beforeAll(async () => {
  expect(await loadMath()).toBe(true);
});

describe("Belajar lesson with maths", () => {
  test("a display equation is drawn in its own scrolling box", () => {
    const out = renderToStaticMarkup(
      <LessonScreen {...lesson("Rumus ABC:\n$$x = \\frac{-b \\pm \\sqrt{b^2 - 4ac}}{2a}$$")} />,
    );
    expect(out).toContain('data-math="display"');
    expect(out).toContain("overflow-x-auto");
  });

  test("invalid LaTeX in a lesson shows its source, not an error", () => {
    const out = renderToStaticMarkup(<LessonScreen {...lesson("Lihat $\\frac{1$ ini.")} />);
    expect(out).toContain("$\\frac{1$");
    expect(out).not.toContain("katex-error");
  });
});
