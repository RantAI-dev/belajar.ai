import { type FormEvent, useState } from "react";
import { AppLink, ArrowLeftIcon, Button, TextField } from "../../ui";

export type ConversationTitleProps = {
  readonly title: string;
  readonly backHref: string;
  /** Resolves to a friendly error sentence, or null when the new name is saved. */
  readonly onRename: (title: string) => Promise<string | null>;
};

const TITLE_MAX = 100;

/** The conversation's name with a "Ganti nama" form. */
export function ConversationTitle({ title, backHref, onRename }: ConversationTitleProps) {
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(title);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const submit = async (e: FormEvent) => {
    e.preventDefault();
    const next = draft.trim();
    if (next.length === 0) {
      setError("Tulis nama percakapannya dulu, ya.");
      return;
    }
    setSaving(true);
    const problem = await onRename(next);
    setSaving(false);
    setError(problem);
    if (problem === null) setEditing(false);
  };
  return (
    <div className="flex flex-col gap-2">
      <AppLink
        href={backHref}
        className="inline-flex min-h-12 items-center gap-2 self-start rounded-chip pr-3 text-[16px] font-extrabold"
      >
        <ArrowLeftIcon size={20} />
        Semua percakapan
      </AppLink>
      {editing ? (
        <form onSubmit={submit} className="flex flex-wrap items-end gap-2.5">
          <TextField
            label="Nama percakapan"
            value={draft}
            maxLength={TITLE_MAX}
            onValueChange={setDraft}
            className="min-w-0 flex-1 basis-[240px]"
            autoFocus
            {...(error !== null ? { error } : {})}
          />
          <Button type="submit" loading={saving} loadingLabel="Menyimpan…">
            Simpan
          </Button>
          <Button
            variant="ghost"
            onClick={() => {
              setEditing(false);
              setDraft(title);
              setError(null);
            }}
          >
            Batal
          </Button>
        </form>
      ) : (
        <div className="flex flex-wrap items-center gap-2">
          <h1 className="m-0 min-w-0 flex-1 font-display text-[24px] font-semibold leading-tight sm:text-[28px]">
            {title}
          </h1>
          <Button
            variant="ghost"
            onClick={() => {
              setDraft(title);
              setEditing(true);
            }}
          >
            Ganti nama
          </Button>
        </div>
      )}
    </div>
  );
}
