import type { ThemePreference } from "../../ui";
import type { Grade } from "../auth";

export type AkunView = {
  readonly nickname: string;
  readonly username: string;
  readonly grade: Grade;
  readonly stars: number;
  readonly streakDays: number;
  readonly theme: ThemePreference;
  /** False for a student who signed up with Google: there is no password form then. */
  readonly hasPassword: boolean;
  readonly isGoogleLinked: boolean;
  /** Whether the server has Google configured: "Hubungkan dengan Google" needs it. */
  readonly isGoogleEnabled: boolean;
};
