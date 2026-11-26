import { describe, expect, test } from "bun:test";
import { bookKind, clampPage, pageAt, renderScale, stepZoom } from "./bookView";

describe("book drawer", () => {
  test("zoom steps through the levels and stops at both ends", () => {
    expect(stepZoom(1, 1)).toBe(1.5);
    expect(stepZoom(3, 1)).toBe(3);
    expect(stepZoom(1.5, -1)).toBe(1);
    expect(stepZoom(1, -1)).toBe(1);
  });

  test("the response's content type decides between the PDF viewer and a picture", () => {
    expect(bookKind("application/pdf")).toBe("pdf");
    expect(bookKind("Application/PDF; charset=binary")).toBe("pdf");
    expect(bookKind("image/svg+xml")).toBe("image");
    expect(bookKind("image/jpeg")).toBe("image");
    expect(bookKind("text/html")).toBeNull();
    expect(bookKind(null)).toBeNull();
  });

  test("a page is drawn sharp, but never bigger than a phone can hold", () => {
    // 400 CSS px on a 2x screen, an A4-ish page 600 points wide.
    expect(renderScale(400, 600, 1.4, 2)).toBeCloseTo(800 / 600);
    // A 3x screen is drawn at 2x.
    expect(renderScale(400, 600, 1.4, 3)).toBeCloseTo(800 / 600);
    // Zoomed far in: capped at 4 million pixels.
    const scale = renderScale(2400, 600, 1.4, 2);
    const width = scale * 600;
    expect(width * width * 1.4).toBeCloseTo(4_000_000, -2);
    expect(renderScale(0, 600, 1.4, 2)).toBe(1);
  });

  test("the page at the top of the view is the last one that starts above it", () => {
    const tops = [0, 500, 1000, 1500];
    expect(pageAt(tops, 0)).toBe(1);
    expect(pageAt(tops, 499)).toBe(2);
    expect(pageAt(tops, 1200)).toBe(3);
    expect(pageAt(tops, 9000)).toBe(4);
    expect(pageAt([], 100)).toBe(1);
  });

  test("a cited page the book does not have opens the first page", () => {
    expect(clampPage(44, 160)).toBe(44);
    expect(clampPage(161, 160)).toBe(1);
    expect(clampPage(0, 160)).toBe(1);
    expect(clampPage(undefined, 160)).toBe(1);
  });
});
