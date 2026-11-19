import {
  AppLink,
  ButtonLink,
  CheckIcon,
  cx,
  Mascot,
  StarRating,
  SubjectIcon,
  subjectById,
} from "../../ui";
import type { ChapterMapView, ChapterNode } from "./types";

export type ChapterMapScreenProps = {
  readonly view: ChapterMapView;
  readonly selectedChapterId: string | null;
  readonly onSelectChapter: (chapterId: string) => void;
  readonly subjectHref: (subjectId: string) => string;
  readonly chapterHref: (chapterId: string) => string;
};

// The path zig-zags like a board game: each node steps right, then back.
const INDENT = ["sm:ml-0", "sm:ml-16", "sm:ml-32", "sm:ml-16"] as const;

function SubjectSwitcher({
  view,
  subjectHref,
}: Pick<ChapterMapScreenProps, "view" | "subjectHref">) {
  return (
    <nav aria-label="Mapel" className="-mx-4 overflow-x-auto px-4 sm:mx-0 sm:px-0">
      <ul className="m-0 flex w-max list-none gap-2 p-0 pb-1 sm:w-auto sm:flex-wrap">
        {view.subjectIds.map((id) => {
          const s = subjectById(id);
          if (s === undefined) return null;
          const on = id === view.subjectId;
          return (
            <li key={id}>
              <AppLink
                href={subjectHref(id)}
                aria-current={on ? "page" : undefined}
                className="inline-flex min-h-14 items-center gap-2.5 rounded-btn border-[3px] py-0 pr-[18px] pl-2 text-[16px] font-extrabold text-ink"
                style={{
                  background: on ? s.tint : "#ffffff",
                  borderColor: on ? s.color : "#d5e3f0",
                }}
              >
                <span className="flex size-10 items-center justify-center rounded-[12px] bg-white">
                  <SubjectIcon subjectId={id} size={30} />
                </span>
                {s.shortName}
              </AppLink>
            </li>
          );
        })}
      </ul>
    </nav>
  );
}

function Node({
  chapter,
  selected,
  color,
  ink,
  edge,
  onSelect,
  indent,
}: {
  readonly chapter: ChapterNode;
  readonly selected: boolean;
  readonly color: string;
  readonly ink: string;
  readonly edge: string;
  readonly onSelect: () => void;
  readonly indent: string;
}) {
  const done = chapter.status === "done";
  const size = selected ? "size-16" : "size-[52px]";
  return (
    <li className={indent}>
      <button
        type="button"
        aria-pressed={selected}
        onClick={onSelect}
        className={cx(
          "pressable inline-flex cursor-pointer items-center gap-3.5 rounded-full border-0 py-1.5 pr-[18px] pl-1.5 text-left text-ink",
          selected ? "bg-white" : "bg-transparent hover:bg-white/60",
        )}
        style={selected ? { boxShadow: `0 4px 0 ${edge}` } : undefined}
      >
        <span
          aria-hidden="true"
          className={cx(
            "flex shrink-0 items-center justify-center rounded-full border-4 font-display text-[20px] font-semibold",
            size,
          )}
          style={
            done
              ? { background: color, borderColor: color, color: "#ffffff" }
              : {
                  background: "#ffffff",
                  borderColor: selected || chapter.status === "current" ? color : "#d5e3f0",
                  color: selected ? ink : "#55697f",
                }
          }
        >
          {done ? <CheckIcon size={24} /> : chapter.number}
        </span>
        <span className="flex flex-col gap-0.5">
          <span
            className="text-[13px] font-extrabold"
            style={{ color: done || selected ? ink : "#55697f" }}
          >
            BAB {chapter.number}
            {done ? <span className="sr-only"> (selesai)</span> : null}
          </span>
          <span className="text-[17px] font-extrabold leading-tight">{chapter.title}</span>
          <StarRating value={chapter.stars} />
        </span>
      </button>
    </li>
  );
}

export function ChapterMapScreen({
  view,
  selectedChapterId,
  onSelectChapter,
  subjectHref,
  chapterHref,
}: ChapterMapScreenProps) {
  const subject = subjectById(view.subjectId);
  if (subject === undefined) return null;
  const selected =
    view.chapters.find((c) => c.id === selectedChapterId) ??
    view.chapters.find((c) => c.status === "current") ??
    view.chapters[0];
  return (
    <div className="flex flex-col gap-5">
      <SubjectSwitcher view={view} subjectHref={subjectHref} />
      <div className="flex flex-wrap items-start gap-6">
        <section
          aria-labelledby="peta-title"
          className="min-w-0 flex-1 basis-[480px] rounded-panel border-2 p-5 sm:p-6"
          style={{ background: subject.tint, borderColor: subject.edge }}
        >
          <div className="mb-5 flex items-center gap-3.5">
            <span
              aria-hidden="true"
              className="flex size-[72px] shrink-0 items-center justify-center rounded-[22px] bg-white"
              style={{ boxShadow: `0 4px 0 ${subject.edge}` }}
            >
              <SubjectIcon subjectId={subject.id} size={50} />
            </span>
            <h1
              id="peta-title"
              className="m-0 font-display text-[26px] font-semibold leading-tight sm:text-[28px]"
            >
              Peta petualangan {subject.name}
            </h1>
          </div>
          <ol className="m-0 flex list-none flex-col gap-1.5 p-0">
            {view.chapters.map((c, i) => (
              <Node
                key={c.id}
                chapter={c}
                selected={c.id === selected?.id}
                color={subject.color}
                ink={subject.ink}
                edge={subject.edge}
                indent={INDENT[i % INDENT.length] ?? ""}
                onSelect={() => onSelectChapter(c.id)}
              />
            ))}
          </ol>
        </section>
        {selected !== undefined && (
          <aside
            aria-labelledby="start-title"
            className="flex min-w-0 flex-1 basis-[320px] flex-col gap-4 rounded-panel border-2 border-line bg-white p-5 sm:p-6"
          >
            <div className="flex items-start gap-3">
              <Mascot mood="happy" size={64} />
              <div className="rounded-[6px_18px_18px_18px] bg-page px-4 py-3 text-[16px] font-bold leading-normal">
                Ayo kita pelajari bab ini bareng! Aku jelaskan pelan-pelan, terus kamu jawab
                pertanyaanku, ya.
              </div>
            </div>
            <h2
              id="start-title"
              className="m-0 font-display text-[24px] font-semibold leading-tight"
            >
              Bab {selected.number}: {selected.title}
            </h2>
            <ol className="m-0 flex list-none flex-col gap-2.5 p-0">
              {["Penjelasan dari buku", "Pertanyaan cek paham", "Kuis kecil, dapat bintang"].map(
                (label, i) => (
                  <li key={label} className="flex items-center gap-2.5 text-[16px] font-bold">
                    <span
                      aria-hidden="true"
                      className="flex size-[30px] items-center justify-center rounded-[10px] bg-primary-soft font-extrabold text-primary"
                    >
                      {i + 1}
                    </span>
                    {label}
                  </li>
                ),
              )}
            </ol>
            <ButtonLink href={chapterHref(selected.id)} size="lg" block>
              {selected.status === "done" ? "Buka lagi bab ini" : "Mulai Belajar"}
            </ButtonLink>
          </aside>
        )}
      </div>
    </div>
  );
}
