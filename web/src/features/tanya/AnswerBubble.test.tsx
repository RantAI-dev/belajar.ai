import { beforeAll, describe, expect, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import { loadMath } from "../../ui";
import { AnswerBubble } from "./AnswerBubble";
import type { TanyaMessage } from "./types";

type Answer = Extract<TanyaMessage, { kind: "answer" }>;

const answer = (text: string, status: Answer["status"] = "done"): Answer => ({
  id: "m1",
  kind: "answer",
  status,
  parts: [{ text }],
  sources: [
    {
      id: "m1-1",
      label: "[1] Matematika VII · hal. 40",
      title: "Matematika VII",
      book: "Matematika VII",
    },
  ],
});

const html = (a: Answer) => renderToStaticMarkup(<AnswerBubble answer={a} />);

beforeAll(async () => {
  expect(await loadMath()).toBe(true);
});

describe("Tanya answer with maths", () => {
  test("a fraction answer is drawn, and its citation and source chip stay", () => {
    const out = html(answer("Jadi $ \\frac{1}{4} + \\frac{3}{4} = 1 $ [1]."));
    expect(out).toContain('data-math="inline"');
    expect(out).toContain("sumber </span>[1]");
    expect(out).toContain("Matematika VII · hal. 40");
  });

  test("a half-streamed formula is plain text, not broken maths", () => {
    const out = html(answer("Jadi $ \\frac{1}{4} + ", "streaming"));
    expect(out).not.toContain("data-math");
    expect(out).toContain("Jadi $ \\frac{1}{4} + ");
  });

  test("money in an answer is not maths", () => {
    expect(html(answer("Harganya Rp5.000 dan $3 [1]."))).not.toContain("data-math");
  });
});
