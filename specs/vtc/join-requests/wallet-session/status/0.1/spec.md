---
slug: vtc/join-requests/wallet-session/status
version: "0.1"
title: VTC Join-Requests — Wallet Session Status
summary: An applicant reads the state of a wallet presentation session they opened, and once a wallet has answered, whether the presentation satisfied the criterion, whose key held the credentials and which credentials they were — never their claim values.
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - vtc
  - join-requests
  - onboarding
  - oid4vp
  - wallet
  - presentation
  - status
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: applicant
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: The session belongs to the applicant who opened it, and the response tells them which wallet key answered it and why a presentation was refused. The document proof's signer is what the consumer compares against the session's applicant; without it, anyone holding a session id — which travels inside a QR code — could read the outcome.
sideEffects:
  level: none
  rationale: Reads the session's state and outcome; persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "The response describes the applicant's own evidence: whether it satisfied the query, the identifier of the key that held it, and each credential's query id, format, issuer and type. It carries no claim values. The request carries a session id the community minted."
retention:
  class: transient
  rationale: A poll reads the session's current state and is discarded. Nothing it returns is created by it — the session's outcome is held by the session, and the evidence by the join request it is recorded on.
errorCodes:
  - code: vtc/join-requests/wallet-session/status:notFound
    meaning: No session with the supplied `sessionId` exists, or it was not opened by the proof signer. The same code for both.
    retryable: false
related:
  - vtc/join-requests/wallet-session/start
  - vtc/join-requests/status
  - vtc/join-requests/submit
---

## Abstract

The **VTC Join-Requests — Wallet Session Status** Trust Task lets an applicant follow a presentation session they opened with [`vtc/join-requests/wallet-session/start`](../../start/0.1/spec.md). The wallet answering the session speaks OpenID4VP to the community and nothing to the applicant, so this poll is how the applicant learns that the QR code was scanned, that the wallet answered, and what the answer established.

Once a wallet has presented, the response says whether the presentation satisfies the criterion's query, identifies the key that held the credentials, and lists the credentials by query, format, issuer and type. It never returns a claim value: the applicant's own wallet already knows what it disclosed, and the community's copy of the claims stays with the join request it serves. When the community refused what the wallet sent, the response says why.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

### Producer (applicant)

1. **MUST** carry a proof signed by the DID that opened the session.
2. **SHOULD** poll no more often than every few seconds while the session is `awaitingWallet` or `requestRetrieved`, and **SHOULD** stop once it is `presented`, `refused` or `expired`: those states are final.

### Consumer (community)

1. **MUST** confirm the session exists and was opened by the proof signer, and refuse with `notFound` if either fails — the same code for both, so a party holding somebody else's session id, as anyone who saw their QR code does, learns nothing from it.
2. **MUST** return `sessionId`, `state` and `expiresAt`, and `requestId` when the session's evidence is recorded on, or bound to, a join request.
3. **MUST** return `evidence` when, and only when, `state` is `presented`, and `reason` when, and only when, `state` is `refused`.
4. **MUST NOT** return a claim value in any member, `ext` included. `evidence` describes which credentials were presented and by which key, never what they say.
5. **MUST** report `presented` only for a wallet response that verified as [`wallet-session/start`](../../start/0.1/spec.md) consumer rule 9 requires. A response that verified but does not satisfy the query is either recorded as evidence and reported `presented` with `satisfied: false`, or not recorded and reported `refused` with `queryNotSatisfied`; which is the community's policy, and a criterion that combines credentials with other requirements may make the first useful.
6. **MUST** report `holder` as the identifier of the key that held the presented credentials, and **MUST NOT** report the applicant's DID there unless that is in fact the holder key. The two are recorded separately because the presentation proves possession of the holder key, not identity with the applicant.
7. **SHOULD**, when evidence is held for a later submission rather than recorded on a request, return `heldUntil`, the deadline set by [`wallet-session/start`](../../start/0.1/spec.md) consumer rule 11.
8. **MAY** stop answering for a session once it has been closed for longer than its own policy keeps session records, and then answers `notFound`.

### States

| `state` | Meaning | Final |
|---|---|---|
| `awaitingWallet` | Open; no wallet has fetched the request object | no |
| `requestRetrieved` | A wallet has fetched the request object and not yet answered | no |
| `presented` | A wallet answered and the presentation verified; `evidence` describes it | yes |
| `refused` | A wallet answered and the community did not accept the answer, or the wallet declined; `reason` says which | yes |
| `expired` | The session closed at `expiresAt` unanswered | yes |

