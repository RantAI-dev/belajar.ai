// Server-sent events over POST: EventSource only does GET, so the answer streams are read with
// fetch. Each feature validates its own `data` shape; this module only frames events.
import type { FetchLike, Operation, PathsWith, RequestBody } from "./client";
import { ERROR_CODES } from "./error-codes";
import {
  type ApiProblem,
  networkProblem,
  noteProblem,
  problemFrom,
  problemFromResponse,
} from "./problem";
import { err, ok, type Result } from "./result";

export type SseEvent = { readonly event: string; readonly data: unknown };

/** JSON when the data is JSON, the raw text otherwise. */
const decodeData = (raw: string): unknown => {
  try {
    return JSON.parse(raw);
  } catch {
    return raw;
  }
};

export type SseParser = {
  /** Feeds the next decoded chunk; chunks may split anywhere, even inside a CRLF. */
  readonly push: (chunk: string) => void;
  /** The stream ended: an event without its closing blank line is dropped, as the spec says. */
  readonly end: () => void;
};

/** The text/event-stream framing (WHATWG HTML, "event stream interpretation"). */
export function createSseParser(onEvent: (e: SseEvent) => void): SseParser {
  let buffer = "";
  let event = "";
  let data: string[] = [];
  // A chunk ending in CR may be the first half of a CRLF.
  let skipLeadingLf = false;

  const dispatch = () => {
    if (data.length > 0)
      onEvent({ event: event === "" ? "message" : event, data: decodeData(data.join("\n")) });
    event = "";
    data = [];
  };

  const line = (text: string) => {
    if (text === "") return dispatch();
    if (text.startsWith(":")) return; // a comment, e.g. the server's `: ping`
    const colon = text.indexOf(":");
    const field = colon === -1 ? text : text.slice(0, colon);
    let value = colon === -1 ? "" : text.slice(colon + 1);
    if (value.startsWith(" ")) value = value.slice(1);
    if (field === "event") event = value;
    else if (field === "data") data.push(value);
    // `id` and `retry` matter only for EventSource reconnects, which a POST does not do.
  };

  return {
    push(chunk) {
      let text = chunk;
      if (skipLeadingLf && text.startsWith("\n")) text = text.slice(1);
      skipLeadingLf = false;
      buffer += text;
      let start = 0;
      for (let i = 0; i < buffer.length; i++) {
        const c = buffer[i];
        if (c !== "\n" && c !== "\r") continue;
        line(buffer.slice(start, i));
        if (c === "\r") {
          if (i + 1 === buffer.length) skipLeadingLf = true;
          else if (buffer[i + 1] === "\n") i++;
        }
        start = i + 1;
      }
      buffer = buffer.slice(start);
    },
    end() {
      buffer = "";
      event = "";
      data = [];
    },
  };
}

/** How a stream that did not fail finished. */
export type StreamEnd = "ended" | "aborted";

export type StreamOptions = {
  readonly onEvent: (e: SseEvent) => void;
  readonly signal?: AbortSignal;
  readonly fetch?: FetchLike;
};

const globalFetch: FetchLike = (input, init) => globalThis.fetch(input, init);

/**
 * POSTs `body` and delivers the answer's events in order. Resolves when the stream ends; an
 * `event: error` (the standard error body) or a non-2xx answer resolves as that problem.
 */
export async function postStream<P extends PathsWith<"post">>(
  path: P,
  body: RequestBody<Operation<P, "post">>,
  options: StreamOptions,
): Promise<Result<StreamEnd, ApiProblem>> {
  const { onEvent, signal } = options;
  const fetchImpl = options.fetch ?? globalFetch;
  const fail = (problem: ApiProblem) => {
    noteProblem(path, problem);
    return err(problem);
  };
  let res: Response;
  try {
    res = await fetchImpl(path, {
      method: "POST",
      credentials: "same-origin",
      headers: { accept: "text/event-stream", "content-type": "application/json" },
      body: JSON.stringify(body),
      ...(signal !== undefined ? { signal } : {}),
    });
  } catch {
    return signal?.aborted === true ? ok("aborted") : fail(networkProblem());
  }
  if (!res.ok) return fail(await problemFromResponse(res));
  if (res.body === null) return ok("ended");

  // An object, so the callback's writes are visible to the narrowing below.
  const stop: { problem: ApiProblem | null } = { problem: null };
  const parser = createSseParser((e) => {
    if (stop.problem !== null) return;
    if (e.event === "error") {
      // The stream already said 200, so the status comes from the code's registry row.
      const parsed = problemFrom(200, e.data, res.headers);
      stop.problem = { ...parsed, status: statusOf(parsed) };
      return;
    }
    onEvent(e);
  });
  const reader = res.body.getReader();
  const decoder = new TextDecoder();
  try {
    for (;;) {
      const { done, value } = await reader.read();
      if (done) break;
      parser.push(decoder.decode(value, { stream: true }));
      if (stop.problem !== null) {
        await reader.cancel();
        break;
      }
    }
    parser.push(decoder.decode());
  } catch {
    if (signal?.aborted === true) return ok("aborted");
    return fail(networkProblem());
  }
  parser.end();
  if (stop.problem !== null) return fail(stop.problem);
  return ok(signal?.aborted === true ? "aborted" : "ended");
}

const statusOf = (p: ApiProblem): number => (p.code === "network" ? 0 : ERROR_CODES[p.code].status);
