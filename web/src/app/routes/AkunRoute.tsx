import { useState } from "react";
import { useLocation } from "react-router";
import { api } from "../../contracts/client";
import { generalMessage, passwordChangeMessage, profileMessage } from "../../contracts/messages";
import { err, ok } from "../../contracts/result";
import { AkunScreen, linkNotice } from "../../features/akun";
import { useHeaderStats } from "../headerStats";
import { useAuth } from "../session";
import { useTheme } from "../theme";
import { useGoogleEnabled } from "../useGoogleEnabled";

export function AkunRoute() {
  const auth = useAuth();
  const stats = useHeaderStats();
  const theme = useTheme();
  const googleEnabled = useGoogleEnabled();
  const { search } = useLocation();
  // `?google=linked|conflict|cancelled|unavailable|failed` from the server's link callback.
  const [googleNotice] = useState(() => linkNotice(search));
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
        theme: theme.preference,
        hasPassword: student.hasPassword,
        isGoogleLinked: student.isGoogleLinked,
        isGoogleEnabled: googleEnabled,
      }}
      googleNotice={googleNotice}
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
      onThemeChange={async (next) => {
        const r = await theme.setPreference(next);
        return r.ok ? ok(undefined) : err(profileMessage(r.error));
      }}
      onLogout={async () => {
        const r = await auth.logout();
        return r.ok ? ok(undefined) : err({ form: generalMessage(r.error), fields: {} });
      }}
    />
  );
}
