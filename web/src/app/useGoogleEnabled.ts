import { useEffect, useState } from "react";
import { api } from "../contracts/client";

/**
 * Whether the server has "Masuk dengan Google" configured. `false` until it answers, and on any
 * failure: the button stays hidden rather than leading to an error page.
 */
export function useGoogleEnabled(): boolean {
  const [enabled, setEnabled] = useState(false);
  useEffect(() => {
    const controller = new AbortController();
    void api.get("/v1/auth/google", { signal: controller.signal }).then((r) => {
      if (r.ok) setEnabled(r.value.isEnabled);
    });
    return () => controller.abort();
  }, []);
  return enabled;
}
