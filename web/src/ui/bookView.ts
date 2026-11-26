/** Zoom levels of the book drawer, as a share of the drawer's width. */
export const ZOOMS = [1, 1.5, 2, 3] as const;
export type Zoom = (typeof ZOOMS)[number];

/** The next zoom level in, or out; the ends stay put. */
export const stepZoom = (zoom: Zoom, direction: 1 | -1): Zoom => {
  const i = ZOOMS.indexOf(zoom);
  return ZOOMS[Math.min(ZOOMS.length - 1, Math.max(0, i + direction))] ?? zoom;
};

/** What the drawer can show, from the response's own `Content-Type`. */
export const bookKind = (contentType: string | null): "pdf" | "image" | null => {
  const type = (contentType ?? "").split(";")[0]?.trim().toLowerCase() ?? "";
  if (type === "application/pdf") return "pdf";
  if (type.startsWith("image/")) return "image";
  return null;
};

// One page's canvas holds at most this many pixels: a phone keeps a few pages drawn at once, and
// a 3x zoom on a sharp screen would otherwise ask for ~70 MB per page.
const MAX_CANVAS_PIXELS = 4_000_000;

/**
 * The pdf.js scale that draws a page `pageWidth` points wide at `cssWidth` CSS pixels, sharp on
 * a `dpr` screen, but never with more than MAX_CANVAS_PIXELS pixels (`aspect` is height/width).
 */
export const renderScale = (
  cssWidth: number,
  pageWidth: number,
  aspect: number,
  dpr: number,
): number => {
  if (cssWidth <= 0 || pageWidth <= 0 || aspect <= 0) return 1;
  const wanted = (cssWidth * Math.min(dpr, 2)) / pageWidth;
  const widthPx = wanted * pageWidth;
  const pixels = widthPx * widthPx * aspect;
  return pixels <= MAX_CANVAS_PIXELS ? wanted : wanted * Math.sqrt(MAX_CANVAS_PIXELS / pixels);
};

/**
 * The 1-based page at the top of the view: the last page whose top is at or above `scrollTop`
 * (with a little slack, so a page just scrolled to counts). `tops` are in page order.
 */
export const pageAt = (tops: readonly number[], scrollTop: number): number => {
  let page = 1;
  tops.forEach((top, i) => {
    if (top <= scrollTop + 48) page = i + 1;
  });
  return page;
};

/** A page number the book has: out-of-range or missing numbers fall back to the first page. */
export const clampPage = (page: number | undefined, pageCount: number): number =>
  page !== undefined && Number.isInteger(page) && page >= 1 && page <= pageCount ? page : 1;
