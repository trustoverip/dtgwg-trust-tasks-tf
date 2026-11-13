/**
 * SPEC §4.5.2 — a namespace marked critical is understood or refused.
 *
 * Runs the shared vectors in `fixtures/consume/ext-critical.json` at the
 * repository root, which the Rust, Go and Dart runtimes also run, and drives
 * the marking through `consumeInbound`.
 */

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { describe, it } from "node:test";

import {
  checkCriticalExtensions,
  consequentialChecks,
  consumeInbound,
  InMemoryReplayGuard,
  respondWith,
  UnauthenticatedTransport,
  type SpecPolicy,
  type TrustTaskDocument,
} from "../src/_runtime/index.js";

const fixture = (name: string): unknown =>
  JSON.parse(
    readFileSync(new URL(`../../../fixtures/consume/${name}`, import.meta.url), "utf8"),
  );

const SCHEMA = fixture("ext-critical.schema.json");
const VECTORS = fixture("ext-critical.json") as {
  cases: {
    name: string;
    payload: unknown;
    recognized: string[];
    expect: string;
    schemaless?: boolean;
  }[];
};

describe("SPEC §4.5.2 — shared vectors", () => {
  for (const c of VECTORS.cases) {
    it(c.name, () => {
      const got = checkCriticalExtensions(
        c.payload,
        c.schemaless === true ? undefined : SCHEMA,
        c.recognized,
      );
      assert.equal(got === null ? "ok" : got.code, c.expect);
    });
  }
});

const MARKED: SpecPolicy = {
  typeUri: "https://trusttasks.org/spec/conformance/marked/0.1",
  isBearer: false,
  isProofRequired: false,
  isRecipientRequired: false,
  payloadSchema: SCHEMA,
};

function marked(id: string): TrustTaskDocument<{ subject: string }> {
  return {
    id,
    type: MARKED.typeUri,
    issuedAt: new Date(Math.floor(Date.now() / 1000) * 1000).toISOString().replace(".000", ""),
    payload: {
      subject: "s",
      ext: { "org.example.a": 1 },
      extCritical: ["org.example.a"],
    } as { subject: string },
  };
}

describe("SPEC §4.5.2 — consumeInbound", () => {
  it("refuses an unrecognized critical namespace before the handler and the record", async () => {
    const guard = new InMemoryReplayGuard();
    let runs = 0;
    const run = (recognized: readonly string[]) =>
      consumeInbound<{ subject: string }, { ok: boolean }>({
        transport: new UnauthenticatedTransport(),
        spec: MARKED,
        proofPolicy: { kind: "acceptUnverified" },
        payloadPolicy: { kind: "acceptUnvalidated" },
        checks: { ...consequentialChecks(guard), recognizedExtensions: recognized },
        doc: marked("urn:uuid:00000000-0000-4000-8000-0000000000c1"),
        myVid: "did:web:consumer.example",
        now: Date.now(),
        newErrorId: () => "err-1",
        handler: (d) => {
          runs += 1;
          return respondWith(d, "resp-1", { ok: true });
        },
      });

    const refused = await run([]);
    assert.equal(refused.kind, "rejected");
    if (refused.kind === "rejected") {
      assert.equal(refused.error.payload.code, "unsupportedExtension");
      assert.equal(refused.error.payload.retryable, false);
    }
    assert.equal(runs, 0, "the handler must not run");

    // The refusal wrote no record: the same document, consumed by a consumer
    // that implements the namespace, runs.
    const handled = await run(["org.example.a"]);
    assert.equal(handled.kind, "handled");
    assert.equal(runs, 1);
  });
});
