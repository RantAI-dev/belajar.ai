// three.js (~140 kB gzip) must stay out of the first load: only scene.ts imports it, and only
// through import(), so the bundler gives it a chunk of its own.
import { expect, test } from "bun:test";
import { Glob } from "bun";

const SRC = new URL("../../../", import.meta.url).pathname;

test("three is imported only by the 3D scene, and the scene only lazily", async () => {
  const three: string[] = [];
  const staticScene: string[] = [];
  for await (const path of new Glob("**/*.{ts,tsx}").scan(SRC)) {
    if (/\.test\.tsx?$/.test(path)) continue;
    const text = await Bun.file(`${SRC}${path}`).text();
    if (/from\s+["']three["']/.test(text)) three.push(path);
    if (/from\s+["'][^"']*mascot3d\/scene["']/.test(text)) staticScene.push(path);
  }
  expect(three).toEqual(["features/auth/mascot3d/scene.ts"]);
  expect(staticScene).toEqual([]);
});
