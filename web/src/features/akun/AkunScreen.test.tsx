import { describe, expect, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import { MESSAGES } from "../../contracts/messages";
import { ok } from "../../contracts/result";
import { AkunScreen } from "./AkunScreen";
import { AKUN_FIXTURE } from "./fixtures";
import { GOOGLE_LINK_HREF, linkNotice } from "./googleLink";
import type { AkunView } from "./types";

const done = async () => ok(undefined);

const render = (view: Partial<AkunView>, search = "") =>
  renderToStaticMarkup(
    <AkunScreen
      view={{ ...AKUN_FIXTURE, ...view }}
      onSaveProfile={done}
      onChangePassword={done}
      onLogout={done}
      onThemeChange={done}
      googleNotice={linkNotice(search)}
    />,
  );

describe("Akun: password", () => {
  test("a student with a password sees the change-password form", () => {
    const out = render({ hasPassword: true });
    expect(out).toContain("Kata sandi sekarang");
    expect(out).toContain("Ganti kata sandi");
    expect(out).not.toContain(MESSAGES.noPasswordWithGoogle);
  });

  test("a Google-only student sees a note instead of the form", () => {
    const out = render({ hasPassword: false, isGoogleLinked: true });
    expect(out).toContain(MESSAGES.noPasswordWithGoogle);
    expect(out).not.toContain("Kata sandi sekarang");
    expect(out).not.toContain('type="password"');
  });
});

describe("Akun: Google", () => {
  test("not linked and Google on: the link button goes to the server's start route", () => {
    const out = render({ isGoogleLinked: false, isGoogleEnabled: true });
    expect(out).toContain("Hubungkan dengan Google");
    expect(out).toContain(`href="${GOOGLE_LINK_HREF}"`);
    expect(out).not.toContain("Terhubung dengan Google");
  });

  test("linked: says so, and offers no second link", () => {
    const out = render({ isGoogleLinked: true, isGoogleEnabled: true });
    expect(out).toContain("Terhubung dengan Google");
    expect(out).not.toContain("Hubungkan dengan Google");
    expect(out).not.toContain(GOOGLE_LINK_HREF);
  });

  test("Google off on this server: no Google section at all", () => {
    const out = render({ isGoogleLinked: false, isGoogleEnabled: false });
    expect(out).not.toContain("Akun Google");
    expect(out).not.toContain(GOOGLE_LINK_HREF);
  });

  test("each callback outcome shows its own Indonesian sentence", () => {
    const outcomes = ["linked", "conflict", "cancelled", "unavailable", "failed"] as const;
    const texts = outcomes.map((o) => linkNotice(`?google=${o}`)?.text);
    for (const t of texts) expect(t).toBeString();
    expect(new Set(texts).size).toBe(outcomes.length);
    expect(linkNotice("?google=linked")?.tone).toBe("success");
    expect(linkNotice("?google=conflict")?.tone).toBe("error");
    expect(linkNotice("")).toBeUndefined();
    expect(linkNotice("?google=toString")).toBeUndefined();

    const out = render({ isGoogleLinked: false }, "?google=conflict");
    expect(out).toContain(linkNotice("?google=conflict")?.text ?? "missing");
    expect(out).toContain('role="alert"');
    const linked = render({ isGoogleLinked: true }, "?google=linked");
    expect(linked).toContain(linkNotice("?google=linked")?.text ?? "missing");
    expect(linked).toContain('role="status"');
  });
});
