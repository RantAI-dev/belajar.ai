import type { BerandaView } from "./types";

export const BERANDA_FIXTURE: BerandaView = {
  studentName: "Siswa A",
  missions: [
    { id: "m1", text: "Tanya 1 hal tentang pelajaranmu", done: true, stars: 5, href: "/tanya" },
    {
      id: "m2",
      text: "Selesaikan 1 langkah belajar",
      done: false,
      stars: 10,
      href: "/belajar/ipa/ipa-3",
    },
    { id: "m3", text: "Kerjakan 5 soal latihan", done: false, stars: 15, href: "/latihan" },
  ],
  continueLesson: {
    subjectId: "ipa",
    chapterTitle: "Suhu, kalor, dan pemuaian",
    step: 2,
    totalSteps: 5,
    href: "/belajar/ipa/ipa-3",
  },
  subjects: [
    { subjectId: "ipa", progressLabel: "Bab 3 dari 5", progress: 60, href: "/belajar/ipa" },
    { subjectId: "mtk", progressLabel: "Bab 2 dari 5", progress: 35, href: "/belajar/mtk" },
    { subjectId: "bing", progressLabel: "Bab 1 dari 4", progress: 20, href: "/belajar/bing" },
    { subjectId: "bind", progressLabel: "Baru mulai", progress: 10, href: "/belajar/bind" },
    { subjectId: "ips", progressLabel: "Belum mulai", progress: 0, href: "/belajar/ips" },
    { subjectId: "inf", progressLabel: "Belum mulai", progress: 0, href: "/belajar/inf" },
    { subjectId: "pp", progressLabel: "Belum mulai", progress: 0, href: "/belajar/pp" },
    { subjectId: "pai", progressLabel: "Bab 1 dari 5", progress: 15, href: "/belajar/pai" },
  ],
};
