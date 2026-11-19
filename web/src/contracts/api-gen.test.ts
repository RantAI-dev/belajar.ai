import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import openapiTS, { astToString, COMMENT_HEADER } from "openapi-typescript";

const webRoot = fileURLToPath(new URL("../..", import.meta.url));

test("api.gen.ts matches the committed openapi.json", async () => {
  // Regenerate exactly what scripts/gen-contracts.sh produces: openapi-typescript's own banner
  // plus the AST, then biome with the same config and the same file path (so the formatter picks
  // the repo's biome.json).
  const ast = await openapiTS(new URL("./openapi.json", import.meta.url));
  const generated = COMMENT_HEADER + astToString(ast);
  const biome = Bun.spawnSync({
    cmd: ["bunx", "biome", "format", "--stdin-file-path", "src/contracts/api.gen.ts"],
    cwd: webRoot,
    stdin: new Blob([generated]),
  });
  expect(biome.exitCode, `biome failed: ${biome.stderr.toString()}`).toBe(0);
  const formatted = biome.stdout.toString();
  const committed = readFileSync(new URL("./api.gen.ts", import.meta.url), "utf8");
  if (formatted !== committed) {
    // Say it before the diff, so the fix is the first thing a reader sees.
    console.error(
      "api.gen.ts is stale: it differs from what openapi.json generates.",
      "Run scripts/gen-contracts.sh and commit the result.",
    );
  }
  expect(formatted).toBe(committed);
});
