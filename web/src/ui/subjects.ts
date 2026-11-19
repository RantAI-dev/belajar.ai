/**
 * The eight subjects. Ids and colours are shared with the server.
 * `color` is for fills, icons and large white text only; `ink` is for any text drawn in the
 * subject's hue (>= 4.5:1 on white, on the page and on `tint`). `tint`, `edge` and `shadow` are the
 * light shades the cards are built from.
 */
export type SubjectId = "ipa" | "mtk" | "bing" | "bind" | "ips" | "inf" | "pp" | "pai";

export type Subject = {
  // A plain string, so lists of ids compare with ids that arrive from the server.
  readonly id: string;
  readonly name: string;
  readonly shortName: string;
  readonly color: string;
  readonly ink: string;
  readonly tint: string;
  readonly edge: string;
  readonly shadow: string;
};

export const SUBJECTS: readonly Subject[] = [
  {
    id: "ipa",
    name: "IPA",
    shortName: "IPA",
    color: "#388ca1",
    ink: "#1f5f6e",
    tint: "#e6f5f7",
    edge: "#bfe3e9",
    shadow: "#9fd3dd",
  },
  {
    id: "mtk",
    name: "Matematika",
    shortName: "Matematika",
    color: "#517fbb",
    ink: "#2f5487",
    tint: "#eaf1fb",
    edge: "#c8d9f1",
    shadow: "#a9c4ea",
  },
  {
    id: "bing",
    name: "Bahasa Inggris",
    shortName: "B. Inggris",
    color: "#574399",
    ink: "#3d2c75",
    tint: "#f0edf9",
    edge: "#d6cdf0",
    shadow: "#c2b5ea",
  },
  {
    id: "bind",
    name: "Bahasa Indonesia",
    shortName: "B. Indonesia",
    color: "#bb5153",
    ink: "#8a3335",
    tint: "#fbeeee",
    edge: "#f0caca",
    shadow: "#e8b2b2",
  },
  {
    id: "ips",
    name: "IPS",
    shortName: "IPS",
    color: "#bb7851",
    ink: "#7f4b2c",
    tint: "#fbf1ea",
    edge: "#efd2be",
    shadow: "#e6bd9f",
  },
  {
    id: "inf",
    name: "Informatika",
    shortName: "Informatika",
    color: "#32836a",
    ink: "#1f5c49",
    tint: "#e8f5f0",
    edge: "#bfe2d4",
    shadow: "#9fd2bf",
  },
  {
    id: "pp",
    name: "Pendidikan Pancasila",
    shortName: "Pancasila",
    color: "#0069a8",
    ink: "#00598a",
    tint: "#e6f2fb",
    edge: "#c4e0f6",
    shadow: "#a2cbee",
  },
  {
    id: "pai",
    name: "Pendidikan Agama Islam",
    shortName: "Agama Islam",
    color: "#2f7d5b",
    ink: "#1e5a40",
    tint: "#e7f4ed",
    edge: "#bde0cd",
    shadow: "#9fd0b6",
  },
];

export const subjectById = (id: string): Subject | undefined => SUBJECTS.find((s) => s.id === id);

const KNOWN: ReadonlySet<string> = new Set<SubjectId>([
  "ipa",
  "mtk",
  "bing",
  "bind",
  "ips",
  "inf",
  "pp",
  "pai",
]);

export const isSubjectId = (id: string): id is SubjectId => KNOWN.has(id);
