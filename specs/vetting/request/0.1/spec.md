---
slug: vetting/request
version: "0.1"
title: Vetting — Request
summary: An applicant asks one existing member to vet their identity for one community. The vetter accepts and shows it is eligible to vet, or refuses.
status: draft
targetFrameworkVersion: "0.5"
category: identity
keywords:
  - vetting
  - identity-vetting
  - onboarding
  - vtc
  - ticket
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: applicant
    requirement: REQUIRED
    member: issuer
    identifierScope: public
  - role: vetter
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: The request is where an applicant's join DID first reaches a vetter, and `joinDid` must equal the document's issuer; only a proof makes that equality hold on every transport, relayed ones included. The response carries the vetter's claim to be eligible to vet for a community, which the applicant relies on before arranging a session and which must be attributable to the vetter DID it names.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: A request redeems a ticket and opens state at the vetter. Replayed, it would spend another use of a ticket or re-open a request the applicant had abandoned; placing it in a window is what lets a vetter recognise the replay.
sideEffects:
  level: mutating
  rationale: "The vetter records an open request and, where a ticket is presented, counts one use of it."
exposure:
  discloses: metadata
  ingests: personal
  actsAsSubject: false
  rationale: "The request carries the applicant's join DID, language preferences, optional free text the applicant wrote (`message`, `availability`), and optionally an invitation credential naming them — personal data about an identifiable applicant, disclosed to one vetter they chose. The response discloses facts about the vetter the applicant needs and would learn at the session anyway: a presentation of the vetter's community membership and role, bound to this request, and what documentation the vetter accepts."
retention:
  class: exchange
  rationale: The vetter holds the request for the life of the vetting — until a statement is issued, a decline is sent, or the request lapses under the vetter's own policy — and needs nothing from it afterwards beyond counters for rate limiting. The applicant keeps the outcome of checking the vetter's eligibility, not the presentation.
errorCodes:
  - code: vetting/request:invalidTicket
    meaning: A scanned ticket's secret does not match an active ticket this vetter issued for the named community, or the ticket is used up or expired. Never returned for a short code — a wrong short code gets no reply at all.
    retryable: false
  - code: vetting/request:capacity
    meaning: The vetter is not taking new requests at the moment.
    retryable: true
  - code: vetting/request:notEligible
    meaning: The addressee is not currently a vetter for the named community.
    retryable: false
  - code: vetting/request:declined
    meaning: The vetter will not take this request. No reason is given, and none is owed.
    retryable: false
  - code: vetting/request:methodUnavailable
    meaning: The vetter does not offer the preferred method.
    retryable: false
related:
  - vetting/session
  - vetting/decline
  - vtc/join-requests/manifest
  - credential-exchange/issue
---

## Abstract

A community that admits people on **peer identity vetting** asks its applicants to be vetted by existing members before it decides. Each vetter checks, in person or on a call, that the applicant is who they claim to be and controls the DID they are applying with, and issues a signed Vetting Statement. This task is the first step: the applicant asks one vetter to vet them for one community.

The vetter either accepts — returning a `requestId` and, ideally, proof that it is currently eligible to vet for that community — or refuses with a `trust-task-error`. An accepted request leads to [`vetting/session`](../../session/0.1/spec.md), in which the vetter checks the applicant and then delivers a statement over [`credential-exchange/issue`](../../../credential-exchange/issue/0.1/spec.md) or declines with [`vetting/decline`](../../decline/0.1/spec.md). The applicant repeats this with as many vetters as the community's [manifest](../../../vtc/join-requests/manifest/0.2/spec.md) requires.

These tasks run between two people's agents. No community service is a party to them; the community sees only the statements the applicant eventually submits.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5 and may change in place while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming **applicant** (`issuer`):

1. **MUST** set `joinDid` equal to the document's `issuer`, and **MUST** use as `joinDid` the DID it will be vetted under and later submit its statements with. One join DID serves one application, with every vetter of it.
2. **SHOULD** set `requirementsDigest` to the digest of the manifest criterion it is gathering for, as recorded when the application started.
3. **MUST NOT** carry both `ticket` and `introduction`, and **SHOULD** carry one of them unless it knows the vetter takes requests without either.
4. **MAY** upper-case a short code a person typed, and **MUST NOT** otherwise rewrite it — a code that fails the pattern was mistyped, and silently mapping `O` to `0` turns a typing error into a guess.
5. **MUST NOT** put a document number, a document image, or any other detail of identity documentation in `message`, `availability`, or `ext`.

