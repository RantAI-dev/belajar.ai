import { describe, expect, test } from "bun:test";
import { parseApiError } from "./api-error";

describe("parseApiError", () => {
  test("reads the standard body", () => {
    const e = parseApiError(
      404,
      { error: { code: "not_found", message: "Not found.", requestId: "r1" } },
      "r1",
    );
    expect(e.code).toBe("not_found");
    expect(e.requestId).toBe("r1");
  });

  test("an unknown code or a non-JSON body becomes internal, never a crash", () => {
    expect(parseApiError(500, { error: { code: "made_up" } }, null).code).toBe("internal");
    expect(parseApiError(502, "<html>Bad gateway</html>", "r2").code).toBe("internal");
    expect(parseApiError(502, "<html>Bad gateway</html>", "r2").requestId).toBe("r2");
  });
});
