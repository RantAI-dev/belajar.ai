import { describe, expect, test } from "bun:test";
import {
  decidePose,
  FAILURE_MS,
  GREETING,
  type PoseInput,
  pokeReaction,
  SAD_MS,
  SLEEP_AFTER_MS,
} from "./pose";

const at = (now: number, over: Partial<PoseInput> = {}): PoseInput => ({
  now,
  focus: null,
  passwordShown: false,
  hover: false,
  lastActivityAt: now,
  reaction: null,
  outcome: null,
  ...over,
});

describe("decidePose", () => {
  test("a successful sign-in celebrates, whatever else is going on", () => {
    const p = decidePose(
      at(5000, { focus: "password", hover: true, outcome: { kind: "success", at: 4000 } }),
    );
    expect(p.expression).toBe("celebration");
    expect(p.arms).toEqual(["up", "up"]);
  });

  test("a failed sign-in is sad, then confused, then back to normal", () => {
    const outcome = { kind: "failure", at: 1000 } as const;
    expect(decidePose(at(1000, { outcome })).expression).toBe("sad");
    expect(decidePose(at(1000 + SAD_MS, { outcome })).expression).toBe("confused");
    expect(decidePose(at(1000 + FAILURE_MS, { outcome })).look).toBe("pointer");
    expect(decidePose(at(1000 + FAILURE_MS, { outcome })).expression).toBe("happy");
  });

  test("covers both eyes on a hidden password and peeks when it is shown", () => {
    const hidden = decidePose(at(0, { focus: "password" }));
    expect(hidden.arms).toEqual(["cover", "cover"]);
    expect(hidden.look).toBe("none");
    const shown = decidePose(at(0, { focus: "password", passwordShown: true }));
    expect(shown.arms).toEqual(["cover", "rest"]);
    expect(shown.look).toBe("field");
  });

  test("watches the caret of the username field", () => {
    expect(decidePose(at(0, { focus: "text" })).look).toBe("field");
  });

  test("falls asleep after a long idle, but not while a field is focused", () => {
    const idle = { lastActivityAt: 0 };
    expect(decidePose(at(SLEEP_AFTER_MS + 1, idle)).expression).toBe("sleepy");
    expect(decidePose(at(SLEEP_AFTER_MS + 1, { ...idle, focus: "text" })).expression).not.toBe(
      "sleepy",
    );
    expect(decidePose(at(SLEEP_AFTER_MS - 1, idle)).expression).not.toBe("sleepy");
  });

  test("a hover waves, a reaction shows for its time and then ends", () => {
    expect(decidePose(at(0, { hover: true })).arms).toEqual(["rest", "wave"]);
    const reaction = { pose: GREETING, at: 0, ms: 1000 };
    expect(decidePose(at(500, { reaction }))).toBe(GREETING);
    expect(decidePose(at(1000, { reaction }))).not.toBe(GREETING);
  });
});

describe("pokeReaction", () => {
  test("cycles through different reactions", () => {
    const seen = [0, 1, 2, 3].map((n) => pokeReaction(at(0), n)?.pose.expression);
    expect(new Set(seen).size).toBe(4);
  });

  test("is ignored while the sign-in result is showing", () => {
    expect(pokeReaction(at(10, { outcome: { kind: "success", at: 0 } }), 0)).toBeNull();
    expect(pokeReaction(at(10, { outcome: { kind: "failure", at: 0 } }), 0)).toBeNull();
    expect(pokeReaction(at(FAILURE_MS, { outcome: { kind: "failure", at: 0 } }), 0)).not.toBeNull();
  });
});
