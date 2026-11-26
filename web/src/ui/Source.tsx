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
  /** Opens the book in the drawer. Without it there is nothing more to see. */
  readonly onOpen?: () => void;
};

export function PagePreview({
  title,
  note = "Bagian yang kupakai ditandai kuning.",
  imageUrl,
  onOpen,
}: PagePreviewProps) {
  const page =
    imageUrl !== undefined ? (
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
    );
  return (
    <div className="flex flex-wrap items-start gap-3.5 rounded-[16px] border-2 border-dashed border-line-sky bg-page p-3">
      {onOpen !== undefined ? (
        // The picture opens the book too; the button below is the one a screen reader announces.
        <button
          type="button"
          tabIndex={-1}
          aria-hidden="true"
          onClick={onOpen}
          className="pressable cursor-zoom-in rounded-[10px] border-0 bg-transparent p-0 hover:-translate-y-0.5"
        >
          {page}
        </button>
      ) : (
        page
      )}
      <div className="flex min-w-[180px] flex-1 flex-col gap-2">
        <span className="text-[16px] font-extrabold">{title}</span>
        <span className="text-[15px] font-semibold text-ink-muted">{note}</span>
        {onOpen !== undefined && (
          <button
            type="button"
            onClick={onOpen}
            aria-haspopup="dialog"
            className="pressable inline-flex min-h-12 cursor-pointer items-center self-start rounded-[12px] border-2 border-line-sky bg-surface px-4 text-[15px] font-extrabold text-ink hover:bg-primary-soft"
          >
            Lihat halaman penuh
          </button>
        )}
      </div>
    </div>
  );
}
