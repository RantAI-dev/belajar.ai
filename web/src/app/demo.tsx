import {
  createContext,
  type ReactNode,
  useCallback,
  useContext,
  useMemo,
  useRef,
  useState,
} from "react";
import { AKUN_FIXTURE } from "../features/akun";
import type { Grade } from "../features/auth";
import { BERANDA_FIXTURE } from "../features/beranda";
import { type Answers, LATIHAN_SET, type QuizSet } from "../features/latihan";
import {
  TANYA_CONVERSATION,
  TANYA_DEMO_REPLY,
  TANYA_OFF_TOPIC,
  type TanyaMessage,
} from "../features/tanya";
import { takeWords, wordCount } from "./streamText";

/**
 * Local stand-in for the server while the screens are presentational: the student, their stars,
 * conversations and practice sets, all in memory. A later slice replaces this with API calls.
 */
type Demo = {
  readonly nickname: string;
  readonly username: string;
  readonly grade: Grade;
  readonly stars: number;
  readonly streakDays: number;
  readonly setProfile: (p: { nickname?: string; username?: string; grade?: Grade }) => void;
  readonly addStars: (n: number) => void;
  readonly conversations: Readonly<Record<string, readonly TanyaMessage[]>>;
  /** Adds the question and streams a canned answer. Returns the conversation id. */
  readonly ask: (conversationId: string | null, question: string) => string;
  readonly sets: Readonly<Record<string, QuizSet>>;
  readonly addSet: (set: QuizSet) => void;
  readonly answers: Readonly<Record<string, Answers>>;
  readonly setAnswers: (setId: string, answers: Answers) => void;
};

const DemoContext = createContext<Demo | null>(null);

const WORD_MS = 70;

export function DemoProvider({ children }: { readonly children: ReactNode }) {
  const [nickname, setNickname] = useState(BERANDA_FIXTURE.studentName);
  const [username, setUsername] = useState(AKUN_FIXTURE.username);
  const [grade, setGrade] = useState<Grade>(AKUN_FIXTURE.grade);
  const [stars, setStars] = useState(AKUN_FIXTURE.stars);
  const [conversations, setConversations] = useState<Record<string, readonly TanyaMessage[]>>({
    "rel-kereta": TANYA_CONVERSATION,
  });
  const [sets, setSets] = useState<Record<string, QuizSet>>({ [LATIHAN_SET.id]: LATIHAN_SET });
  const [answers, setAnswersState] = useState<Record<string, Answers>>({});
  const seq = useRef(0);

  const setProfile = useCallback((p: { nickname?: string; username?: string; grade?: Grade }) => {
    if (p.nickname !== undefined) setNickname(p.nickname);
    if (p.username !== undefined) setUsername(p.username);
    if (p.grade !== undefined) setGrade(p.grade);
  }, []);

  const addStars = useCallback((n: number) => setStars((s) => s + n), []);

  const ask = useCallback(
    (conversationId: string | null, question: string): string => {
      seq.current += 1;
      const n = seq.current;
      const id = conversationId ?? `percakapan-${n}`;
      const answerId = `a-${n}`;
      const offTopic = TANYA_OFF_TOPIC.test(question);
      const update = (fn: (msgs: readonly TanyaMessage[]) => readonly TanyaMessage[]) =>
        setConversations((all) => ({ ...all, [id]: fn(all[id] ?? []) }));
      const replace = (msg: TanyaMessage) =>
        update((msgs) => msgs.map((m) => (m.id === answerId ? msg : m)));

      update((msgs) => [
        ...msgs,
        { id: `q-${n}`, kind: "question", text: question },
        { id: answerId, kind: "answer", status: "streaming", parts: [], sources: [] },
      ]);

      if (offTopic) {
        window.setTimeout(
          () => replace({ id: answerId, kind: "not-in-book", gradeLabel: `Kelas ${grade}` }),
          900,
        );
        return id;
      }
      const total = wordCount(TANYA_DEMO_REPLY.text);
      let words = 0;
      const timer = window.setInterval(() => {
        words += 1;
        const done = words >= total;
        replace({
          id: answerId,
          kind: "answer",
          status: done ? "done" : "streaming",
          parts: [{ text: takeWords(TANYA_DEMO_REPLY.text, words) }],
          ...(done ? { example: TANYA_DEMO_REPLY.example } : {}),
          sources: done ? TANYA_DEMO_REPLY.sources : [],
        });
        if (done) window.clearInterval(timer);
      }, WORD_MS);
      return id;
    },
    [grade],
  );

  const addSet = useCallback((set: QuizSet) => setSets((all) => ({ ...all, [set.id]: set })), []);
  const setAnswers = useCallback(
    (setId: string, a: Answers) => setAnswersState((all) => ({ ...all, [setId]: a })),
    [],
  );

  const value = useMemo<Demo>(
    () => ({
      nickname,
      username,
      grade,
      stars,
      streakDays: AKUN_FIXTURE.streakDays,
      setProfile,
      addStars,
      conversations,
      ask,
      sets,
      addSet,
      answers,
      setAnswers,
    }),
    [
      nickname,
      username,
      grade,
      stars,
      setProfile,
      addStars,
      conversations,
      ask,
      sets,
      addSet,
      answers,
      setAnswers,
    ],
  );
  return <DemoContext.Provider value={value}>{children}</DemoContext.Provider>;
}

const FALLBACK: Demo = {
  nickname: BERANDA_FIXTURE.studentName,
  username: AKUN_FIXTURE.username,
  grade: AKUN_FIXTURE.grade,
  stars: AKUN_FIXTURE.stars,
  streakDays: AKUN_FIXTURE.streakDays,
  setProfile: () => {},
  addStars: () => {},
  conversations: {},
  ask: () => "",
  sets: {},
  addSet: () => {},
  answers: {},
  setAnswers: () => {},
};

export const useDemo = (): Demo => useContext(DemoContext) ?? FALLBACK;
