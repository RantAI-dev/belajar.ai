import { describe, expect, test } from "bun:test";
import { diagramConfig, isValidDiagram } from "./diagram";

describe("mermaid diagrams", () => {
  // Without labels: mermaid sanitises label text with DOMPurify, which needs a browser DOM.
  test("a valid flowchart parses", async () => {
    expect(await isValidDiagram("graph TD\n  Air --> Uap\n  Uap --> Awan")).toBe(true);
  });

  test("an invalid diagram does not parse, and does not throw", async () => {
    expect(await isValidDiagram("graph TD\n  A -->")).toBe(false);
    expect(await isValidDiagram("bukan diagram")).toBe(false);
  });

  test("the config is strict and takes its colours from the theme tokens", () => {
    const tokens: Record<string, string> = {
      "--color-primary": "#0069a8",
      "--color-primary-soft": "#e0f0fd",
      "--color-ink": "#10233a",
    };
    const config = diagramConfig((name) => tokens[name] ?? "");
    expect(config.securityLevel).toBe("strict");
    expect(config.startOnLoad).toBe(false);
    expect(config.htmlLabels).toBe(false);
    expect(config.themeVariables).toMatchObject({
      primaryColor: "#e0f0fd",
      primaryBorderColor: "#0069a8",
      primaryTextColor: "#10233a",
    });
    // A token that does not exist is left to mermaid rather than set to "".
    expect(Object.values(config.themeVariables ?? {})).not.toContain("");
  });

  test("dark tokens give dark diagram colours", () => {
    const config = diagramConfig((name) => (name === "--color-ink" ? "#f3f9ff" : ""));
    expect(config.themeVariables).toEqual({
      primaryTextColor: "#f3f9ff",
      tertiaryTextColor: "#f3f9ff",
      textColor: "#f3f9ff",
      titleColor: "#f3f9ff",
    });
  });
});
