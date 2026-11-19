import { describe, expect, test } from "bun:test";
import { setSessionExpiredListener } from "./problem";
import { createSseParser, postStream, type SseEvent } from "./sse";

const parse = (chunks: readonly string[]): SseEvent[] => {
  const out: SseEvent[] = [];
  const p = createSseParser((e) => out.push(e));
  for (const c of chunks) p.push(c);
  p.end();
  return out;
};

const STREAM =
  'event: delta\ndata: {"text":"Ha"}\n\n: ping\n\nevent: delta\ndata: {"text":"lo"}\n\nevent: done\ndata: {}\n\n';

describe("createSseParser", () => {
  test("frames events in order and decodes JSON data", () => {
    expect(parse([STREAM])).toEqual([
      { event: "delta", data: { text: "Ha" } },
      { event: "delta", data: { text: "lo" } },
      { event: "done", data: {} },
    ]);
  });

  test("any split of the stream gives the same events", () => {
    const whole = parse([STREAM]);
    for (let i = 1; i < STREAM.length; i++) {
      expect(parse([STREAM.slice(0, i), STREAM.slice(i)])).toEqual(whole);
    }
    expect(parse(STREAM.split(""))).toEqual(whole);
  });

  test("CRLF and lone CR end lines, even when a CRLF is split across chunks", () => {
    const crlf = STREAM.replaceAll("\n", "\r\n");
    expect(parse([crlf])).toEqual(parse([STREAM]));
    for (let i = 1; i < crlf.length; i++) {
      expect(parse([crlf.slice(0, i), crlf.slice(i)])).toEqual(parse([STREAM]));
    }
    expect(parse([STREAM.replaceAll("\n", "\r")])).toEqual(parse([STREAM]));
  });

  test("multi-line data joins with newlines; non-JSON stays text", () => {
    expect(parse(["data: baris satu\ndata: baris dua\n\n"])).toEqual([
      { event: "message", data: "baris satu\nbaris dua" },
    ]);
    expect(parse(['data: {"a":\ndata: 1}\n\n'])).toEqual([{ event: "message", data: { a: 1 } }]);
  });

  test("comments, keep-alives and events without data deliver nothing", () => {
    expect(parse([": ping\n\n", "\n\n", "event: lonely\n\n", ":\n"])).toEqual([]);
  });

  test("an event cut off by the end of the stream is dropped", () => {
    expect(parse(['event: delta\ndata: {"text":"x"}\n'])).toEqual([]);
  });

  test("only one leading space of a value is removed; ids are ignored", () => {
    expect(parse(["id: 7\nevent:x\ndata:  two\n\n"])).toEqual([{ event: "x", data: " two" }]);
  });
});

const streamResponse = (chunks: readonly string[], init: ResponseInit = {}): Response => {
  const enc = new TextEncoder();
  const body = new ReadableStream<Uint8Array>({
    start(c) {
      for (const chunk of chunks) c.enqueue(enc.encode(chunk));
      c.close();
    },
  });
  return new Response(body, { headers: { "content-type": "text/event-stream" }, ...init });
};

describe("postStream", () => {
  test("delivers events and resolves when the stream ends", async () => {
    const seen: SseEvent[] = [];
    const r = await postStream(
      "/v1/auth/login",
      { username: "a", password: "b" },
      {
        onEvent: (e) => seen.push(e),
        fetch: async () => streamResponse([STREAM.slice(0, 9), STREAM.slice(9)]),
      },
    );
    expect(r).toEqual({ ok: true, value: "ended" });
    expect(seen.map((e) => e.event)).toEqual(["delta", "delta", "done"]);
  });

  test("sends a JSON POST that asks for an event stream", async () => {
    let sent: RequestInit | undefined;
    await postStream(
      "/v1/auth/logout",
      {},
      {
        onEvent: () => {},
        fetch: async (_url, init) => {
          sent = init;
          return streamResponse([]);
        },
      },
    );
    expect(sent?.method).toBe("POST");
    expect(sent?.credentials).toBe("same-origin");
    expect(new Headers(sent?.headers).get("accept")).toBe("text/event-stream");
  });

  test("a JSON error before the stream becomes the same problem as the client's", async () => {
    const r = await postStream(
      "/v1/auth/logout",
      {},
      {
        onEvent: () => {},
        fetch: async () =>
          Response.json(
            { error: { code: "rate_limited", message: "x", requestId: "r9" } },
            { status: 429, headers: { "retry-after": "90" } },
          ),
      },
    );
    expect(r.ok).toBe(false);
    if (!r.ok) {
      expect(r.error.status).toBe(429);
      expect(r.error.code).toBe("rate_limited");
      expect(r.error.retryAfterSeconds).toBe(90);
      expect(r.error.requestId).toBe("r9");
    }
  });

  test("event: error ends the stream with that problem; later events are not delivered", async () => {
    const seen: string[] = [];
    const r = await postStream(
      "/v1/auth/logout",
      {},
      {
        onEvent: (e) => seen.push(e.event),
        fetch: async () =>
          streamResponse([
            'event: delta\ndata: {"text":"a"}\n\n',
            'event: error\ndata: {"error":{"code":"upstream_failed","message":"m","requestId":"r1"}}\n\n',
            'event: delta\ndata: {"text":"b"}\n\n',
          ]),
      },
    );
    expect(seen).toEqual(["delta"]);
    expect(r.ok).toBe(false);
    if (!r.ok) {
      expect(r.error.code).toBe("upstream_failed");
      expect(r.error.status).toBe(502);
      expect(r.error.requestId).toBe("r1");
    }
  });

  test("a failed fetch is a network problem; an aborted one is not a problem", async () => {
    const down = await postStream(
      "/v1/auth/logout",
      {},
      {
        onEvent: () => {},
        fetch: async () => Promise.reject(new TypeError("offline")),
      },
    );
    expect(down.ok ? null : down.error.code).toBe("network");

    const ctrl = new AbortController();
    ctrl.abort();
    const aborted = await postStream(
      "/v1/auth/logout",
      {},
      {
        onEvent: () => {},
        signal: ctrl.signal,
        fetch: async () => Promise.reject(new DOMException("aborted", "AbortError")),
      },
    );
    expect(aborted).toEqual({ ok: true, value: "aborted" });
  });

  test("aborting mid-stream resolves as aborted", async () => {
    const ctrl = new AbortController();
    const seen: string[] = [];
    const body = new ReadableStream<Uint8Array>({
      start(c) {
        c.enqueue(new TextEncoder().encode("data: 1\n\n"));
        ctrl.signal.addEventListener("abort", () =>
          c.error(new DOMException("aborted", "AbortError")),
        );
      },
    });
    const r = await postStream(
      "/v1/auth/logout",
      {},
      {
        signal: ctrl.signal,
        onEvent: (e) => {
          seen.push(String(e.data));
          ctrl.abort();
        },
        fetch: async () => new Response(body),
      },
    );
    expect(seen).toEqual(["1"]);
    expect(r).toEqual({ ok: true, value: "aborted" });
  });

  test("a 401 tells the session-expired listener", async () => {
    let expired = 0;
    setSessionExpiredListener(() => {
      expired += 1;
    });
    await postStream(
      "/v1/auth/logout",
      {},
      {
        onEvent: () => {},
        fetch: async () =>
          Response.json(
            { error: { code: "unauthenticated", message: "m", requestId: "r" } },
            { status: 401 },
          ),
      },
    );
    setSessionExpiredListener(null);
    expect(expired).toBe(1);
  });
});
