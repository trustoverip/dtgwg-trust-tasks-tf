/**
 * The shared forward-routing vectors in `fixtures/consume/extends-routing.json`
 * at the repository root, which the Rust, Go and Dart runtimes also run: a
 * consumer implementing one version reads a later one whose declared `extends`
 * chain reaches it as that version, pruned, and answers it as the received
 * version; an unreached version of the same specification is
 * `unsupportedVersion`, anything else `unsupportedType` (SPEC §5.2, §7.2 item 3).
 */

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { describe, it } from "node:test";

import { readForSpec, respondWith, type SpecPolicy } from "../src/_runtime/index.js";

interface Vectors {
  implements: { typeUri: string; payloadSchema: string };
  extends: Record<string, string>;
  cases: {
    name: string;
    document: string;
    verdict: string;
    extended?: boolean;
    payload?: unknown;
    answerType?: string;
  }[];
}

const vectors = JSON.parse(
  readFileSync(new URL("../../../fixtures/consume/extends-routing.json", import.meta.url), "utf8"),
) as Vectors;

const spec: SpecPolicy = {
  typeUri: vectors.implements.typeUri,
  isBearer: false,
  isProofRequired: false,
  isRecipientRequired: false,
  payloadSchema: JSON.parse(vectors.implements.payloadSchema),
} as SpecPolicy;

describe("shared extends-routing vectors (fixtures/consume/extends-routing.json)", () => {
  assert.ok(vectors.cases.length > 0);
  for (const c of vectors.cases) {
    it(c.name, () => {
      const read = readForSpec<unknown>(JSON.parse(c.document), spec, { index: vectors.extends });
      if (c.verdict !== "accept") {
        assert.equal(read.kind, "refused", `expected ${c.verdict}`);
        if (read.kind === "refused") assert.equal(read.reason.code, c.verdict);
        return;
      }
      assert.equal(read.kind, "accepted", read.kind === "refused" ? read.reason.message : "");
      if (read.kind !== "accepted") return;
      assert.deepEqual(read.doc.payload, c.payload);
      assert.equal(read.extended, c.extended);
      const answer = respondWith(read.doc, "urn:uuid:answer", {});
      assert.equal(answer.type, c.answerType);
    });
  }
});
