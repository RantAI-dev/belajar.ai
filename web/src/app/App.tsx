import { BrowserRouter, Route, Routes } from "react-router";
import { LinkProvider } from "../ui";
import { DemoProvider } from "./demo";
import { RouterLink } from "./RouterLink";
import { AkunRoute } from "./routes/AkunRoute";
import { LoginRoute, RegisterRoute } from "./routes/AuthRoutes";
import { BelajarHomeRoute, ChapterMapRoute, ChapterSessionRoute } from "./routes/BelajarRoutes";
import { BerandaRoute } from "./routes/BerandaRoute";
import { LatihanSetupRoute, QuizRoute, ResultRoute } from "./routes/LatihanRoutes";
import { NotFound } from "./routes/NotFound";
import { TanyaRoute } from "./routes/TanyaRoute";
import { Shell } from "./Shell";

export function App() {
  return (
    <BrowserRouter>
      <LinkProvider component={RouterLink}>
        <DemoProvider>
          <Routes>
            <Route path="/masuk" element={<LoginRoute />} />
            <Route path="/daftar" element={<RegisterRoute />} />
            <Route element={<Shell />}>
              <Route index element={<BerandaRoute />} />
              <Route path="tanya" element={<TanyaRoute />} />
              <Route path="tanya/:conversationId" element={<TanyaRoute />} />
              <Route path="belajar" element={<BelajarHomeRoute />} />
              <Route path="belajar/:subjectId" element={<ChapterMapRoute />} />
              <Route path="belajar/:subjectId/:chapterId" element={<ChapterSessionRoute />} />
              <Route path="latihan" element={<LatihanSetupRoute />} />
              <Route path="latihan/:setId" element={<QuizRoute />} />
              <Route path="latihan/:setId/hasil" element={<ResultRoute />} />
              <Route path="akun" element={<AkunRoute />} />
              <Route path="*" element={<NotFound />} />
            </Route>
          </Routes>
        </DemoProvider>
      </LinkProvider>
    </BrowserRouter>
  );
}
