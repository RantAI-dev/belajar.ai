import { beforeAll, describe, expect, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import { loadMath } from "../../ui";
import { QuizScreen } from "./QuizScreen";
import { ResultScreen } from "./ResultScreen";
import type { QuizQuestion } from "./types";

const question: QuizQuestion = {
  id: "q1",
  kind: "story",
  story: "Ibu membeli kue seharga Rp5.000 dan $3 permen.",
  prompt: "Berapakah $\\frac{1}{4} + \\frac{2}{4}$?",
  options: [
    { id: "A", text: "$\\frac{3}{4}$" },
    { id: "B", text: "$\\frac{3}{8}$" },
  ],
};

const noop = () => {};

beforeAll(async () => {
  expect(await loadMath()).toBe(true);
});

describe("Latihan with maths", () => {
  test("the question and its options draw maths; money in the story stays text", () => {
    const out = renderToStaticMarkup(
      <QuizScreen
        topic="Pecahan"
        subjectId="mtk"
        question={question}
        index={0}
        total={5}
        chosen={[]}
        onChoose={noop}
        onPrev={noop}
        onNext={noop}
        onFinish={noop}
        canFinish={false}
        submitting={false}
        submitError={null}
      />,
    );
    expect(out.match(/data-math="inline"/g)?.length).toBe(3);
    expect(out).toContain("Rp5.000 dan $3 permen.");
  });

  test("the explanation in the review draws maths and keeps its citation", () => {
    const out = renderToStaticMarkup(
      <ResultScreen
        result={{
          correct: 0,
          total: 1,
          starsEarned: 0,
          items: [
            {
              question,
              isCorrect: false,
              chosen: ["B"],
              correct: ["A"],
              explanation: "Penyebutnya sama, jadi $\\frac{1+2}{4} = \\frac{3}{4}$ [1].",
              source: null,
            },
          ],
        }}
        againHref="/latihan"
        studyHref="/belajar"
      />,
    );
    expect(out).not.toContain("$\\frac");
    expect(out).toContain("sumber </span>[1]");
    expect(out).toContain('data-math="inline"');
  });
});
