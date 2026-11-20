import { describe, expect, test } from "bun:test";
import { latihanHref, shouldAutostart } from "./href";

describe("latihanHref", () => {
  test("fills subject and topic, and leaves out what is unknown", () => {
    expect(latihanHref("ipa", "Pemuaian")).toBe("/latihan?mapel=ipa&topik=Pemuaian");
    expect(latihanHref(null, "")).toBe("/latihan");
  });

  test("asks the setup to start straight away only when there is a topic", () => {
    expect(latihanHref("mtk", "Pecahan", { start: true })).toBe(
      "/latihan?mapel=mtk&topik=Pecahan&mulai=1",
    );
    expect(latihanHref("mtk", "", { start: true })).toBe("/latihan?mapel=mtk");
  });
});

describe("shouldAutostart", () => {
  test("needs the flag and a non-blank topic", () => {
    expect(shouldAutostart(new URLSearchParams("mapel=ipa&topik=Gaya&mulai=1"))).toBe(true);
    expect(shouldAutostart(new URLSearchParams("mapel=ipa&topik=Gaya"))).toBe(false);
    expect(shouldAutostart(new URLSearchParams("mapel=ipa&topik=%20&mulai=1"))).toBe(false);
    expect(shouldAutostart(new URLSearchParams("mulai=0&topik=Gaya"))).toBe(false);
  });
});
