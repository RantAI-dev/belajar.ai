// Mechanical architecture rules for web/src. Adapted from the RantAI guidelines
// (stacks/typescript/check-boundaries.ts) for .tsx and this app's layers. Each message says how to fix it.
//
//   contracts  <-  ui  <-  features/<slice>  <-  app
import { Glob } from "bun";

const failures: string[] = [];
const glob = new Glob("src/**/*.{ts,tsx}");

const layerOf = (path: string): string | undefined => path.match(/^src\/([^/]+)\//)?.[1];
const importedLayer = (spec: string): string | undefined =>
  spec.match(/(?:^|\/)(contracts|ui|features|app)(?:\/|$)/)?.[1];

const allowed: Record<string, string[]> = {
  contracts: [],
  ui: ["contracts"],
  features: ["contracts", "ui", "features"],
  app: ["contracts", "ui", "features", "app"],
};

for await (const path of glob.scan(".")) {
  const text = await Bun.file(path).text();
  const isTest = /\.test\.tsx?$/.test(path);
  const layer = layerOf(path);

  text.split("\n").forEach((line, i) => {
    const where = `${path}:${i + 1}`;
    const code = line.replace(/\/\/.*$/, "");

    if (path.startsWith("src/features/") && !isTest && /\bthrow\b/.test(code)) {
      failures.push(
        `${where} uses throw. Feature code returns Result<T, E>: use err(...) from src/contracts/result.ts.`,
      );
    }

    const spec = code.match(/from\s+["']([^"']+)["']/)?.[1];
    if (spec === undefined || !spec.startsWith(".")) return;

    const target = importedLayer(spec);
    if (layer !== undefined && target !== undefined && target !== layer) {
      const ok = allowed[layer] ?? [];
      if (!ok.includes(target)) {
        failures.push(
          `${where} imports ${target} from ${layer}. Allowed for ${layer}: ${ok.join(", ") || "nothing"}. Move the shared code down to src/contracts or src/ui.`,
        );
      }
    }

    const slice = path.match(/^src\/features\/([^/]+)\//)?.[1];
    const other = spec.match(/^\.\.\/([^./][^/]*)\/.+$/)?.[1];
    if (slice !== undefined && other !== undefined && other !== slice && layer === "features") {
      failures.push(
        `${where} reaches into slice "${other}" internals (${spec}). Import from "../${other}" (its index.ts) instead.`,
      );
    }
  });
}

if (failures.length > 0) {
  console.error(failures.join("\n"));
  console.error(`\nboundaries: ${failures.length} failure(s)`);
  process.exit(1);
}
console.log("boundaries: ok");
