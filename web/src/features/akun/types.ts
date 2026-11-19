import type { Grade } from "../auth";

export type AkunView = {
  readonly nickname: string;
  readonly username: string;
  readonly grade: Grade;
  readonly stars: number;
  readonly streakDays: number;
};
