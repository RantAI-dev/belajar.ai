import { describe, expect, test } from "bun:test";
import { type ChapterDto, chapterNodes, lessonStep, subjectEntry } from "./fromApi";

const ch = (number: number, over: Partial<ChapterDto> = {}): ChapterDto => ({
  id: `bab-${number}`,
  number,
  bookNumber: number,
  title: `Bab ${number}`,
  page: 0,
  documentId: "dok",
  documentTitle: "Buku IPA",
  topics: [],
  stars: 0,
  step: null,
  isCompleted: false,
  lessonId: null,
  ...over,
});

describe("chapterNodes", () => {
  test("finished chapters are done, the first unfinished is current", () => {
    const nodes = chapterNodes([
      ch(2),
      ch(1, { isCompleted: true, stars: 3, step: "done" }),
      ch(3),
    ]);
    expect(nodes.map((n) => [n.id, n.status, n.stars])).toEqual([
      ["bab-1", "done", 3],
      ["bab-2", "current", 0],
      ["bab-3", "todo", 0],
    ]);
  });

  test("a chapter the student already started is current, even when an earlier one is not", () => {
    const nodes = chapterNodes([ch(1), ch(2, { step: "check", lessonId: "l2" })]);
    expect(nodes.map((n) => n.status)).toEqual(["todo", "current"]);
  });

  test("stars stay within 0..3", () => {
    expect(chapterNodes([ch(1, { stars: 9 })])[0]?.stars).toBe(3);
  });

  test("real book data is carried through: book number, page, book title and materi", () => {
    const node = chapterNodes([
      ch(7, {
        bookNumber: 1,
        title: "Sistem Organisasi Kehidupan Makhluk Hidup",
        page: 6,
        documentTitle: "Ilmu Pengetahuan Alam Kelas VII Semester 2",
        topics: [
          { title: "Sel sebagai Unit Struktural dan Fungsional Kehidupan", page: 9 },
          { title: "Jaringan-Jaringan pada Hewan dan Tumbuhan", page: null },
        ],
      }),
    ])[0];
    expect(node).toMatchObject({
      number: 7,
      bookNumber: 1,
      title: "Sistem Organisasi Kehidupan Makhluk Hidup",
      page: 6,
      documentTitle: "Ilmu Pengetahuan Alam Kelas VII Semester 2",
    });
    expect(node?.topics).toEqual([
      { title: "Sel sebagai Unit Struktural dan Fungsional Kehidupan", page: 9 },
      { title: "Jaringan-Jaringan pada Hewan dan Tumbuhan", page: null },
    ]);
  });
});

describe("subjectEntry", () => {
  test("counts finished chapters and half of a started one", () => {
    const e = subjectEntry("ipa", [
      ch(1, { isCompleted: true }),
      ch(2, { step: "explain" }),
      ch(3),
      ch(4),
    ]);
    expect(e).toEqual({
      subjectId: "ipa",
      progressLabel: "Bab 1 dari 4 selesai",
      progress: 38,
      href: "/belajar/ipa",
    });
  });

  test("labels a subject not started, started, loading and without chapters", () => {
    expect(subjectEntry("mtk", [ch(1), ch(2)]).progressLabel).toBe("Belum mulai");
    expect(subjectEntry("mtk", [ch(1, { step: "quiz" })]).progressLabel).toBe("Sedang belajar");
    expect(subjectEntry("mtk", null)).toMatchObject({ progressLabel: "…", progress: 0 });
    expect(subjectEntry("mtk", [])).toMatchObject({ progressLabel: "Segera hadir", progress: 0 });
  });
});

test("lessonStep reads the server's step names", () => {
  expect(lessonStep("check")).toBe("check");
  expect(lessonStep("done")).toBe("done");
  expect(lessonStep(null)).toBe("explain");
  expect(lessonStep("weird")).toBe("explain");
});
