import { expect, test } from "bun:test";
import { CITE } from "./CitationPill";
import { contrastRatio } from "./contrast";

test("pill and marker text stay readable (>= 4.5:1)", () => {
  const ink = "#10233a";
  expect(contrastRatio("#ffffff", CITE.strong)).toBeGreaterThanOrEqual(4.5);
  expect(contrastRatio(ink, CITE.soft)).toBeGreaterThanOrEqual(4.5);
  expect(contrastRatio(ink, CITE.marker)).toBeGreaterThanOrEqual(4.5);
});
