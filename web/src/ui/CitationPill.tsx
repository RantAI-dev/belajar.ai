import { BookIcon } from "./icons";

/** The source chips' teal. White text needs the darker shade (`CITE.strong`) to pass 4.5:1. */
export const CITE = {
  line: "#388ca1",
  soft: "#e6f5f7",
  strong: "#1f5f6e",
  /** The yellow marker on a cited sentence, matching the highlight on the page image. */
  marker: "#ffe58a",
} as const;

export type CitationPillProps = {
  /** e.g. "hal. 78" or "IPA hal. 78". */
  readonly label: string;
  /** e.g. "Sumber: IPA Buku Siswa Kelas VII, halaman 78". */
  readonly ariaLabel: string;
  readonly pressed: boolean;
  readonly onPress: () => void;
};

/** An inline "book + page" citation inside answer text; pressing it opens the page preview. */
export function CitationPill({ label, ariaLabel, pressed, onPress }: CitationPillProps) {
  return (
    <button
      type="button"
      aria-pressed={pressed}
      aria-label={ariaLabel}
      onClick={onPress}
      className="relative mx-0.5 inline-flex cursor-pointer items-center gap-1 whitespace-nowrap rounded-full border-2 px-2 py-px align-[0.1em] text-[13px] font-extrabold leading-[1.3] transition-colors motion-reduce:transition-none"
      style={
        pressed
          ? { background: CITE.strong, borderColor: CITE.strong, color: "#ffffff" }
          : { background: CITE.soft, borderColor: CITE.line, color: "var(--color-ink)" }
      }
    >
      {/* A 48px-tall tap target that does not spread the line. */}
      <span aria-hidden="true" className="absolute inset-x-0 -inset-y-3" />
      <BookIcon size={14} />
      {label}
    </button>
  );
}
