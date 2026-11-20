/**
 * Reads the colour tokens out of theme.css, so tests check the values the browser gets: light is
 * `@theme` plus `:root`, dark is light with the `[data-theme="dark"]` overrides on top. A value
 * that is `var(--other)` resolves within the same theme.
 */
export type ThemeName = "light" | "dark";
export type TokenMap = Readonly<Record<string, string>>;
export type ThemeTokens = Readonly<Record<ThemeName, TokenMap>>;

const BLOCK = /(@theme(?:\s+static)?|:root|\[data-theme="dark"\])\s*\{([^{}]*)\}/g;
const DECLARATION = /--([\w-]+)\s*:\s*([^;]+);/g;
const VAR = /^var\(--([\w-]+)\)$/;

const declarations = (body: string): Record<string, string> => {
  const out: Record<string, string> = {};
  for (const m of body.matchAll(DECLARATION)) {
    const [, name, value] = m;
    if (name !== undefined && value !== undefined) out[name] = value.trim().toLowerCase();
  }
  return out;
};

const resolve = (raw: Record<string, string>): TokenMap => {
  const out: Record<string, string> = {};
  for (const name of Object.keys(raw)) {
    let value = raw[name];
    // A chain longer than the token count is a cycle; leave it unresolved.
    for (let hops = 0; value !== undefined && hops <= Object.keys(raw).length; hops++) {
      const ref = value.match(VAR)?.[1];
      if (ref === undefined) break;
      value = raw[ref];
    }
    if (value !== undefined) out[name] = value;
  }
  return out;
};

export const parseThemeTokens = (css: string): ThemeTokens => {
  const text = css.replace(/\/\*[\s\S]*?\*\//g, "");
  const light: Record<string, string> = {};
  const darkOverrides: Record<string, string> = {};
  for (const m of text.matchAll(BLOCK)) {
    const [, selector, body = ""] = m;
    Object.assign(selector === '[data-theme="dark"]' ? darkOverrides : light, declarations(body));
  }
  return { light: resolve(light), dark: resolve({ ...light, ...darkOverrides }) };
};

/** The names a `[data-theme="dark"]` block overrides (for "no dark-only token" checks). */
export const darkOverrideNames = (css: string): readonly string[] => {
  const text = css.replace(/\/\*[\s\S]*?\*\//g, "");
  const names: string[] = [];
  for (const m of text.matchAll(BLOCK)) {
    if (m[1] === '[data-theme="dark"]') names.push(...Object.keys(declarations(m[2] ?? "")));
  }
  return names;
};
