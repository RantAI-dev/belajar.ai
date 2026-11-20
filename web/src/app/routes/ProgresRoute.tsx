import { useEffect, useState } from "react";
import { api } from "../../contracts/client";
import { generalMessage } from "../../contracts/messages";
import { ProgresScreen, type ProgressDto, progresView } from "../../features/progres";
import { ErrorState, LoadingState } from "../../ui";
import { useSubjectGrid } from "../subjects";

type Load =
  | { readonly status: "loading" }
  | { readonly status: "ready"; readonly dto: ProgressDto }
  | { readonly status: "failed"; readonly message: string };

export function ProgresRoute() {
  const [load, setLoad] = useState<Load>({ status: "loading" });
  const [attempt, setAttempt] = useState(0);
  // The lesson's own title can be a chat title; the chapters list has the real one.
  const { chapters } = useSubjectGrid();

  useEffect(() => {
    void attempt;
    const ctrl = new AbortController();
    setLoad({ status: "loading" });
    void api.get("/v1/progress", { signal: ctrl.signal }).then((r) => {
      if (ctrl.signal.aborted) return;
      setLoad(
        r.ok
          ? { status: "ready", dto: r.value }
          : { status: "failed", message: generalMessage(r.error) },
      );
    });
    return () => ctrl.abort();
  }, [attempt]);

  if (load.status === "loading") return <LoadingState text="Sebentar, aku hitung progresmu…" />;
  if (load.status === "failed")
    return <ErrorState message={load.message} onRetry={() => setAttempt((n) => n + 1)} />;
  return (
    <ProgresScreen
      view={progresView(
        load.dto,
        (subjectId, chapterId) => chapters[subjectId]?.find((c) => c.id === chapterId)?.title,
      )}
    />
  );
}
