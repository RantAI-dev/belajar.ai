import { readdirSync, readFileSync } from "node:fs";
import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig, type Plugin } from "vite";

/** pdf.js fetches these at run time, by URL: copy them into the build (see src/ui/pdf.ts). */
const pdfjsAssets = (): Plugin => ({
  name: "pdfjs-assets",
  apply: "build",
  generateBundle() {
    for (const dir of ["standard_fonts", "wasm"]) {
      const from = new URL(`./node_modules/pdfjs-dist/${dir}/`, import.meta.url);
      for (const file of readdirSync(from)) {
        const source = readFileSync(new URL(file, from));
        this.emitFile({ type: "asset", fileName: `pdfjs/${dir}/${file}`, source });
      }
    }
  },
});

export default defineConfig({
  plugins: [react(), tailwindcss(), pdfjsAssets()],
  server: {
    // The Rust server answers /v1; the dev server forwards to it.
    proxy: { "/v1": "http://localhost:8080" },
  },
});
