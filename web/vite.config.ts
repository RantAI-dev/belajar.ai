import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [react(), tailwindcss()],
  build: {
    // Only mermaid's lazy-of-lazy chunks pass 500 kB (elkjs 1.5 MB, @mermaid-js/parser 0.6 MB): each is
    // one prebundled module that cannot be split, never in the first load. src/app/bundle.test.ts
    // budgets what is.
    chunkSizeWarningLimit: 1600,
  },
  server: {
    // The Rust server answers /v1; the dev server forwards to it.
    proxy: { "/v1": "http://localhost:8080" },
  },
});