A conforming **vetter** (`recipient`):

1. Applies the [SPEC §7.2](/SPEC.md#72-consumer-requirements) pipeline, and refuses a request whose `joinDid` differs from its `issuer`, or that carries both a ticket and an introduction, with `malformedRequest`.
2. **MUST** decide whether the request passes its own gate — a valid ticket it issued for `community`, an introduction it accepts, or a policy of taking requests without either — **before** anything about the request reaches a person. A request that fails the gate **MUST NOT** produce a notification.
3. **MUST NOT** answer a request whose short `code` matches no active ticket — not with a response, and not with a `trust-task-error`. A scanned ticket whose `secret` does not match is refused with `vetting/request:invalidTicket`.
4. **MUST** compare ticket codes and secrets in constant time, count one use of a ticket per accepted request, and treat the first `issuer` to redeem a ticket as the only party later documents of that vetting may come from.
5. On accepting, returns the `#response` with a `requestId` unique among its open requests. It **SHOULD** include `eligibilityVp`: a presentation, held by the vetter's DID (the response's `issuer`), of the membership credential and the role credential the community issued it, with `challenge` equal to `requestId` and `domain` equal to `community`. It **SHOULD** include `acceptsDocumentation`.
6. Otherwise returns a `trust-task-error` carrying one of this specification's codes. A refusal is never a `#response`.

A conforming **applicant**, on receiving a response carrying `eligibilityVp`, **SHOULD** check that its holder is the response's `issuer`; that its `challenge` and `domain` are `requestId` and `community`; that both credentials are issued by `community` and name the vetter; that the role credential names the role the community's manifest requires; and that neither is expired or revoked. The check is advisory: the community evaluates eligibility again when it decides, and a statement from a vetter that was not eligible does not count.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

What a request needs is authority to be **heard**, not authority to be vetted. It comes from the vetter: possession of a ticket the vetter issued, an introduction the vetter accepts, or the vetter's own policy of taking requests without either. A ticket is a bearer secret. Holding one says nothing about who the applicant is — that is the proof and the `joinDid` equality — and entitles the holder to nothing beyond having the request considered. Whether to vet anyone remains the vetter's decision, and `vetting/request:declined` needs no reason.

What the response asserts is the vetter's eligibility, and that authority is the **community's**: a membership credential and a role credential the community issued to the vetter. `eligibilityVp` presents that evidence; it does not create it. A vetter that accepts without being eligible has done nothing a conforming applicant need rely on, because its statements will not count.

Per [SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10, verifying either document's `proof` establishes who sent it, never that they may do what it asks.

## Definitions

**Applicant** — the person who wants to join a community, acting through their agent.

**Vetter** — an existing member the community has made eligible to vet, acting through their agent.

**Join DID** — the DID the applicant applies with. Every card, every statement and the final submission name it; that is what ties an application's evidence together.

**Ticket** — something a vetter hands an applicant out of band so that a request reaches them. The **short code** is eight Crockford base32 characters (`XXXX-XXXX`, 40 bits) that a person reads or types; the **scanned** form is a `ticketId` plus a 32-byte `secret`, usually from a QR code.

**Introduction** — an invitation credential for the community, naming the applicant's join DID, issued by the community or a member. A vetter may accept one in place of a ticket.

**Gate** — the vetter's decision, before any person is involved, whether a request is one it will consider at all.

## Request

The applicant sends the request to the vetter. See the top-level schema in [`payload.schema.json`](payload.schema.json).

### A ticket read aloud at a conference

```json
{
  "id": "urn:uuid:6f1c2b0a-3d4e-4f5a-8b6c-7d8e9f0a1b01",
  "type": "https://trusttasks.org/spec/vetting/request/0.1",
  "threadId": "urn:uuid:6f1c2b0a-3d4e-4f5a-8b6c-7d8e9f0a1b01",
  "issuer": "did:webvh:QmAliceScid1:alice.example",
  "recipient": "did:webvh:QmCarolScid1:kernel-vtc.example:carol",
  "issuedAt": "2026-09-14T09:00:00Z",
  "payload": {
    "community": "did:webvh:QmVtcScid:kernel-vtc.example",
    "requirementsDigest": "zQmfJG5HsnXfn2PtAh8hhcbHL41fKjN5pPmoHtnb2uUPSGs",
    "joinDid": "did:webvh:QmAliceScid1:alice.example",
    "ticket": { "code": "K7QF-2M9X" },
    "preferredMethod": "video",
    "languages": ["en", "de"],
    "message": "Hi Carol — we worked on the mm reclaim series together in 2025.",
    "availability": "Weekdays 14:00–18:00 UTC"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmAliceScid1:alice.example#key-1",
    "created": "2026-09-14T09:00:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z2RA8945kouBqzqifZqkbB8ZSrj1sfVLZPvr6wz4RvHSaYqXySHQoep9vM1fRYit6tNfmaTDThA2ibMPhBMFh8w3N"
  }
}
```

## Response

The vetter, now responding, accepts the request, per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). `requestId` is what the session and any decline name. Refusals are `trust-task-error` documents, never a `#response`.

