import { describe, expect, test } from "bun:test";
import { googleNotice, loginWithNotice } from "./google";

describe("googleNotice", () => {
  test("each outcome the server sends has its own Indonesian sentence", () => {
    const outcomes = ["cancelled", "conflict", "unavailable", "failed"] as const;
    const sentences = outcomes.map((o) => googleNotice(`?google=${o}`));
    for (const s of sentences) expect(s).toBeString();
    expect(new Set(sentences).size).toBe(outcomes.length);
    expect(googleNotice("?google=cancelled")).toContain("dibatalkan");
  });

  test("no parameter, or one we do not know, shows nothing", () => {
    expect(googleNotice("")).toBeUndefined();
    expect(googleNotice("?ke=%2Fakun")).toBeUndefined();
    expect(googleNotice("?google=<script>")).toBeUndefined();
    expect(googleNotice("?google=toString")).toBeUndefined();
  });

  test("the onboarding step's way back round-trips", () => {
    expect(loginWithNotice("conflict")).toBe("/masuk?google=conflict");
    expect(googleNotice(loginWithNotice("failed").slice("/masuk".length))).toBeString();
  });
});
