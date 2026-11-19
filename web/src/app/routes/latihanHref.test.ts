import { expect, test } from "bun:test";
import { latihanHref } from "./latihanHref";

test("latihanHref fills subject and topic, and leaves out what is unknown", () => {
  expect(latihanHref("ipa", "Suhu, Kalor, dan Pemuaian")).toBe(
    "/latihan?mapel=ipa&topik=Suhu%2C+Kalor%2C+dan+Pemuaian",
  );
  expect(latihanHref(null, "")).toBe("/latihan");
});
