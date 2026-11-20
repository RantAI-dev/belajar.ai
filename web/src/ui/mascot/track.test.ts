import { expect, test } from "bun:test";
import { lookOffset, MAX_LOOK } from "./track";

const box = { left: 100, top: 100, width: 100, height: 100 };
const eyes = { x: 150, y: 158 };

test("a pointer on the eyes leaves the face centred", () => {
  expect(lookOffset(eyes, box)).toEqual({ x: 0, y: 0 });
});

test("the face turns toward the pointer", () => {
  const right = lookOffset({ x: 180, y: 158 }, box);
  expect(right.x).toBeGreaterThan(0);
  expect(right.y).toBe(0);
  const up = lookOffset({ x: 150, y: 120 }, box);
  expect(up.y).toBeLessThan(0);
  expect(up.x).toBe(0);
});

test("a far pointer is clamped to the maximum", () => {
  const far = lookOffset({ x: 5000, y: -5000 }, box);
  expect(Math.hypot(far.x, far.y)).toBeCloseTo(MAX_LOOK, 0);
  expect(lookOffset({ x: 9e9, y: 158 }, box)).toEqual({ x: MAX_LOOK, y: 0 });
});

test("a near pointer moves the face less than a far one", () => {
  const near = lookOffset({ x: 170, y: 158 }, box);
  const far = lookOffset({ x: 400, y: 158 }, box);
  expect(near.x).toBeLessThan(far.x);
});

test("an empty box or a bad pointer gives a finite offset", () => {
  expect(lookOffset({ x: 10, y: 10 }, { left: 0, top: 0, width: 0, height: 0 })).toEqual({
    x: 4.2,
    y: 4.2,
  });
  expect(lookOffset({ x: Number.NaN, y: 0 }, box)).toEqual({ x: 0, y: 0 });
});
