import { describe, expect, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import { ChapterMapScreen, type ChapterMapScreenProps } from "./ChapterMapScreen";
import type { ChapterNode } from "./types";

const props = (chapters: readonly ChapterNode[]): ChapterMapScreenProps => ({
  view: { subjectId: "ipa", subjectIds: ["ipa", "mtk", "bing"], chapters },
  selectedChapterId: null,
  onSelectChapter: () => {},
  subjectHref: (id) => `/belajar/${id}`,
  chapterHref: (id) => `/belajar/ipa/${id}`,
  tanyaHref: (id) => `/tanya?subject=${id}`,
});

const chapter = (over: Partial<ChapterNode> = {}): ChapterNode => ({
  id: "bab-1",
  number: 1,
  bookNumber: 1,
  title: "Sistem Organisasi Kehidupan Makhluk Hidup",
  page: 6,
  documentTitle: "Ilmu Pengetahuan Alam Kelas VII Semester 2",
  topics: [
    { title: "Sel sebagai Unit Struktural dan Fungsional Kehidupan", page: 9 },
    { title: "Jaringan-Jaringan pada Hewan dan Tumbuhan", page: 17 },
  ],
  stars: 0,
  status: "current",
  ...over,
});

describe("ChapterMapScreen with real chapters", () => {
  test("an empty chapters list shows a clear empty state and a way into Tanya", () => {
    const out = renderToStaticMarkup(<ChapterMapScreen {...props([])} />);
    expect(out).toContain("Daftar bab untuk mapel ini belum tersedia");
    expect(out).toContain('href="/tanya?subject=ipa"');
    expect(out).toContain("Mulai Tanya gratis");
  });

  test("a subject with several books shows each book title and the selected chapter's materi", () => {
    const out = renderToStaticMarkup(
      <ChapterMapScreen
        {...props([
          chapter(),
          chapter({
            id: "bab-7",
            number: 7,
            bookNumber: 1,
            title: "Hakikat Ilmu Sains dan Metode Ilmiah",
            documentTitle: "Ilmu Pengetahuan Alam untuk SMPMTs Kelas VII Edisi Revisi",
            topics: [],
            status: "todo",
          }),
        ])}
      />,
    );
    expect(out).toContain("Sistem Organisasi Kehidupan Makhluk Hidup");
    expect(out).toContain("Hakikat Ilmu Sains dan Metode Ilmiah");
    expect(out).toContain("Ilmu Pengetahuan Alam Kelas VII Semester 2");
    expect(out).toContain("Ilmu Pengetahuan Alam untuk SMPMTs Kelas VII Edisi Revisi");
    expect(out).toContain("Materi dalam bab ini");
    expect(out).toContain("Sel sebagai Unit Struktural dan Fungsional Kehidupan");
    expect(out).toContain("hal. 10");
  });

  test("with one book the book title is not repeated on every node", () => {
    const out = renderToStaticMarkup(<ChapterMapScreen {...props([chapter()])} />);
    expect(out).not.toContain("Ilmu Pengetahuan Alam Kelas VII Semester 2");
  });
});