`requestRetrieved` exists so the applicant's screen can say "your wallet has the request" while its holder is deciding: the gap between scanning and answering is the wallet's consent screen, and can last as long as the person takes.

### Refusal reasons

| `reason` | Meaning |
|---|---|
| `presentationInvalid` | The response or a presentation in it failed verification: decryption, `state`, issuer signature, validity period, key binding, nonce or audience |
| `queryNotSatisfied` | The presentations verified but do not satisfy the criterion's query, and the community did not record them |
| `issuerNotTrusted` | A credential's issuer is not one the criterion's `credentialIssuers`, or the community's trust anchors for the wallet ecosystem, accept |
| `statusNotValid` | A credential's status is revoked or suspended, or its status could not be read — status is never assumed valid |
| `walletDeclined` | The wallet answered with an OpenID4VP error instead of a presentation, such as `access_denied` when its holder declined |

## Authorization

The authority this task assumes is **having opened the session**: being the applicant it is bound to. The `proof` does not confer that authority; it names the signer so the consumer can compare them against the session's applicant, and an unknown session and somebody else's are both `notFound` (consumer rule 1). Possession of the session id is deliberately not enough: the id travels in the `request_uri` inside a QR code, so it is held by anyone who saw the code, and the refusal detail and holder identifier this task returns are for the applicant alone.

## Definitions

**`sessionId`** — the session to read, as [`wallet-session/start`](../../start/0.1/spec.md) returned it.

**`state`**, **`expiresAt`** — the session's state, as tabled above, and when it closes or closed if unanswered.

**`requestId`** — the join request the evidence is recorded on: the one the session was bound to, or the one a later [`submit`](../../../submit/0.3/spec.md) attached held evidence to.

**`evidence`** — what a verified presentation established:

