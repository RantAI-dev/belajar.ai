// What every API call (JSON or stream) gives back when it fails. Built from the standard error body
// (api-error.ts) plus the transport facts a screen needs: the status and Retry-After.
import { parseApiError } from "./api-error";
import type { ErrorCode } from "./error-codes";

/** `network`: the request never got an answer (offline, DNS, the connection dropped). */
export type ProblemCode = ErrorCode | "network";

export type ApiProblem = {
  readonly status: number;
  readonly code: ProblemCode;
  /** JSON Pointer of the field (`/username`) → its code (`too_short`). */
  readonly fieldErrors: Readonly<Record<string, string>>;
  readonly retryAfterSeconds: number | null;
  readonly requestId: string;
};

export const networkProblem = (): ApiProblem => ({
  status: 0,
  code: "network",
  fieldErrors: {},
  retryAfterSeconds: null,
  requestId: "",
});

/** Retry-After in seconds. The HTTP-date form is not used by our server, so it reads as unknown. */
export const parseRetryAfter = (value: string | null): number | null => {
  if (value === null || !/^\s*\d+\s*$/.test(value)) return null;
  return Number.parseInt(value, 10);
};

/** A problem from a status, a parsed body (anything) and the response headers. */
export function problemFrom(status: number, body: unknown, headers: Headers | null): ApiProblem {
  const e = parseApiError(status, body, headers?.get("x-request-id") ?? null);
  const fieldErrors: Record<string, string> = {};
  for (const f of e.errors) {
    if (typeof f?.field === "string" && typeof f.code === "string") fieldErrors[f.field] = f.code;
  }
  return {
    status,
    code: e.code,
    fieldErrors,
    retryAfterSeconds: parseRetryAfter(headers?.get("retry-after") ?? null),
    requestId: e.requestId,
  };
}

/** Reads a failed response's body as JSON when it is JSON; anything else becomes `internal`. */
export async function problemFromResponse(res: Response): Promise<ApiProblem> {
  let body: unknown = null;
  try {
    body = await res.json();
  } catch {
    // Not JSON (a proxy page, an empty body): problemFrom maps it by status.
  }
  return problemFrom(res.status, body, res.headers);
}

// One listener: the app sends the student to /masuk when their session is gone.
let sessionExpired: (() => void) | null = null;

/** Sets the one session-expired listener (null removes it). */
export const setSessionExpiredListener = (listener: (() => void) | null): void => {
  sessionExpired = listener;
};

// On these a 401 means "wrong credentials", not "your session ended".
const CREDENTIAL_PATHS: readonly string[] = ["/v1/auth/login", "/v1/auth/register"];

/** Called by the client and the stream reader for every failure. */
export const noteProblem = (path: string, problem: ApiProblem): void => {
  if (problem.status === 401 && !CREDENTIAL_PATHS.includes(path)) sessionExpired?.();
};
