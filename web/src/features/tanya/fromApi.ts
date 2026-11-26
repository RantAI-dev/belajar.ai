import type { components } from "../../contracts/api.gen";
import type { BookSource, ConversationSummary, TanyaMessage } from "./types";

export type MessageDto = components["schemas"]["MessageDto"];
export type SourceDto = components["schemas"]["SourceDto"];
export type ConversationDto = components["schemas"]["ConversationDto"];

/** "IPA Buku Siswa Kelas VII · hal. 78", numbered like the `[1]` in the answer. */
export const sourceFromDto = (messageId: string, s: SourceDto): BookSource => {
  const page = s.page ?? null;
  const title = [s.title, s.section ?? null, page === null ? null : `halaman ${page}`]
    .filter((p) => p !== null && p !== "")
    .join(" · ");
  const url = s.pagePreviewUrl ?? null;
  const filePage = s.previewPage ?? null;
  const section = s.section ?? "";
  return {
    id: `${messageId}-${s.number}`,
    number: s.number,
    ...(page === null ? {} : { page }),
    label: `[${s.number}] ${s.title}${page === null ? "" : ` · hal. ${page}`}`,
    title,
    book: s.title,
    ...(section === "" ? {} : { section }),
    ...(url === null ? {} : { fileUrl: url }),
    // A whole-book PDF cannot be an <img>: the preview draws a page sketch instead.
    ...(url !== null && filePage === null ? { imageUrl: url } : {}),
    ...(url !== null && filePage !== null ? { filePage } : {}),
  };
};

/** A stored message as the chat shows it. An answer without book sources is "not in your book". */
export const messageFromDto = (m: MessageDto, gradeLabel: string): TanyaMessage => {
  if (m.role === "user") return { id: m.id, kind: "question", text: m.content };
  const cited = m.sources.filter((s) => s.isCited);
  if (m.isGrounded === false || m.sources.length === 0)
    return { id: m.id, kind: "not-in-book", gradeLabel };
  return {
    id: m.id,
    kind: "answer",
    status: "done",
    parts: [{ text: m.content }],
    // The answer cited nothing by number: show every passage it used rather than none.
    sources: (cited.length > 0 ? cited : m.sources).map((s) => sourceFromDto(m.id, s)),
    allSources: m.sources.map((s) => sourceFromDto(m.id, s)),
  };
};

const MONTHS = ["Jan", "Feb", "Mar", "Apr", "Mei", "Jun", "Jul", "Agu", "Sep", "Okt", "Nov", "Des"];

/** "19 Nov" in the student's own time zone. */
export const dateLabel = (iso: string): string => {
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return "";
  return `${d.getDate()} ${MONTHS[d.getMonth()] ?? ""}`;
};

export const summaryFromDto = (c: ConversationDto): ConversationSummary => ({
  id: c.id,
  title: c.title,
  subjectId: c.subjectId ?? null,
  dateLabel: dateLabel(c.updatedAt),
  href: `/tanya/${c.id}`,
});

const isRecord = (v: unknown): v is Record<string, unknown> =>
  typeof v === "object" && v !== null && !Array.isArray(v);

const isSource = (v: unknown): v is SourceDto =>
  isRecord(v) &&
  typeof v.number === "number" &&
  typeof v.title === "string" &&
  typeof v.isCited === "boolean";

/** The `message` and `done` events carry a stored message; anything else is ignored. */
export const parseMessage = (data: unknown): MessageDto | null => {
  if (!isRecord(data)) return null;
  const { id, role, content, sources } = data;
  if (typeof id !== "string" || typeof content !== "string") return null;
  if (role !== "user" && role !== "assistant") return null;
  if (!Array.isArray(sources) || !sources.every(isSource)) return null;
  return data as MessageDto;
};

/** The `delta` event's text, or null when it is not `{ text }`. */
export const parseDelta = (data: unknown): string | null =>
  isRecord(data) && typeof data.text === "string" ? data.text : null;
