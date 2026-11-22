import { describe, expect, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import { EXPRESSIONS } from "./face";
import { Mascot } from "./Mascot";
import { MASCOT_ICON_KINDS, MascotIcon } from "./MascotIcon";

// useId differs per render; strip it so only the drawing is compared.
const drawing = (html: string) => html.replace(/id="[^"]*"|url\(#[^)]*\)/g, "");

describe("Mascot", () => {
  test("every expression draws something different", () => {
    const shapes = EXPRESSIONS.map((e) => drawing(renderToStaticMarkup(<Mascot expression={e} />)));
    expect(new Set(shapes).size).toBe(EXPRESSIONS.length);
  });

  test("the old moods map onto expressions", () => {
    const html = (el: React.ReactElement) => drawing(renderToStaticMarkup(el));
    expect(html(<Mascot mood="hmm" />)).toBe(html(<Mascot expression="confused" />));
    expect(html(<Mascot mood="celebrate" />)).toBe(html(<Mascot expression="celebration" />));
    expect(html(<Mascot mood="neutral" />)).toBe(html(<Mascot expression="cheerful" />));
  });

  test("decorative by default, an image with a title, a button only when it can be activated", () => {
    expect(renderToStaticMarkup(<Mascot />)).toContain('aria-hidden="true"');
    const img = renderToStaticMarkup(<Mascot title="Maskot senang" />);
    expect(img).toContain('role="img"');
    expect(img).toContain("<title>Maskot senang</title>");
    expect(renderToStaticMarkup(<Mascot interactive />)).not.toContain("tabindex");
    const button = renderToStaticMarkup(<Mascot interactive onActivate={() => {}} title="Sapa" />);
    expect(button).toContain('role="button"');
    expect(button).toContain('tabindex="0"');
    expect(button).toContain('aria-label="Sapa"');
  });

  test("two mascots on a page use different gradient ids", () => {
    const html = renderToStaticMarkup(
      <>
        <Mascot />
        <Mascot />
      </>,
    );
    const ids = [...html.matchAll(/linearGradient id="([^"]+)"/g)].map((m) => m[1]);
    expect(ids).toHaveLength(2);
    expect(ids[0]).not.toBe(ids[1]);
  });

  test("the old book artwork is gone", () => {
    expect(renderToStaticMarkup(<Mascot />)).not.toContain("mascot-book");
  });
});

describe("MascotIcon", () => {
  test("every kind draws something different", () => {
    const shapes = MASCOT_ICON_KINDS.map((k) =>
      drawing(renderToStaticMarkup(<MascotIcon kind={k} />)),
    );
    expect(new Set(shapes).size).toBe(MASCOT_ICON_KINDS.length);
  });
});
