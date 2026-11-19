// Who may see which route. Pure, so the redirects are tested without a router.

/** The only routes a signed-out visitor may see. */
export const PUBLIC_PATHS: readonly string[] = ["/masuk", "/daftar"];

/** `/masuk?ke=/latihan` brings the student back to /latihan after signing in. */
export const RETURN_PARAM = "ke";

const isPublic = (pathname: string): boolean => PUBLIC_PATHS.includes(pathname.replace(/\/+$/, ""));

/** A same-origin, non-auth path, or "/". `//evil.example` and `https://…` never pass. */
export const safeReturnPath = (value: string | null): string => {
  if (value === null || !value.startsWith("/") || value.startsWith("//") || value.includes("\\")) {
    return "/";
  }
  const pathname = value.split(/[?#]/)[0] ?? "/";
  return isPublic(pathname) ? "/" : value;
};

export type GuardLocation = { readonly pathname: string; readonly search: string };

/**
 * Where to send this visitor instead, or null to show the route. `remember: false` (after "Keluar")
 * sends them to /masuk without `?ke=`, so signing in again starts at home.
 */
export function guardRedirect(
  location: GuardLocation,
  signedIn: boolean,
  { remember = true }: { readonly remember?: boolean } = {},
): string | null {
  const publicRoute = isPublic(location.pathname);
  if (signedIn) {
    return publicRoute
      ? safeReturnPath(new URLSearchParams(location.search).get(RETURN_PARAM))
      : null;
  }
  if (publicRoute) return null;
  const target = `${location.pathname}${location.search}`;
  return target === "/" || !remember
    ? "/masuk"
    : `/masuk?${RETURN_PARAM}=${encodeURIComponent(target)}`;
}
