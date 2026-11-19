import { api } from "../../contracts/client";
import { generalMessage, passwordChangeMessage, profileMessage } from "../../contracts/messages";
import { err, ok } from "../../contracts/result";
import { AkunScreen } from "../../features/akun";
import { useHeaderStats } from "../headerStats";
import { useAuth } from "../session";

export function AkunRoute() {
  const auth = useAuth();
  const stats = useHeaderStats();
  const { student } = auth;
  // RequireSession renders this route only for a signed-in student.
  if (student === null) return null;
  return (
    <AkunScreen
      view={{
        nickname: student.nickname,
        username: student.username,
        grade: student.grade,
        stars: stats.stars,
        streakDays: stats.streakDays,
      }}
      onSaveProfile={async (p) => {
        const r = await auth.updateProfile(p);
        return r.ok ? ok(undefined) : err(profileMessage(r.error));
      }}
      onChangePassword={async ({ current, next }) => {
        const r = await api.post("/v1/auth/password", {
          currentPassword: current,
          newPassword: next,
        });
        return r.ok ? ok(undefined) : err(passwordChangeMessage(r.error));
      }}
      onLogout={async () => {
        const r = await auth.logout();
        return r.ok ? ok(undefined) : err({ form: generalMessage(r.error), fields: {} });
      }}
    />
  );
}
