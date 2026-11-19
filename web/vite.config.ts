import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [react(), tailwindcss()],
  server: {
    // The Rust server answers /v1; the dev server forwards to it.
    proxy: { "/v1": "http://localhost:8080" },
  },
});
