import { useEffect, useState } from "react";
import { api } from "../contracts/client";
import { generalMessage } from "../contracts/messages";
import { type ChapterDto, type SubjectEntry, subjectEntry } from "../features/belajar";
import { isSubjectId, SUBJECTS } from "../ui";

export type SubjectGrid =
  | { readonly status: "ready"; readonly entries: readonly SubjectEntry[] }
  | { readonly status: "failed"; readonly message: string };

/**
 * The subject cards with the student's progress: GET /v1/subjects, then each subject's chapters.
 * Cards show "…" until their chapters arrive; a subject whose chapters fail keeps that label.
 */
export function useSubjectGrid(): {
  readonly grid: SubjectGrid;
  /** Each subject's chapters, once they arrived. */
  readonly chapters: Readonly<Record<string, readonly ChapterDto[]>>;
  readonly retry: () => void;
} {
  const [ids, setIds] = useState<readonly string[]>(SUBJECTS.map((s) => s.id));
  const [chapters, setChapters] = useState<Readonly<Record<string, readonly ChapterDto[]>>>({});
  const [failed, setFailed] = useState<string | null>(null);
  const [attempt, setAttempt] = useState(0);

  useEffect(() => {
    void attempt;
    const ctrl = new AbortController();
    setFailed(null);
    void (async () => {
      const r = await api.get("/v1/subjects", { signal: ctrl.signal });
      if (ctrl.signal.aborted) return;
      if (!r.ok) {
        setFailed(generalMessage(r.error));
        return;
      }
      const known = r.value.data.map((s) => s.id).filter(isSubjectId);
      setIds(known);
      await Promise.all(
        known.map(async (subjectId) => {
          const c = await api.get("/v1/subjects/{subjectId}/chapters", {
            pathParams: { subjectId },
            signal: ctrl.signal,
          });
          if (c.ok && !ctrl.signal.aborted)
            setChapters((all) => ({ ...all, [subjectId]: c.value.data }));
        }),
      );
    })();
    return () => ctrl.abort();
  }, [attempt]);

  const retry = () => setAttempt((n) => n + 1);
  if (failed !== null) return { grid: { status: "failed", message: failed }, chapters, retry };
  return {
    grid: { status: "ready", entries: ids.map((id) => subjectEntry(id, chapters[id] ?? null)) },
    chapters,
    retry,
  };
}