### Accepted, with proof of eligibility

The presentation is abridged; its credentials are the community's membership credential and `vetter` role credential for Carol.

```json
{
  "id": "urn:uuid:6f1c2b0a-3d4e-4f5a-8b6c-7d8e9f0a1b02",
  "type": "https://trusttasks.org/spec/vetting/request/0.1#response",
  "threadId": "urn:uuid:6f1c2b0a-3d4e-4f5a-8b6c-7d8e9f0a1b01",
  "issuer": "did:webvh:QmCarolScid1:kernel-vtc.example:carol",
  "recipient": "did:webvh:QmAliceScid1:alice.example",
  "issuedAt": "2026-09-14T09:05:00Z",
  "payload": {
    "requestId": "urn:uuid:4b2e8f10-7a6c-4d3b-9e21-0f5a6b7c8d01",
    "eligibilityVp": {
      "@context": ["https://www.w3.org/ns/credentials/v2"],
      "type": ["VerifiablePresentation"],
      "holder": "did:webvh:QmCarolScid1:kernel-vtc.example:carol",
      "verifiableCredential": [
        {
          "type": ["VerifiableCredential", "DTGCredential", "MembershipCredential"],
          "issuer": "did:webvh:QmVtcScid:kernel-vtc.example",
          "credentialSubject": { "id": "did:webvh:QmCarolScid1:kernel-vtc.example:carol" }
        },
        {
          "type": ["VerifiableCredential", "DTGCredential", "EndorsementCredential"],
          "issuer": "did:webvh:QmVtcScid:kernel-vtc.example",
          "credentialSubject": {
            "id": "did:webvh:QmCarolScid1:kernel-vtc.example:carol",
            "endorsement": { "type": "CommunityRole", "role": "vetter" }
          }
        }
      ],
      "proof": {
        "type": "DataIntegrityProof",
        "cryptosuite": "eddsa-jcs-2022",
        "verificationMethod": "did:webvh:QmCarolScid1:kernel-vtc.example:carol#key-1",
        "proofPurpose": "authentication",
        "challenge": "urn:uuid:4b2e8f10-7a6c-4d3b-9e21-0f5a6b7c8d01",
        "domain": "did:webvh:QmVtcScid:kernel-vtc.example",
        "proofValue": "z5k2pxtz3XrdADnsNJ1XQiZ4oj7XHVqTamscwe3Wir6JjjNKp6mJZZTmynBW32NBmWCxjs5g8Xmjck9ZLfPKtU5G4"
      }
    },
    "acceptsDocumentation": ["passport", "national-id", "none"],
    "sessionHint": "Video, Thursday 17 September at 15:00 UTC. I will email you a link."
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmCarolScid1:kernel-vtc.example:carol#key-1",
    "created": "2026-09-14T09:05:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z63jiSzsVJshBfyZwcr6nUopHo5M1QnBnWJHtwTpdNEFeD7KoX5rezJcGeoY8AVuTSo5Q3uH2KqMoEZk68qqGu3AR"
  }
}
```

