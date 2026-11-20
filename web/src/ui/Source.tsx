import { BookIcon } from "./icons";

export type SourceChipProps = {
  readonly label: string;
  readonly pressed: boolean;
  readonly onPressedChange: (next: boolean) => void;
  readonly color?: string;
};

/** A book-page citation; pressing it opens the page preview. */
export function SourceChip({
  label,
  pressed,
  onPressedChange,
  color = "var(--color-subject-ipa)",
}: SourceChipProps) {
  return (
    <button
      type="button"
      aria-pressed={pressed}
      onClick={() => onPressedChange(!pressed)}
      className={`pressable inline-flex min-h-12 cursor-pointer items-center gap-2 rounded-chip border-2 px-3.5 text-left text-[15px] font-extrabold text-ink ${pressed ? "bg-surface-alt" : "border-line-strong bg-surface hover:bg-page"}`}
      style={pressed ? { borderColor: color } : undefined}
    >
      <span style={{ color }}>
        <BookIcon size={18} />
      </span>
      {label}
    </button>
  );
}

export type PagePreviewProps = {
  /** e.g. "IPA Kelas VII Semester 1 · Pemuaian · halaman 112" */
  readonly title: string;
  readonly note?: string;
  /** A picture of the book page. Without one a sketch of a page stands in. */
  readonly imageUrl?: string;
  /** Without one there is no "open the full page" link. */
  readonly fullPageHref?: string;
};

export function PagePreview({
  title,
  note = "Bagian yang kupakai ditandai kuning.",
  imageUrl,
  fullPageHref,
}: PagePreviewProps) {
  return (
    <div className="flex flex-wrap items-start gap-3.5 rounded-[16px] border-2 border-dashed border-line-sky bg-page p-3">
      {imageUrl !== undefined ? (
        <img
          src={imageUrl}
          alt={`Halaman buku: ${title}`}
          className="aspect-[3/4] w-[150px] rounded-[10px] border-2 border-line bg-paper object-cover"
        />
      ) : (
        <div
          aria-hidden="true"
          className="flex aspect-[3/4] w-[150px] flex-col gap-1.5 rounded-[10px] border-2 border-line bg-paper p-3"
        >
          <span className="h-2 w-[70%] rounded bg-paper-line" />
          <span className="h-1.5 w-[95%] rounded bg-paper-line-soft" />
          <span className="h-1.5 w-[90%] rounded bg-paper-mark" />
          <span className="h-1.5 w-[80%] rounded bg-paper-mark" />
          <span className="grow rounded-md bg-paper-fill" />
        </div>
      )}
      <div className="flex min-w-[180px] flex-1 flex-col gap-2">
        <span className="text-[16px] font-extrabold">{title}</span>
        <span className="text-[15px] font-semibold text-ink-muted">{note}</span>
        {/* A plain link: the full page is a file, not an app route. */}
        {fullPageHref !== undefined && (
          <a
            href={fullPageHref}
            target="_blank"
            rel="noopener"
            className="inline-flex min-h-12 items-center self-start rounded-[12px] border-2 border-line-sky bg-surface px-4 text-[15px] font-extrabold hover:bg-primary-soft"
          >
            Buka halaman penuh
            <span className="sr-only"> (di tab baru)</span>
          </a>
        )}
      </div>
    </div>
  );
}
