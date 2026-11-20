import { afterEach, describe, expect, test } from "bun:test";
import { buildUrl, createApiClient, type FetchLike } from "./client";
import { setSessionExpiredListener } from "./problem";

type Call = { url: string; init: RequestInit };

const mock = (respond: () => Response | Promise<Response>) => {
  const calls: Call[] = [];
  const fetch: FetchLike = async (url, init) => {
    calls.push({ url, init });
    return respond();
  };
  return { calls, client: createApiClient(fetch) };
};

const errorBody = (code: string, extra: Record<string, unknown> = {}) => ({
  error: { code, message: "English text", requestId: "req-1", ...extra },
});

const STUDENT = {
  id: "0193b2c4-7a1e-7c3f-9d2a-5e8f1b6c4d90",
  username: "demo",
  nickname: "Demo",
  grade: 7,
  theme: "system" as const,
  createdAt: "2026-11-11T08:30:00.000Z",
  hasPassword: true,
  isGoogleLinked: false,
};

afterEach(() => setSessionExpiredListener(null));

describe("api client", () => {
  test("a 200 answers the JSON body; the request is same-origin JSON", async () => {
    const { calls, client } = mock(() => Response.json(STUDENT));
    const r = await client.post("/v1/auth/login", { username: "demo", password: "belajar123" });
    expect(r).toEqual({ ok: true, value: STUDENT });
    expect(calls[0]?.url).toBe("/v1/auth/login");
    expect(calls[0]?.init.method).toBe("POST");
    expect(calls[0]?.init.credentials).toBe("same-origin");
    expect(calls[0]?.init.body).toBe('{"username":"demo","password":"belajar123"}');
    expect(new Headers(calls[0]?.init.headers).get("content-type")).toBe("application/json");
  });

  test("a 204 answers undefined", async () => {
    const { client } = mock(() => new Response(null, { status: 204 }));
    expect(await client.post("/v1/auth/logout", {})).toEqual({ ok: true, value: undefined });
  });

  test("a GET sends no body", async () => {
    const { calls, client } = mock(() => Response.json(STUDENT));
    await client.get("/v1/me");
    expect(calls[0]?.init.method).toBe("GET");
    expect(calls[0]?.init.body).toBeUndefined();
  });

  test("field errors map pointer → code", async () => {
    const { client } = mock(() =>
      Response.json(
        errorBody("invalid_request", {
          errors: [
            { field: "/username", code: "too_short", message: "m" },
            { field: "/grade", code: "out_of_range", message: "m" },
          ],
        }),
        { status: 400 },
      ),
    );
    const r = await client.post("/v1/auth/register", {
      username: "a",
      password: "p",
      nickname: "n",
      grade: 0,
    });
    expect(r.ok).toBe(false);
    if (!r.ok) {
      expect(r.error.status).toBe(400);
      expect(r.error.code).toBe("invalid_request");
      expect(r.error.fieldErrors).toEqual({ "/username": "too_short", "/grade": "out_of_range" });
      expect(r.error.requestId).toBe("req-1");
    }
  });

  test("a 429 carries Retry-After in seconds", async () => {
    const { client } = mock(() =>
      Response.json(errorBody("rate_limited"), { status: 429, headers: { "Retry-After": "61" } }),
    );
    const r = await client.post("/v1/auth/login", { username: "a", password: "b" });
    expect(r.ok ? null : [r.error.code, r.error.retryAfterSeconds]).toEqual(["rate_limited", 61]);
  });

  test("a body that is not the error shape becomes internal with the header's request id", async () => {
    const { client } = mock(
      () =>
        new Response("<html>bad gateway</html>", {
          status: 502,
          headers: { "X-Request-Id": "h1" },
        }),
    );
    const r = await client.get("/v1/me");
    expect(r.ok ? null : [r.error.status, r.error.code, r.error.requestId]).toEqual([
      502,
      "internal",
      "h1",
    ]);
  });

  test("a fetch that throws is the network problem", async () => {
    const client = createApiClient(async () => Promise.reject(new TypeError("Failed to fetch")));
    const r = await client.get("/v1/me");
    expect(r.ok ? null : [r.error.status, r.error.code]).toEqual([0, "network"]);
  });

  test("a 401 notifies the session-expired listener, except on login and register", async () => {
    let expired = 0;
    setSessionExpiredListener(() => {
      expired += 1;
    });
    const { client } = mock(() => Response.json(errorBody("unauthenticated"), { status: 401 }));
    await client.post("/v1/auth/login", { username: "a", password: "b" });
    await client.post("/v1/auth/register", {
      username: "a",
      password: "b",
      nickname: "c",
      grade: 7,
    });
    expect(expired).toBe(0);
    await client.get("/v1/me");
    await client.patch("/v1/me", { nickname: "x" });
    expect(expired).toBe(2);
  });

  test("other failures do not notify the listener", async () => {
    let expired = 0;
    setSessionExpiredListener(() => {
      expired += 1;
    });
    const { client } = mock(() => Response.json(errorBody("permission_denied"), { status: 403 }));
    await client.post("/v1/auth/password", { currentPassword: "a", newPassword: "b" });
    expect(expired).toBe(0);
  });
});

describe("buildUrl", () => {
  test("fills path parameters and drops undefined query values", () => {
    expect(
      buildUrl("/v1/x/{id}", { pathParams: { id: "a b" }, query: { q: 1, n: undefined } }),
    ).toBe("/v1/x/a%20b?q=1");
    expect(buildUrl("/v1/me")).toBe("/v1/me");
  });
});
