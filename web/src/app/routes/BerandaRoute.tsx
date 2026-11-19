import { useEffect, useState } from "react";
import { generalMessage } from "../../contracts/messages";
import type { ChapterDto } from "../../features/belajar";
import { BerandaScreen, berandaView, type HomeDto } from "../../features/beranda";
import { ErrorState, LoadingState } from "../../ui";
import { useHome } from "../home";
import { useSubjectGrid } from "../subjects";
import { useStartConversation } from "./TanyaRoute";

// The server's continueLearning title follows the lesson conversation's name, which changes after
// the first message; the chapters list keeps the real chapter title.
const withChapterTitles = (
  home: HomeDto,
  chapters: Readonly<Record<string, readonly ChapterDto[]>>,
): HomeDto => ({
  ...home,
  continueLearning: home.continueLearning.map((c) => ({
    ...c,
    chapterTitle: chapters[c.subjectId]?.find((x) => x.id === c.chapterId)?.title ?? c.chapterTitle,
  })),
});

export function BerandaRoute() {
  const { home, problem, refresh } = useHome();
  const { grid, chapters } = useSubjectGrid();
  const start = useStartConversation();
  const [asking, setAsking] = useState(false);
  const [askError, setAskError] = useState<string | null>(null);

  // Missions and stars change elsewhere: show today's numbers every time Beranda opens.
  useEffect(() => {
    void refresh();
  }, [refresh]);

  if (home === null)
    return problem === null ? (
      <LoadingState />
    ) : (
      <ErrorState message={generalMessage(problem)} onRetry={() => void refresh()} />
    );
  return (
    <BerandaScreen
      view={berandaView(
        withChapterTitles(home, chapters),
        grid.status === "ready" ? grid.entries : [],
      )}
      asking={asking}
      {...(askError === null ? {} : { askError })}
      onAsk={async (question) => {
        setAsking(true);
        const failure = await start(question, null);
        setAsking(false);
        setAskError(failure);
      }}
    />
  );
}