- **`satisfied`** — whether the presentation satisfies the criterion's DCQL query. Satisfying it is not admission; the community still decides under the criterion's `admission`.
- **`holder`** — the identifier of the key that held the presented credentials; for an SD-JWT VC, the `did:key` of the credential's `cnf` key. It is recorded beside the applicant DID and never as it.
- **`credentials`** — one entry per presented credential: `credentialQueryId` (the DCQL credential query it answered), `format` (OpenID4VP's identifier, verbatim), `issuer` (as the community verified it), and `vct` for an SD-JWT VC.
- **`heldUntil`** — when evidence not yet on a join request is discarded if the applicant does not submit under the session's criterion.

**`reason`** — why a session ended `refused`, as tabled above.

## Request

The applicant sends the request to the community, signed; the payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Polling a session

```json
{
  "id": "urn:uuid:2a7f5c18-9d3e-4b61-8c04-6e1b9f2d7a35",
  "type": "https://trusttasks.org/spec/vtc/join-requests/wallet-session/status/0.1#request",
  "issuer": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
  "recipient": "did:web:community.example",
  "issuedAt": "2026-10-11T09:02:30Z",
  "threadId": "urn:uuid:4e9b2d73-1f6c-4a85-b3e0-7c2a5d8f1b96",
  "payload": {
    "sessionId": "q7Xw2LmN9pRt4VbK8sYc3Hd6"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-10-11T09:02:30Z",
    "verificationMethod": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH#z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
    "proofPurpose": "authentication",
    "proofValue": "z2iVbXn8GmrWdPzKxHb4TtDCmjYSXuqQq3MkdGxJsLwuSGTb5k3KAeD7uxpEJ8WqSdFp9dVAz5HwNcVxYm3yTnbQR"
  }
}
```

## Response

The community answers with the session's state; the payload is the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). `evidence` appears only for `presented` and `reason` only for `refused`. A refusal to answer at all is a `trust-task-error` document carrying `notFound`, not a `#response`.

### Presented — evidence recorded on the deferred request

```json
{
  "id": "urn:uuid:6c3e8a21-5b7d-4f94-a1c6-9d2f0e4b8a57",
  "type": "https://trusttasks.org/spec/vtc/join-requests/wallet-session/status/0.1#response",
  "issuer": "did:web:community.example",
  "recipient": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
  "issuedAt": "2026-10-11T09:02:31Z",
  "threadId": "urn:uuid:4e9b2d73-1f6c-4a85-b3e0-7c2a5d8f1b96",
  "payload": {
    "sessionId": "q7Xw2LmN9pRt4VbK8sYc3Hd6",
    "state": "presented",
    "expiresAt": "2026-10-11T09:10:01Z",
    "requestId": "6f1d2c3b-9a4e-4b8f-a7d6-0e5c1b2a3f49",
    "evidence": {
      "satisfied": true,
      "holder": "did:key:zDnaerDaTF5BXEavCrfRZEk316dpbLsfPDZ3WJ5hRTPFU2169",
      "credentials": [
        {
          "credentialQueryId": "residence",
          "format": "dc+sd-jwt",
          "issuer": "did:webvh:QmIssuerScid:issuer.example.org",
          "vct": "https://example.org/credentials/residence-permit"
        }
      ]
    }
  }
}
```

### Refused — the wallet's holder declined

A different session, opened without a `requestId`; the person holding the wallet declined at its consent screen.

```json
{
  "id": "urn:uuid:8f1a4d62-3c9b-4e07-b5d8-2a6e1c9f4b73",
  "type": "https://trusttasks.org/spec/vtc/join-requests/wallet-session/status/0.1#response",
  "issuer": "did:web:community.example",
  "recipient": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
  "issuedAt": "2026-10-11T09:03:12Z",
  "threadId": "urn:uuid:0b5d7e29-6a1f-4c38-9e42-3f8c1a6d2b70",
  "payload": {
    "sessionId": "Rk4tZp8WcN2mXq6VhLs9Jd3B",
    "state": "refused",
    "expiresAt": "2026-10-11T09:12:40Z",
    "reason": "walletDeclined"
  }
}
```

## Security & Privacy

### Data carried

The request carries a session id the community minted. The response carries the session's state and, once a wallet has answered, a description of the applicant's own evidence: whether it satisfied the query, the identifier of the holder key, and for each credential its query id, format, issuer and type. It carries **no claim value**, in any member (consumer rule 4). The applicant's wallet already knows what it disclosed, and returning the claims here would put a second copy of a natural person's attributes on another channel for no purpose the applicant has.

`holder` is the member to treat with care. It identifies the wallet's key, which for a hardware-bound key may be stable across every verifier that wallet presents to; it is personal data about whoever holds the wallet, who is usually, though not necessarily, the applicant. `issuer` and `vct` together say what kind of credential the applicant holds — a residence permit from a named authority, say — which is itself informative, and is why this task answers only the applicant.

### Correlation

The applicant declares `identifierScope: pairwise`, and the community `public`, for the reasons [`wallet-session/start`](../../start/0.1/spec.md) gives: the applicant's identifier need last only as long as this application, and the applicant must be able to recognise the community answering the poll as the one it opened the session with — a pairwise community identifier would leave it unable to confirm that the party reporting its evidence is the party that took it. The session id is the handle polls are joined by, and it is also in the `request_uri` the wallet fetched, so the community can join the applicant's polls to the wallet's requests — which is the session's purpose, not a leak.

The `notFound` collapse (consumer rule 1) is what keeps the session id from being an oracle. Anyone who saw the QR code holds the id; without the proof signer's comparison they could read whether the session was answered, by which key, and why a presentation failed. With it, they learn nothing beyond what any unknown id would tell them.

The refusal detail is returned only to the applicant who began the session. It tells them which check failed — that a credential's status was not valid, or its issuer was not trusted — which is information about their own credential and the community's policy, disclosed to the party with the strongest claim to it. A community uncomfortable with that granularity **MAY** report `presentationInvalid` for any verification failure.

Polling cadence is visible to the community, and to an observer of the transport, as a timing signal: it shows roughly when the applicant was waiting for their wallet. A producer can vary it, and producer rule 2 keeps it bounded.

### Retention

Transient. A poll reads the session and is discarded. The session's outcome lives as long as the community keeps the session record ([`wallet-session/start`](../../start/0.1/spec.md) *Retention*), and the evidence as long as the join request it is recorded on; this task extends neither. An applicant's client **MAY** keep the final response as its own record of what the community accepted or refused and when.

This task does not require `issuedAt`: it changes nothing, so it is not consequential, and the framework's freshness floor does not reach it. A consumer that wants to bound replay of a captured poll — whose response, on a transport that answers on the request's own channel, goes to whoever sent it — **MAY** apply its own acceptance window to `issuedAt` where present, or to `expiresAt`; the proof requirement already confines what a replay can read to the session of the applicant who signed it.

### Consent/purpose

The purpose is the applicant learning the outcome of an exchange they began, so they know whether to submit, to try again with another credential, or to stop. The description of the evidence is the minimum that serves it: enough to tell which credential counted and whose key presented it, and nothing of what the credential says. Whether the community notifies the applicant instead of letting them poll, and what it does with the claims it received, are its own policy and outside this specification.
