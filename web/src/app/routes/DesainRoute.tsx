// The brand's living style sheet at /desain (no sign-in, not in the menu; the repo has no story
// tooling, so this page is the story). Public API, all from src/ui:
//   <Logo href? compact? className? />  <LogoMark size? />  <Wordmark className? />
//   <Mascot expression? mood?(deprecated) size? backdrop? inverse? animate? interactive?
//           onActivate? title? className? />   expressions: MASCOT_EXPRESSIONS
//   <MascotIcon kind size? title? className? />  kinds: MASCOT_ICON_KINDS
import { type ReactNode, useState } from "react";
import {
  Button,
  Logo,
  MASCOT_EXPRESSIONS,
  MASCOT_ICON_KINDS,
  Mascot,
  type MascotExpression,
  MascotIcon,
} from "../../ui";
import { useTheme } from "../theme";

function Section({ title, children }: { readonly title: string; readonly children: ReactNode }) {
  return (
    <section className="flex flex-col gap-4 rounded-panel border-2 border-line bg-surface p-5 sm:p-7">
      <h2 className="m-0 font-display text-[24px] font-semibold">{title}</h2>
      {children}
    </section>
  );
}

function Demo() {
  const [index, setIndex] = useState(0);
  const expression: MascotExpression = MASCOT_EXPRESSIONS[index] ?? "happy";
  return (
    <div className="flex flex-wrap items-center gap-6">
      <Mascot
        expression={expression}
        size={180}
        animate
        interactive
        title={`Maskot (${expression}). Tekan untuk ganti ekspresi.`}
        onActivate={() => setIndex((i) => (i + 1) % MASCOT_EXPRESSIONS.length)}
      />
      <p className="m-0 max-w-[340px] text-[17px] text-ink-muted">
        Gerakkan penunjuk: wajahnya mengikuti. Klik atau tekan Enter untuk ganti ekspresi (
        <strong className="text-ink">{expression}</strong>). Dengan "kurangi gerakan" hanya
        ekspresinya yang tampil.
      </p>
    </div>
  );
}

export default function DesainRoute() {
  const theme = useTheme();
  return (
    <main className="mx-auto flex max-w-[1100px] flex-col gap-6 px-4 py-8">
      <div className="flex flex-wrap items-center justify-between gap-4">
        <Logo />
        <div className="flex gap-2">
          <Button variant="secondary" onClick={() => void theme.setPreference("light")}>
            Terang
          </Button>
          <Button variant="secondary" onClick={() => void theme.setPreference("dark")}>
            Gelap
          </Button>
        </div>
      </div>
      <h1 className="m-0 font-display text-[32px] font-semibold">Desain: logo dan maskot</h1>
      <Section title="Logo">
        <div className="flex flex-wrap gap-4">
          <div className="rounded-card border-2 border-line bg-page p-5">
            <Logo />
          </div>
          <div data-theme="dark" className="rounded-card bg-page p-5">
            <Logo />
          </div>
        </div>
      </Section>
      <Section title="Ekspresi">
        <ul className="m-0 grid list-none grid-cols-2 gap-4 p-0 sm:grid-cols-3 lg:grid-cols-5">
          {MASCOT_EXPRESSIONS.map((e) => (
            <li key={e} className="flex flex-col items-center gap-1 text-center">
              <Mascot expression={e} size={120} />
              <Mascot expression={e} size={48} />
              <code className="text-[15px]">{e}</code>
            </li>
          ))}
        </ul>
      </Section>
      <Section title="Interaktif">
        <Demo />
      </Section>
      <Section title="Ikon">
        <ul className="m-0 flex list-none flex-wrap gap-6 p-0">
          {MASCOT_ICON_KINDS.map((k) => (
            <li key={k} className="flex flex-col items-center gap-2">
              <div className="flex items-end gap-2">
                <MascotIcon kind={k} size={20} />
                <MascotIcon kind={k} size={32} />
                <MascotIcon kind={k} size={40} />
              </div>
              <code className="text-[15px]">{k}</code>
            </li>
          ))}
        </ul>
      </Section>
    </main>
  );
}
