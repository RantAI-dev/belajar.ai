import { useNavigate } from "react-router";
import { BERANDA_FIXTURE, BerandaScreen } from "../../features/beranda";
import { useDemo } from "../demo";
import { useAuth } from "../session";

export function BerandaRoute() {
  const navigate = useNavigate();
  const demo = useDemo();
  const { student } = useAuth();
  return (
    <BerandaScreen
      view={{ ...BERANDA_FIXTURE, studentName: student?.nickname ?? BERANDA_FIXTURE.studentName }}
      onAsk={(question) => navigate(`/tanya/${demo.ask(null, question)}`)}
    />
  );
}
