export type AnswerPart = { readonly text: string; readonly emphasis?: boolean };

export type BookSource = {
  readonly id: string;
  /** The `n` of the `[n]` markers in the answer text. */
  readonly number: number;
  /** The book page, when the source has one. */
  readonly page?: number;
  /** Short chip label, e.g. "Buku IPA 7 · hal. 112". */
  readonly label: string;
  /** Long title for the preview, e.g. "IPA Kelas VII Semester 1 · Pemuaian · halaman 112". */
  readonly title: string;
  /** The book's own title, e.g. "IPA Buku Siswa Kelas VII". */
  readonly book: string;
  /** The chapter or section, e.g. "Bab 5: Suhu, Kalor, dan Pemuaian". */
  readonly section?: string;
  readonly imageUrl?: string;
  readonly fullPageHref?: string;
};

export type TanyaMessage =
  | { readonly id: string; readonly kind: "question"; readonly text: string }
  | {
      readonly id: string;
      readonly kind: "answer";
      readonly status: "streaming" | "done";
      readonly parts: readonly AnswerPart[];
      /** "Contoh lain" shown in a warm box under the answer. */
      readonly example?: string;
      /** The chips under the answer: the cited sources, or all of them when none is cited. */
      readonly sources: readonly BookSource[];
      /** Every source of the answer, cited or not: an inline `[n]` is looked up here. */
      readonly allSources?: readonly BookSource[];
    }
  | {
      readonly id: string;
      readonly kind: "not-in-book";
      /** e.g. "Kelas 7", to say which books were searched. */
      readonly gradeLabel: string;
    }
  | {
      readonly id: string;
      /** The answer could not be fetched; `text` is the friendly sentence. */
      readonly kind: "error";
      readonly text: string;
    };

export type FollowUp = { readonly id: string; readonly label: string };

export type SubjectFilter = "semua" | string;

/** One row of the "Percakapanmu" list. */
export type ConversationSummary = {
  readonly id: string;
  readonly title: string;
  readonly subjectId: string | null;
  /** e.g. "19 Nov". */
  readonly dateLabel: string;
  readonly href: string;
};

export type TanyaView = {
  /** Null hides the chips: an existing conversation keeps the subject it started with. */
  readonly filter: SubjectFilter | null;
  /** Subject ids offered as filter chips, after "Semua". */
  readonly filterSubjects: readonly string[];
  readonly messages: readonly TanyaMessage[];
  /** Offered under the last finished answer. */
  readonly followUps: readonly FollowUp[];
  /** Starter questions for an empty conversation. */
  readonly suggestions: readonly string[];
  /** True while an answer is arriving: sending is paused. */
  readonly busy: boolean;
};
