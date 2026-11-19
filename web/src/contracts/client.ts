// The typed client every feature uses. Paths, bodies and answers come from the generated
// api.gen.ts, so a server change that breaks a caller breaks the typecheck, not the student.
import type { paths } from "./api.gen";
import {
  type ApiProblem,
  networkProblem,
  noteProblem,
  problemFrom,
  problemFromResponse,
} from "./problem";
import { err, ok, type Result } from "./result";

type Method = "get" | "post" | "put" | "patch" | "delete";

/** The paths that answer `method`. */
export type PathsWith<M extends Method> = {
  [P in keyof paths]: [NonNullable<paths[P][M]>] extends [never] ? never : P;
}[keyof paths];

export type Operation<P extends keyof paths, M extends Method> = NonNullable<paths[P][M]>;

export type RequestBody<O> = O extends {
  requestBody: { content: { "application/json": infer B } };
}
  ? B
  : never;

type JsonOf<R> = R extends { content: { "application/json": infer T } } ? T : undefined;

/** The JSON of every 2xx answer of an operation; `undefined` for 204. */
export type SuccessBody<O> = O extends { responses: infer R }
  ? { [K in keyof R]: K extends 200 | 201 | 202 | 204 ? JsonOf<R[K]> : never }[keyof R]
  : never;

export type ApiResult<T> = Result<T, ApiProblem>;

export type RequestOptions = {
  /** Values for `{name}` segments in the path. */
  readonly pathParams?: Readonly<Record<string, string>>;
  readonly query?: Readonly<Record<string, string | number | boolean | undefined>>;
  readonly signal?: AbortSignal;
};

export type FetchLike = (input: string, init: RequestInit) => Promise<Response>;

export type ApiClient = {
  get<P extends PathsWith<"get">>(
    path: P,
    options?: RequestOptions,
  ): Promise<ApiResult<SuccessBody<Operation<P, "get">>>>;
  delete<P extends PathsWith<"delete">>(
    path: P,
    options?: RequestOptions,
  ): Promise<ApiResult<SuccessBody<Operation<P, "delete">>>>;
  post<P extends PathsWith<"post">>(
    path: P,
    body: RequestBody<Operation<P, "post">>,
    options?: RequestOptions,
  ): Promise<ApiResult<SuccessBody<Operation<P, "post">>>>;
  put<P extends PathsWith<"put">>(
    path: P,
    body: RequestBody<Operation<P, "put">>,
    options?: RequestOptions,
  ): Promise<ApiResult<SuccessBody<Operation<P, "put">>>>;
  patch<P extends PathsWith<"patch">>(
    path: P,
    body: RequestBody<Operation<P, "patch">>,
    options?: RequestOptions,
  ): Promise<ApiResult<SuccessBody<Operation<P, "patch">>>>;
};

/** `/v1/x/{id}` + `{id: "a b"}` + `{q: 1}` → `/v1/x/a%20b?q=1`. */
export function buildUrl(path: string, options: RequestOptions = {}): string {
  const filled = path.replace(/\{([^}]+)\}/g, (whole, name: string) => {
    const value = options.pathParams?.[name];
    return value === undefined ? whole : encodeURIComponent(value);
  });
  const params = new URLSearchParams();
  for (const [k, v] of Object.entries(options.query ?? {})) {
    if (v !== undefined) params.set(k, String(v));
  }
  const qs = params.toString();
  return qs.length > 0 ? `${filled}?${qs}` : filled;
}

// Read at call time so tests (and the dev tools) can swap globalThis.fetch.
const globalFetch: FetchLike = (input, init) => globalThis.fetch(input, init);

export function createApiClient(fetchImpl: FetchLike = globalFetch): ApiClient {
  async function send(
    method: string,
    path: string,
    body: unknown,
    options: RequestOptions = {},
  ): Promise<ApiResult<never>> {
    const init: RequestInit = {
      method: method.toUpperCase(),
      // The session cookie is HttpOnly and set by the server; the client only sends it along.
      credentials: "same-origin",
      headers:
        body === undefined
          ? { accept: "application/json" }
          : {
              accept: "application/json",
              "content-type": "application/json",
            },
      ...(body === undefined ? {} : { body: JSON.stringify(body) }),
      ...(options.signal !== undefined ? { signal: options.signal } : {}),
    };
    let res: Response;
    try {
      res = await fetchImpl(buildUrl(path, options), init);
    } catch {
      const problem = networkProblem();
      noteProblem(path, problem);
      return err(problem);
    }
    if (!res.ok) {
      const problem = await problemFromResponse(res);
      noteProblem(path, problem);
      return err(problem);
    }
    if (res.status === 204) return ok(undefined as never);
    try {
      return ok((await res.json()) as never);
    } catch {
      // A 2xx that is not the JSON the contract promises (a captive portal, a broken proxy).
      return err(problemFrom(res.status, null, res.headers));
    }
  }
  return {
    get: (path, options) => send("get", path, undefined, options),
    delete: (path, options) => send("delete", path, undefined, options),
    post: (path, body, options) => send("post", path, body, options),
    put: (path, body, options) => send("put", path, body, options),
    patch: (path, body, options) => send("patch", path, body, options),
  };
}

/** The app's client: same origin, the browser's fetch. */
export const api: ApiClient = createApiClient();
