import { ButtonLink, Mascot } from "../../ui";

export function NotFound() {
  return (
    <div className="mx-auto flex max-w-[520px] flex-col items-center gap-4 py-10 text-center">
      <Mascot mood="hmm" size={120} />
      <h1 className="m-0 font-display text-[28px] font-semibold">Hmm, halaman ini tidak ada.</h1>
      <p className="m-0 text-[17px] font-semibold text-ink-muted">
        Mungkin alamatnya salah ketik. Yuk, kembali ke beranda.
      </p>
      <ButtonLink href="/" size="lg">
        Ke beranda
      </ButtonLink>
    </div>
  );
}
