# trust_tasks_capability_client

Client-side wire helpers for the **capability** [Trust Task](https://trusttasks.org)
families — `governance/capability/*` (enable / disable / list a community
capability) and `git-trust/*` (grant / revoke commit-signing trust).

This package owns the **documents**, not a transport: it builds request
documents, parses inbound envelope replies, and classifies them. A capability
producer (a community service) and a management UI share this wire layer, so
they cannot drift on the contract. It is the Dart port of the Rust
`trust-tasks-capability-client` crate — **pure wire logic, no crypto**.

**Signing is deliberately not here.** Attach a Data Integrity proof with
[`package:trust_tasks_proof`](https://pub.dev/packages/trust_tasks_proof) over the
document minus its `proof` member (`eddsa-jcs-2022`).

## Build a request, hold its thread, act on the reply

```dart
import 'package:trust_tasks_capability_client/trust_tasks_capability_client.dart';

final doc = buildGitTrustGrant(authorityDid, registryDid, subjectDid, 'openvtc');
final thread = correlationThread(doc); // hold this from the moment you send

// ...sign `doc`, send it over your transport, receive an envelope body...

switch (classifyGitTrustReply(replyDoc, thread)) {
  case WriteSuccess():
  case WriteIdempotentSuccess(): // already_granted / not_granted — the end state holds
    // the write is done
  case WriteRejected(:final code, :final message):
    log('$code $message');
  case null:
    // not an answer to this request — keep waiting
}
```

The outcome and reply types are **sealed**, so a `switch` over them is
exhaustive.

## Retries vs. new attempts

A **retry** is a bit-for-bit resend of the *same* document (same `id`); the
consumer's SPEC §7.2 item-11 record absorbs it and returns the first outcome. A
**new attempt** is a different document — anything that changes the bytes,
including a re-stamped `issuedAt` or a re-signed `proof`, makes it one — and MUST
carry a fresh `id`. Use `newAttempt(previous)` for that (it clears `proof` and
re-stamps `issuedAt`); reusing an `id` with altered content is `idConflict`.

## Governance management UIs

```dart
final list = buildListDocument(myDid, vtcDid);
final thread = correlationThread(list);
// ...on an inbound envelope body:
final reply = parseEnvelopeReply(body, thread);
if (reply is CapabilityListing) render(reply.entries); // List<CapabilitySummary>
```

Idempotent success and every classification are keyed on the SPEC §8.5 **extended
error code**, never on the non-normative free-text `message`.
