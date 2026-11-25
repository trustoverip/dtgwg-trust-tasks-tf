// The demo's own tests: hermetic (no network, no files outside this example),
// deterministic (fixed keys, ids and clock), and run with `npm test`.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

import { MANAGEMENT_TOKEN, PURPOSE, World, runDemo } from "../src/demo.js";
import { HcpEntriesQuery_v0_1 as query } from "../src/trust-tasks/index.js";

const GOLDEN = new URL("../../transcript.golden.txt", import.meta.url);

test("the five steps print exactly the golden transcript (shared by every language's port)", async () => {
  const { transcript } = await runDemo();
  assert.equal(transcript, readFileSync(GOLDEN, "utf8"));
});

test("a query after revoke fails with hcp:invalidToken (HCP 6.2.2)", async () => {
  const { postRevokeError } = await runDemo();
  assert.equal(postRevokeError.type, "https://trusttasks.org/spec/trust-task-error/0.6");
  assert.equal(postRevokeError.payload.code, "hcp:invalidToken");
});

/** A world after the demo has run, with a query payload that would otherwise be well-formed. */
async function afterDemo() {
  const world = new World();
  await runDemo(world);
  const payload: query.Payload = { query: "Anything else?", purpose: PURPOSE, accessToken: MANAGEMENT_TOKEN };
  return { world, payload };
}

test("a document altered after signing is refused with proofInvalid", async () => {
  const { world, payload } = await afterDemo();
  const { contextClient: C, provider: P } = world;
  const signed = world.request(C, P, `${query.TYPE_URI}#request`, payload);
  const tampered = { ...signed, payload: { ...signed.payload, query: "Tell me everything." } };
  const reply = await world.exchange(C, P, tampered);
  assert.equal(reply.payload.code, "proofInvalid");
});

test("a correctly signed document that fails its schema is refused with malformedRequest", async () => {
  const { world, payload } = await afterDemo();
  const { contextClient: C, provider: P } = world;
  const { purpose: _dropped, ...noPurpose } = payload;
  const reply = await world.exchange(C, P, world.request(C, P, `${query.TYPE_URI}#request`, noPurpose as query.Payload));
  assert.equal(reply.payload.code, "malformedRequest");
});
