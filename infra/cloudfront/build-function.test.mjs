// The deployed CloudFront function must fit CloudFront Functions' code size
// limit, or `update-function` fails with FunctionSizeLimitExceeded — which is
// how the source outgrew it unnoticed (#1217). Fails a little before the cap,
// so the next rule added gets a warning in CI rather than at deploy time.

import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { buildFunction, FUNCTION_SIZE_LIMIT } from "./build-function.mjs";

describe("deployable CloudFront function", () => {
  const code = buildFunction();
  const size = Buffer.byteLength(code, "utf8");

  it(`stays well under CloudFront's ${FUNCTION_SIZE_LIMIT}-byte limit (${size} bytes)`, () => {
    assert.ok(size <= FUNCTION_SIZE_LIMIT * 0.9, `${size} bytes is within 10% of the ${FUNCTION_SIZE_LIMIT}-byte limit`);
  });

  it("still declares the bare handler CloudFront calls", () => {
    assert.match(code, /function handler\(event\)/);
  });

  it("uses no template literals (plain quoted strings only)", () => {
    assert.ok(!code.includes("`"));
  });
});
