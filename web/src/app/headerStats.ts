import { useDemo } from "./demo";

/**
 * Stars and streak for the header and Akun. On fixtures until GET /v1/home exists; the slice that
 * wires Beranda changes only this hook.
 */
export const useHeaderStats = (): { readonly stars: number; readonly streakDays: number } => {
  const demo = useDemo();
  return { stars: demo.stars, streakDays: demo.streakDays };
};