### Refused: the vetter is at capacity

```json
{
  "id": "urn:uuid:6f1c2b0a-3d4e-4f5a-8b6c-7d8e9f0a1b03",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "threadId": "urn:uuid:6f1c2b0a-3d4e-4f5a-8b6c-7d8e9f0a1b01",
  "issuer": "did:webvh:QmCarolScid1:kernel-vtc.example:carol",
  "recipient": "did:webvh:QmAliceScid1:alice.example",
  "issuedAt": "2026-09-14T09:05:00Z",
  "payload": {
    "code": "vetting/request:capacity",
    "retryable": true
  }
}
```

## Security & Privacy

**A wrong short code gets no reply.** A short code carries 40 bits, and the only thing that makes guessing one impractical is that a guesser learns nothing from a wrong guess. A vetter therefore sends nothing at all — no `trust-task-error` — for a request whose short code matches no active ticket (Conformance item 3). It **SHOULD** stop considering requests from a sender after five failed short codes in an hour, and **SHOULD** stop accepting short codes altogether, while still accepting scanned tickets, when failures spike across senders. A scanned ticket's 256-bit secret is not guessable, so its failure is reported as `invalidTicket`. Silence is not a refusal an applicant can rely on ([SPEC §4.12](/SPEC.md#412-document-lifecycle)); an applicant whose request goes unanswered should check the code with the vetter out of band.

### Data carried

The smallest request that works is `community`, `joinDid` and whatever gets it past the vetter's gate. Everything else is the applicant's choice. `message` and `availability` are free text the applicant wrote. They are bounded, read by the vetter alone, and **MUST** be attributed to the applicant wherever they are shown, because they are the one part of the request whose truth nobody has checked. An `introduction` is a credential naming the applicant and the member who introduced them, so carrying one tells the vetter who that member is.

A ticket secret or short code is a credential. The request **MUST** travel over a channel confidential to the two parties, as DIDComm authcrypt and TSP both are.

Nothing about identity documents belongs in this exchange — not a number, not a scan, not "passport, expires 2031". The vetter looks at the document during the session, and it never goes on the wire.

The response discloses facts about the vetter: that they are a member of the community and hold its `vetter` role, and what documentation they will rely on. The applicant needs these to decide whether the vetter is worth approaching, and would learn them at the session anyway.

### Correlation

Both parties declare `identifierScope: public`, and both do so because the application needs an identifier recognisable across a bounded set of relationships.

The join DID is shown to every vetter of the application and, at submission, to the community. That is its purpose: the evidence of several vetters is useful only if every statement names the same subject, and a pairwise identifier per vetter would leave the community nothing to count. The cost is that the vetters of one application can recognise that they vetted the same DID. An applicant **SHOULD** use a join DID for one application to one community only; reusing it elsewhere hands every vetter and community a join key into the others.

The vetter's DID is its member DID in the community, recognisable by every member. Statements must be attributable to a member for the community's accountability rules to work, so a pairwise vetter DID would make them unattributable. What the applicant learns — that this DID is a vetter in that community — is something they would learn at the session.

A redeemed ticket binds to the first DID that used it, and a request tells the vetter who is applying to which community, and when. Neither is avoidable. A vetter holding many requests holds a partial list of a community's applicants, which is a reason to keep less (below).

### Retention

The vetter holds the request while the vetting is live and **SHOULD** delete `message`, `availability` and any `introduction` once a statement is issued, a decline is sent, or the request lapses. After that, what remains useful is a count for rate limiting and, if the vetter wants one, a note that it vetted this join DID for this community. Requests that fail the gate **SHOULD** be counted, not recorded individually.

The applicant keeps the fact of acceptance and the result of checking the vetter's eligibility. It has no need to keep the presentation itself.

### Consent/purpose

The applicant discloses the request to arrange identity vetting for one application to one community, and the vetter's use of it is limited to that. Forwarding requests to the community, building a list of who is applying where, or reusing a `message` for anything else is a purpose the applicant did not address. This task gives the community no view of requests, refusals or silence. Whether an agent asks its user before sending a request, or before a vetter's agent accepts one, is that agent's policy; per [SPEC §7.3](/SPEC.md#73-specification-requirements) item 13 this specification takes no position on it.
