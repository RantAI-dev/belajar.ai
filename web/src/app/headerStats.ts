import { useHome } from "./home";

/** Stars and streak for the header and Akun, from GET /v1/home (0 until it answers). */
export const useHeaderStats = (): { readonly stars: number; readonly streakDays: number } => {
  const { home } = useHome();
  return { stars: home?.stars ?? 0, streakDays: home?.streakDays ?? 0 };
};
