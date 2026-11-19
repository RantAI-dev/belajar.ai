import { useNavigate } from "react-router";
import { BERANDA_FIXTURE, BerandaScreen } from "../../features/beranda";
import { useDemo } from "../demo";

export function BerandaRoute() {
  const navigate = useNavigate();
  const demo = useDemo();
  return (
    <BerandaScreen
      view={{ ...BERANDA_FIXTURE, studentName: demo.nickname }}
      onAsk={(question) => navigate(`/tanya/${demo.ask(null, question)}`)}
    />
  );
}
