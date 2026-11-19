export type Grade = 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12;
export type GradeLevel = "SD" | "SMP" | "SMA";

export type GradeGroup = {
  readonly level: GradeLevel;
  readonly label: string;
  readonly grades: readonly Grade[];
};

export const GRADE_GROUPS: readonly GradeGroup[] = [
  { level: "SD", label: "SD · Kelas 1–6", grades: [1, 2, 3, 4, 5, 6] },
  { level: "SMP", label: "SMP · Kelas 7–9", grades: [7, 8, 9] },
  { level: "SMA", label: "SMA · Kelas 10–12", grades: [10, 11, 12] },
];

export const isGrade = (n: number): n is Grade => Number.isInteger(n) && n >= 1 && n <= 12;

export const levelOf = (grade: Grade): GradeLevel =>
  grade <= 6 ? "SD" : grade <= 9 ? "SMP" : "SMA";

/** "Kelas 7 SMP" */
export const gradeLabel = (grade: Grade): string => `Kelas ${grade} ${levelOf(grade)}`;
