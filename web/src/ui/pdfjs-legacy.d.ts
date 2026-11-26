// pdfjs-dist ships types only for its modern build; the legacy build has the same API.
declare module "pdfjs-dist/legacy/build/pdf.mjs" {
  export * from "pdfjs-dist";
}
