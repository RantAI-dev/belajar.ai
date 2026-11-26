import type { PDFDocumentProxy } from "pdfjs-dist";

type PdfJs = typeof import("pdfjs-dist/legacy/build/pdf.mjs");

let loading: Promise<PdfJs> | null = null;

// The build copies these out of pdfjs-dist (vite.config.ts); the dev server reads them in place.
const ASSETS = import.meta.env.DEV ? "/node_modules/pdfjs-dist/" : "/pdfjs/";

/** pdf.js is large: it is fetched only when a student opens a book. */
const loadPdfJs = (): Promise<PdfJs> => {
  loading ??= Promise.all([
    // The legacy build: the modern one needs this year's JavaScript (Map.getOrInsertComputed,
    // ...), which the older phones many students use do not have.
    import("pdfjs-dist/legacy/build/pdf.mjs"),
    import("pdfjs-dist/legacy/build/pdf.worker.min.mjs?url"),
  ]).then(
    ([pdfjs, worker]) => {
      pdfjs.GlobalWorkerOptions.workerSrc = worker.default;
      return pdfjs;
    },
    (e: unknown) => {
      loading = null;
      return Promise.reject(e);
    },
  );
  return loading;
};

export type PdfDocument = PDFDocumentProxy;

/** Opens a PDF from its bytes. The bytes are handed to the worker, so `data` is unusable after. */
export const openPdf = async (data: ArrayBuffer): Promise<PdfDocument> => {
  const pdfjs = await loadPdfJs();
  return pdfjs.getDocument({
    data: new Uint8Array(data),
    // Books that do not embed their fonts, and scans compressed as JPEG 2000 or JBIG2.
    standardFontDataUrl: `${ASSETS}standard_fonts/`,
    wasmUrl: `${ASSETS}wasm/`,
  }).promise;
};
