// The client side of the one error body (docs/conventions.md). The server builds it in
// rust/crates/server/src/api_error.rs; the web app only reads it and chooses its own text by code.
import { ERROR_CODES, type ErrorCode } from "./error-codes";

export type FieldError = { field: string; code: string; message: string };

export type ApiError = {
  code: ErrorCode;
  message: string;
  requestId: string;
  errors: FieldError[];
};

const isErrorCode = (value: unknown): value is ErrorCode =>
  typeof value === "string" && Object.hasOwn(ERROR_CODES, value);

/** Reads an error body. Anything that is not one (a proxy page, a broken body) becomes `internal`. */
export function parseApiError(
  status: number,
  body: unknown,
  headerRequestId: string | null,
): ApiError {
  const inner =
    typeof body === "object" && body !== null && "error" in body
      ? (body as { error: Record<string, unknown> }).error
      : undefined;
  const code = isErrorCode(inner?.code) ? inner.code : status === 404 ? "not_found" : "internal";
  return {
    code,
    message: typeof inner?.message === "string" ? inner.message : ERROR_CODES[code].message,
    requestId: typeof inner?.requestId === "string" ? inner.requestId : (headerRequestId ?? ""),
    errors: Array.isArray(inner?.errors) ? (inner.errors as FieldError[]) : [],
  };
}
