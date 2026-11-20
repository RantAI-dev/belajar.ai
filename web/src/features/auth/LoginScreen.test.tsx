import { describe, expect, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import { GoogleSignupScreen } from "./GoogleSignupScreen";
import { LoginScreen } from "./LoginScreen";
import { RegisterScreen } from "./RegisterScreen";

const screens = {
  login: renderToStaticMarkup(
    <LoginScreen onSubmit={() => {}} registerHref="/daftar" googleHref="/v1/auth/google/start" />,
  ),
  register: renderToStaticMarkup(<RegisterScreen onSubmit={() => {}} loginHref="/masuk" />),
  google: renderToStaticMarkup(<GoogleSignupScreen suggestedNickname="Budi" onSubmit={() => {}} />),
};

describe("sign-in screens", () => {
  test("no demo password hint ships on any of them", () => {
    for (const html of Object.values(screens)) {
      expect(html).not.toContain("belajar123");
      expect(html).not.toMatch(/Demo:/i);
    }
  });

  test("login asks for a username and a password with a show/hide toggle", () => {
    const html = screens.login;
    expect(html).toContain('autoComplete="username"');
    expect(html).toContain("Nama pengguna");
    expect(html).not.toContain('type="email"');
    expect(html).toContain('type="password"');
    expect(html).toContain('aria-pressed="false"');
    expect(html).toContain('aria-label="Tampilkan kata sandi"');
    expect(html).toContain("Belum punya akun?");
  });

  test("the Google button shows only when Google sign-in is configured", () => {
    expect(screens.login).toContain("Masuk dengan Google");
    const without = renderToStaticMarkup(
      <LoginScreen onSubmit={() => {}} registerHref="/daftar" />,
    );
    expect(without).not.toContain("Masuk dengan Google");
  });

  test("every screen has the mascot stage and the brand", () => {
    for (const html of Object.values(screens)) {
      expect(html).toContain('aria-label="Maskot belajar.ai"');
      expect(html).toContain("Belajar Lebih Mudah, Lebih Pintar");
    }
  });

  test("a login error is read out at once", () => {
    const html = renderToStaticMarkup(
      <LoginScreen
        onSubmit={() => {}}
        registerHref="/daftar"
        error="Nama pengguna atau kata sandi salah."
        outcome={{ kind: "failure", key: 1 }}
      />,
    );
    expect(html).toContain('role="alert"');
  });
});
