import { useState } from "react";
import { useNavigate } from "react-router";
import { AkunScreen } from "../../features/akun";
import { useDemo } from "../demo";

export function AkunRoute() {
  const navigate = useNavigate();
  const demo = useDemo();
  const [notice, setNotice] = useState<string | undefined>(undefined);
  return (
    <AkunScreen
      view={{
        nickname: demo.nickname,
        username: demo.username,
        grade: demo.grade,
        stars: demo.stars,
        streakDays: demo.streakDays,
      }}
      {...(notice !== undefined ? { notice } : {})}
      onSaveProfile={(p) => {
        demo.setProfile(p);
        setNotice("Tersimpan! Profilmu sudah diperbarui.");
      }}
      onChangePassword={() => setNotice("Kata sandi sudah diganti. Ingat baik-baik, ya!")}
      onLogout={() => navigate("/masuk")}
    />
  );
}
