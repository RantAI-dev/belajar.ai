import { describe, expect, test } from "bun:test";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { build, type Rolldown } from "vite";

// The first load is the entry chunk plus every chunk it imports statically. Mermaid, KaTeX and the
// rest must stay behind dynamic import(); this fails when the first load grows past the budget.
// Measured 2026-11-20: 125.9 kB gzip. Raise it only for a reason you can name in the PR.
const ENTRY_GZIP_BUDGET_KB = 140;

const firstLoad = (chunks: Rolldown.OutputChunk[]): Rolldown.OutputChunk[] => {
  const byName = new Map(chunks.map((c) => [c.fileName, c]));
  const seen = new Map<string, Rolldown.OutputChunk>();
  const visit = (chunk: Rolldown.OutputChunk): void => {
    if (seen.has(chunk.fileName)) return;
    seen.set(chunk.fileName, chunk);
    for (const name of chunk.imports) {
      const next = byName.get(name);
      if (next !== undefined) visit(next);
    }
  };
  for (const chunk of chunks) if (chunk.isEntry) visit(chunk);
  return [...seen.values()];
};

const gzipKb = (code: string): number => Bun.gzipSync(new TextEncoder().encode(code)).length / 1000;

describe("bundle budget", () => {
  test("the first load stays under its gzip budget and holds no mermaid or katex", async () => {
    const outDir = mkdtempSync(join(tmpdir(), "belajar-bundle-"));
    // bun test sets NODE_ENV=test, which would bundle React's development build.
    const nodeEnv = process.env.NODE_ENV;
    process.env.NODE_ENV = "production";
    try {
      const result = await build({
        root: join(import.meta.dir, "../.."),
        logLevel: "silent",
        build: { outDir, emptyOutDir: true },
      });
      const outputs = (Array.isArray(result) ? result : [result]).flatMap((r) =>
        "output" in r ? r.output : [],
      );
      const chunks = outputs.filter((o): o is Rolldown.OutputChunk => o.type === "chunk");
      const initial = firstLoad(chunks);
      expect(initial.length).toBeGreaterThan(0);

      const heavy = initial
        .flatMap((c) => c.moduleIds)
        .filter((m) => /node_modules\/(mermaid|katex|elkjs|cytoscape)\//.test(m));
      expect(heavy).toEqual([]);

      const total = initial.reduce((sum, c) => sum + gzipKb(c.code), 0);
      expect(total).toBeLessThanOrEqual(ENTRY_GZIP_BUDGET_KB);
    } finally {
      if (nodeEnv === undefined) delete process.env.NODE_ENV;
      else process.env.NODE_ENV = nodeEnv;
      rmSync(outDir, { recursive: true, force: true });
    }
  }, 60_000);
});
