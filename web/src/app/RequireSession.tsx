import type { ReactNode } from "react";
import { Navigate, Outlet, useLocation } from "react-router";
import { generalMessage } from "../contracts/messages";
import { Button, Mascot } from "../ui";
import { guardRedirect } from "./guard";
import { useAuth } from "./session";

function FullScreen({ children }: { readonly children: ReactNode }) {
  return (
    <main className="mx-auto flex min-h-screen max-w-[520px] flex-col items-center justify-center gap-4 px-4 text-center">
      {children}
    </main>
  );
}

/** Waits for the session, then shows the route or redirects (see guard.ts). */
export function RequireSession() {
  const location = useLocation();
  const auth = useAuth();
  const { state } = auth;
  if (state.status === "loading") {
    return (
      <FullScreen>
        <Mascot mood="thinking" size={120} animate />
        <p role="status" className="m-0 font-display text-[22px] font-semibold">
          Sebentar, aku siapkan dulu…
        </p>
      </FullScreen>
    );
  }
  if (state.status === "failed") {
    return (
      <FullScreen>
        <Mascot mood="hmm" size={120} />
        <p role="alert" className="m-0 text-[18px] font-bold">
          {generalMessage(state.problem)}
        </p>
        <Button size="lg" onClick={() => void auth.refresh()}>
          Coba lagi
        </Button>
      </FullScreen>
    );
  }
  const to = guardRedirect(location, state.status === "signedIn", {
    remember: !(state.status === "signedOut" && state.loggedOut),
  });
  return to === null ? <Outlet /> : <Navigate to={to} replace />;
}
