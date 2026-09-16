# @openvtc/trust-tasks-capability-client

Client-side wire helpers for the **capability** [Trust Task](https://trusttasks.org)
families — `governance/capability/*` (enable / disable / list a community
capability) and `git-trust/*` (grant / revoke commit-signing trust).

This package owns the **documents**, not a transport: it builds request
documents, parses inbound envelope replies, and classifies them. A capability
producer (a community service) and a management UI share this wire layer, so
they cannot drift on the contract. Zero runtime dependencies beyond
`@openvtc/trust-tasks`; it is the TypeScript port of the Rust
`trust-tasks-capability-client` crate.

**Signing is deliberately not here.** Attach a Data Integrity proof with
[`@openvtc/trust-tasks-proof`](https://www.npmjs.com/package/@openvtc/trust-tasks-proof)
over the document minus its `proof` member (`eddsa-jcs-2022`), so this package
stays free of any crypto dependency.

## Install

```sh
npm install @openvtc/trust-tasks-capability-client
```

## Build a request, hold its thread, act on the reply

```ts
import {
  buildGitTrustGrant,
  correlationThread,
  classifyGitTrustReply,
} from "@openvtc/trust-tasks-capability-client";

const doc = buildGitTrustGrant(authorityDid, registryDid, subjectDid, "openvtc");
const thread = correlationThread(doc); // hold this from the moment you send

// ...sign `doc`, send it over your transport, receive an envelope body...

const outcome = classifyGitTrustReply(replyDoc, thread);
switch (outcome?.kind) {
  case "success":
  case "idempotentSuccess": // already_granted / not_granted — the end state holds
    // the write is done
    break;
  case "rejected":
    console.error(outcome.code, outcome.message);
    break;
  case undefined:
    // not an answer to this request — keep waiting
    break;
}
```

## Retries vs. new attempts

A **retry** is a bit-for-bit resend of the *same* document (same `id`); the
consumer's SPEC §7.2 item-11 record absorbs it and returns the first outcome. A
**new attempt** is a different document — anything that changes the bytes,
including a re-stamped `issuedAt` or a re-signed `proof`, makes it one — and MUST
carry a fresh `id`. Use `newAttempt(previous)` for that (it clears `proof` and
re-stamps `issuedAt`); reusing an `id` with altered content is rejected with
`idConflict`.

## Governance management UIs

```ts
import { buildListDocument, correlationThread, parseEnvelopeReply } from "@openvtc/trust-tasks-capability-client";

const list = buildListDocument(myDid, vtcDid);
const thread = correlationThread(list);
// ...on an inbound envelope body:
const reply = parseEnvelopeReply(body, thread);
if (reply?.kind === "listing") render(reply.entries); // CapabilitySummary[]
```

Idempotent success (`git-trust`) and every classification are keyed on the SPEC
§8.5 **extended error code**, never on the non-normative free-text `message`.
