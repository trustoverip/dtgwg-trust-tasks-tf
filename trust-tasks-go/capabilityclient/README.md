# trust-tasks-go/capabilityclient

Client-side wire helpers for the **capability** [Trust Task](https://trusttasks.org)
families — `governance/capability/*` (enable / disable / list a community
capability) and `git-trust/*` (grant / revoke commit-signing trust).

This module owns the **documents**, not a transport: it builds request
documents, parses inbound envelope replies, and classifies them. A capability
producer (a community service) and a management UI share this wire layer, so
they cannot drift on the contract. It is the Go port of the Rust
`trust-tasks-capability-client` crate — **pure wire logic, no crypto and no
transport, and no third-party dependency** (only the core module).

**Signing is deliberately not here.** Attach a Data Integrity proof with
[`trust-tasks-go/proof`](../proof) over the document minus its `proof` member
(`eddsa-jcs-2022`).

This is a **separate module** from the core, like `tsp`/`proof`/`didcomm`.

## Install

```sh
go get github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/capabilityclient
```

## Build a request, hold its thread, act on the reply

```go
import cc "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/capabilityclient"

doc := cc.BuildGitTrustGrant(authorityDID, registryDID, subjectDID, "openvtc")
thread := cc.CorrelationThread(doc) // hold this from the moment you send

// ...sign `doc`, send it over your transport, receive an envelope body...

outcome, ok := cc.ClassifyGitTrustReply(replyDoc, thread)
switch {
case !ok:
    // not an answer to this request — keep waiting
case outcome.Kind == cc.WriteSuccess, outcome.Kind == cc.WriteIdempotentSuccess:
    // the write is done (idempotent = already_granted / not_granted)
case outcome.Kind == cc.WriteRejected:
    log.Println(outcome.Code, outcome.Message)
}
```

## Retries vs. new attempts

A **retry** is a bit-for-bit resend of the *same* document (same `id`); the
consumer's SPEC §7.2 item-11 record absorbs it and returns the first outcome. A
**new attempt** is a different document — anything that changes the bytes,
including a re-stamped `issuedAt` or a re-signed `proof`, makes it one — and MUST
carry a fresh `id`. Use `NewAttempt(previous)` for that (it clears `proof` and
re-stamps `issuedAt`); reusing an `id` with altered content is `idConflict`.

## Governance management UIs

```go
list := cc.BuildListDocument(myDID, vtcDID)
thread := cc.CorrelationThread(list)
// ...on an inbound envelope body:
if reply, ok := cc.ParseEnvelopeReply(body, thread); ok && reply.Kind == cc.ReplyListing {
    render(reply.Entries) // []CapabilitySummary
}
```

Idempotent success and every classification are keyed on the SPEC §8.5 **extended
error code**, never on the non-normative free-text `message`.
