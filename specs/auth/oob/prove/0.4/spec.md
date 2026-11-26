---
slug: auth/oob/prove
version: "0.4"
title: "Auth — OOB Prove"
summary: "The lock holder proves membership (for deviceEnrol, administration of the VTA) and the number on the starter's screen with a signed auth/oob/identify, and gets back the service-signed step 2: where the request came from, the starter key, and what it would be allowed."
status: draft
targetFrameworkVersion: "0.6.0"
category: authentication
keywords: [auth, oob, out-of-band, sign-in, login, membership, wallet, delegation, session-key]
parties:
  - role: Approver (lock key)
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: Service
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  request: REQUIRED
  response: REQUIRED
  rationale: >-
    The request's proof by the lock key is what ties the carried identify to the device holding the lock, so another device cannot re-wrap it. The response's proof, by the service's assertionMethod key, makes step 2 an attestation: the wallet shows its contents as the community's statement, and the grant binds its digest.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task changes request state, so it is consequential and SPEC §7.3 item 17 sets REQUIRED as the floor.
sideEffects:
  level: mutating
  rationale: >-
    Moves the request from claimed to identified and records the identified DID, or to declined on any failure. Each request allows one proof attempt.
exposure:
  discloses: metadata
  ingests: personal
  actsAsSubject: false
  rationale: >-
    Ingests the member's chosen DID (inside identify). Discloses, to the proven lock holder only, the starter's coarse location (city and country), browser family, OS, creation time, a same-network signal, the starter key and the task families the service offers that key — or, for deviceEnrol, the client's self-asserted kind and name and the scope it asks for — never an IP address.
retention:
  class: durable
  rationale: >-
    The service audits every proof, including failed and peeking ones, with city and country, because step 2 is the moment the starter's location was shown to someone. It keeps the digest of the signed step 2 for the life of the request, to check the grant against.
errorCodes:
  - code: auth/oob/prove:numberMismatch
    meaning: The typed number does not match. The request is declined.
    retryable: false
  - code: auth/oob:notClaimant
    meaning: The prove is not signed by the request's lock key.
    retryable: false
  - code: auth/oob:requestExpired
    meaning: The decision window has passed, or the request has otherwise ended.
    retryable: false
  - code: auth/oob:notAuthorized
    meaning: The carried identify failed verification, does not match the request or the lock, or its issuer is not an active member — for deviceEnrol, not an administrator of the VTA. Deliberately generic. The request is declined.
    retryable: false
  - code: auth/oob:rateLimited
    meaning: The approver's network has hit the service's prove limit. `details.retryAfter` MAY give seconds until a retry.
    retryable: true
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        retryAfter: { type: integer, minimum: 0 }
related:
  - auth/oob/claim
  - auth/oob/identify
  - auth/oob/respond
  - auth/oob/grant
---

## Abstract

After the claim, the member picks an identity and the number shown on the starter's screen is entered — typed by the member, or read from that page by a same-device reader ([`auth/oob/identify`](../../identify/0.2/spec.md) item 1). The wallet has the member's VTA sign an [`auth/oob/identify`](../../identify/0.2/spec.md) and sends it inside this task, signed by the lock key. If the member is a member and the number matches, the service answers with **step 2**: the step 1 members unchanged, plus the starter key (`sessionKey`), where the request came from (`requester`), which DID was identified (`identifiedAs`) and — since 0.2 — which task families the service offers to let the starter key sign for (`signingScope`). The wallet shows this before the member approves or declines.

For `deviceEnrol` the service is the person's VTA, the identity proved must be one of its administrators, and step 2 names the client asking to be enrolled (`client`) and the most it asks for (`requestedScope`) in place of a signing scope.

### Changes from 0.1

- **Step 2 may carry `signingScope`.** A service whose sign-in surface needs to author signed Trust Task documents — a member portal reading and editing a directory — offers here the task families the starter key would sign for, as the approving DID, for the session the grant arms. Step 2 is the service's signed statement and the grant binds its digest, so the offer the member is shown is the offer the grant accepts: [`auth/oob/grant/0.2`](../../grant/0.2/spec.md) copies it unchanged and states what the key may and may not sign. A step 2 without `signingScope` is a 0.1 step 2 in all but its type, and a grant over it delegates no signing.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Changes from 0.3

- Step 2's **`purpose`** may be `deviceEnrol`
  ([`oob.schema.json` 0.2](../../_shared/0.2/oob.schema.json),
  [`CONVENTIONS.md`](../../_shared/0.2/CONVENTIONS.md) §3).
- For `deviceEnrol`, step 2 carries two more members, REQUIRED for that purpose
  and absent for `login`: **`client`**, the client's self-asserted kind and name,
  and **`requestedScope`**, the ceiling it asks for, both as
  [`auth/oob/request/0.3`](../../request/0.3/spec.md) carried them. It never
  carries `signingScope`.
- For `deviceEnrol`, the identify issuer must be an administrator of the VTA,
  not a member of a community (service item 3), and the entered number has
  four digits (A12).
- The carried identify is [`auth/oob/identify/0.2`](../../identify/0.2/spec.md).
- The grant this step 2 is offered towards is
  [`auth/oob/grant/0.4`](../../grant/0.4/spec.md).

A `login` step 2 is the same document as in 0.3, apart from its `type`, and is
produced and checked the same way. The request payload is unchanged. This
version cannot declare `extends: "0.3"`
([`CONVENTIONS.md`](../../_shared/0.2/CONVENTIONS.md) §7 A10). `0.3` remains
published.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **approver**:

1. **MUST** sign with the lock key `K_a` and **SHOULD** set `parentThreadId` to the `requestId` ([SPEC §4.9.2](/SPEC.md#492-the-parentthreadid-member)).
2. **MUST** check that the identify its VTA returned is the one it sent, apart from the added proof, before carrying it.
3. **MUST** verify the response's proof against a key the service DID lists under `assertionMethod`, and **MUST** check that it repeats the step 1 members unchanged and that `identifiedAs` is the DID it chose.
4. **SHOULD**, when step 2 carries `signingScope`, show the member the families named in it before they decide, in words that say the browser that started the sign-in will be able to act as them for those tasks until the session ends.
5. For `deviceEnrol`, **MUST** show the person, before they decide: that a program will get lasting access to their VTA; the client's `displayName` and `consumerKind`, marked as what the client says about itself; the `requestedScope`; and the `requester` signals, with a warning when `sameNetwork` is not `true`. It **MUST** let the person narrow the scope and choose the context, and **SHOULD** propose the narrowest scope by default ([`CONVENTIONS.md`](../../_shared/0.2/CONVENTIONS.md) §7 A1, A3, A8). Where `requestedScope.purposes` is `any`, it **MUST** show it to the person as any purpose, not as a list (CONVENTIONS §7 A14). The `sameNetwork` warning is mandatory; for a client the person says runs remotely, it **MAY** show the signal as expected rather than as an alarm (CONVENTIONS §7, open questions).

A conforming **service**, in this order:

1. **MUST** require the outer issuer to equal the lock, else `auth/oob:notClaimant`, and the request to be `claimed` and inside its decision window, else `auth/oob:requestExpired`.
2. **MUST** require the carried identify to name this request and the lock as `approverKey`.
3. **MUST** check that the identify issuer string is an active member **before** resolving any DID. For `deviceEnrol`, the check is instead that it is an administrator: the subject of a live ACL entry at this VTA from which it could write an entry by [`acl/grant/0.3`](../../../../acl/grant/0.3/spec.md) in at least one context, under the bounds of [ACL CONVENTIONS §9](../../../../acl/_shared/0.3/CONVENTIONS.md#9-bounds-on-the-granter-and-delegation) ([`CONVENTIONS.md`](../../_shared/0.2/CONVENTIONS.md) §7 A7); anyone else gets `auth/oob:notAuthorized`, as a non-member does.
4. **MUST** verify the identify as [`auth/oob/identify/0.2`](../../identify/0.2/spec.md) requires, against the issuer's `authentication` relationship.
5. **MUST** require `enteredNumber` to equal the match number, else `auth/oob/prove:numberMismatch`.
6. On success, **MUST** move the request to `identified` in one compare-and-set, record the DID, set `sameNetwork` by comparing this call's egress network with the starter's, answer with step 2 signed by its `assertionMethod` key, and store the digest of that signed response. For `deviceEnrol`, step 2 **MUST** carry the stored `client` and `requestedScope` exactly as the request carried them.
7. On any failure from step 2 on, **MUST** move the request to `declined`. A non-member and a bad signature **MUST** both get `auth/oob:notAuthorized`.
8. **MAY** include `signingScope` in step 2, naming the narrowest set of task families the surface at `origin` needs to sign. It **MUST NOT** name a family it would not accept a starter-key document for under [`auth/oob/grant/0.4`](../../grant/0.4/spec.md), and **MUST NOT** name `auth`, `acl`, `consent`, `task-consent` or `confirm`, or any family under one of them: those families exist to establish identity, confer authority or record a decision, which a starter key never may. Which families it offers **MAY** depend on the surface the sign-in is for; it **MUST** be fixed by the service, never taken from the starter or the approver. For `deviceEnrol` it **MUST NOT** include `signingScope`.
9. **MUST** answer an earlier version of this task — `auth/oob/prove/0.3` or before — for a request opened as `deviceEnrol` with `auth/oob:notAuthorized`, and move the request to `declined`: such a request is handled only by the versions that define its purpose ([`CONVENTIONS.md`](../../_shared/0.2/CONVENTIONS.md) §5 rule 3c).

## Authorization

The lock entitles its holder to attempt one proof. Membership, read from the service's own member list, and the matching number are what entitle the holder to see step 2. Neither the outer nor the inner proof is authorization by itself ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10): the outer one identifies the lock holder, the inner one the DID.

## Definitions

- **`identify`** — a complete signed `auth/oob/identify/0.2` document, verified over its members as received.
- **Step 2** (response) — step 1's `requestId`, `service`, `origin`, `purpose` and `decisionClosesAt`, plus:
  - **`sessionKey`** — the starter key `K_b`;
  - **`requester`** — `location` (city and country, or `unknown`), `browser` (family), `os`, `createdAt`, and `sameNetwork` (`true`, `false` or `"unknown"`);
  - **`identifiedAs`** — the DID that issued the identify;
  - **`signingScope`** (optional, `login` only) — the task families, as slug prefixes, that the service offers to let the starter key sign for as `identifiedAs`, for the session a grant would arm;
  - **`client`** (`deviceEnrol` only) — the client's self-asserted `consumerKind` and `displayName`;
  - **`requestedScope`** (`deviceEnrol` only) — the ceiling the client asks to be enrolled under.

## Request

The approver (`issuer`, the lock key) sends the carried identify to the service (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### A wallet proves membership and the number

```json
{
  "id": "urn:uuid:7c3d4e5f-6071-4b8c-9d0e-1f2a3b4c5d01",
  "type": "https://trusttasks.org/spec/auth/oob/prove/0.4",
  "parentThreadId": "q3Vt7mXo9LwA2bR5cJ8kNg",
  "issuer": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "recipient": "did:web:community.example",
  "issuedAt": "2026-10-10T10:01:01Z",
  "payload": {
    "identify": {
      "id": "urn:uuid:6b2c3d4e-5f60-4a7b-8c9d-0e1f2a3b4c01",
      "type": "https://trusttasks.org/spec/auth/oob/identify/0.2",
      "issuer": "did:web:alice.example",
      "recipient": "did:web:community.example",
      "issuedAt": "2026-10-10T10:01:00Z",
      "payload": {
        "requestId": "q3Vt7mXo9LwA2bR5cJ8kNg",
        "approverKey": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
        "enteredNumber": "47"
      },
      "proof": {
        "type": "DataIntegrityProof",
        "cryptosuite": "eddsa-jcs-2022",
        "verificationMethod": "did:web:alice.example#key-1",
        "created": "2026-10-10T10:01:00Z",
        "proofPurpose": "authentication",
        "proofValue": "z3kg…"
      }
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK#z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "created": "2026-10-10T10:01:01Z",
    "proofPurpose": "authentication",
    "proofValue": "z3kg…"
  }
}
```

## Response

The service answers with step 2, signed by its `assertionMethod` key. The payload is the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document.

### Step 2, offering the member portal's signing scope

```json
{
  "id": "urn:uuid:7c3d4e5f-6071-4b8c-9d0e-1f2a3b4c5d02",
  "type": "https://trusttasks.org/spec/auth/oob/prove/0.4#response",
  "threadId": "urn:uuid:7c3d4e5f-6071-4b8c-9d0e-1f2a3b4c5d01",
  "parentThreadId": "q3Vt7mXo9LwA2bR5cJ8kNg",
  "issuer": "did:web:community.example",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-10-10T10:01:01Z",
  "payload": {
    "requestId": "q3Vt7mXo9LwA2bR5cJ8kNg",
    "service": { "did": "did:web:community.example", "name": "Example Community" },
    "origin": "https://members.community.example",
    "purpose": "login",
    "decisionClosesAt": "2026-10-10T10:02:40Z",
    "sessionKey": "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp",
    "requester": {
      "location": "Sydney, Australia",
      "browser": "Chrome",
      "os": "macOS",
      "createdAt": "2026-10-10T10:00:00Z",
      "sameNetwork": true
    },
    "identifiedAs": "did:web:alice.example",
    "signingScope": ["vtc/directory", "vtc/groups"]
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:community.example#key-1",
    "created": "2026-10-10T10:01:01Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3kg…"
  }
}
```

### Step 2 for an AI agent asking to be enrolled

The service is Alice's VTA; Alice's wallet proved her administrator DID and the number the agent printed.

```json
{
  "id": "urn:uuid:7c3d4e5f-6071-4b8c-9d0e-1f2a3b4c5d12",
  "type": "https://trusttasks.org/spec/auth/oob/prove/0.4#response",
  "threadId": "urn:uuid:7c3d4e5f-6071-4b8c-9d0e-1f2a3b4c5d11",
  "parentThreadId": "Hk2Lm9Qw4Rt7Yu1Io3Pa5S",
  "issuer": "did:web:vta.alice.example",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-10-10T10:01:01Z",
  "payload": {
    "requestId": "Hk2Lm9Qw4Rt7Yu1Io3Pa5S",
    "service": { "did": "did:web:vta.alice.example", "name": "Alice's VTA" },
    "origin": "https://vta.alice.example",
    "purpose": "deviceEnrol",
    "decisionClosesAt": "2026-10-10T10:02:40Z",
    "sessionKey": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
    "requester": {
      "location": "Sydney, Australia",
      "browser": "unknown",
      "os": "Linux",
      "createdAt": "2026-10-10T10:00:00Z",
      "sameNetwork": true
    },
    "identifiedAs": "did:web:alice.example",
    "client": {
      "consumerKind": { "kind": "service", "serviceKind": "aiAgent" },
      "displayName": "Research agent on laptop"
    },
    "requestedScope": {
      "capabilities": [
        { "capability": "memory.read" },
        { "capability": "memory.write" }
      ],
      "purposes": { "scope": "listed", "purposes": ["taskExecution"] },
      "keys": { "scope": "none" },
      "maxLifetimeDays": 90
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:vta.alice.example#key-1",
    "created": "2026-10-10T10:01:01Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3kg…"
  }
}
```

### The number did not match

```json
{
  "id": "urn:uuid:7c3d4e5f-6071-4b8c-9d0e-1f2a3b4c5d03",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "threadId": "urn:uuid:7c3d4e5f-6071-4b8c-9d0e-1f2a3b4c5d01",
  "parentThreadId": "q3Vt7mXo9LwA2bR5cJ8kNg",
  "issuer": "did:web:community.example",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-10-10T10:01:01Z",
  "payload": {
    "code": "auth/oob/prove:numberMismatch",
    "message": "The number does not match. Refresh the code on the website and scan again.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

In: the member's chosen DID, inside identify. Out: the starter's city and country (or `unknown`), browser family, OS, creation time, a same-network signal, the starter key, and the task families offered to it — or, for `deviceEnrol`, the client's kind and name and the scope it asks for. A client that is not a browser usually reports `unknown` browser and OS. Never the starter's IP address or full User-Agent. Step 2 is revealed only to the lock holder, after a membership proof and a number proof.

### Correlation

The service learns which DID approved which request, which is the purpose of the task. The lock key remains fresh per request. The service has identifier scope `public` because the approver must reach and verify the community's published DID; the approver's lock key is pairwise.

### Retention

The service keeps the identified DID and the digest of the signed step 2 for the life of the request, and audits each proof — including a failed one, and one that saw step 2 and went no further — with city and country, never the address.

### Consent/purpose

The starter's coarse location and browser are shown to the member so they can tell whether the request is theirs. They are not to be used for anything else. The location is a heuristic: a relaying attacker controls where the request appears to come from.

`signingScope` is shown so that the member decides, knowingly, what the browser will be able to sign in their name. It is an offer and describes the delegation a grant would make; the limits that hold whatever it says are those of [`auth/oob/grant/0.4`](../../grant/0.4/spec.md).

For `deviceEnrol`, `client` and `requestedScope` are shown so that the person decides, knowingly, which program gets lasting access to their VTA and how much. The client's name and kind are its own claim, and a relaying attacker can set them; the wallet's own record of the VTAs it administers and the `requester` signals are what the person can check.

## Pending confirmation

This version takes the readings in [`CONVENTIONS.md`](../../_shared/0.2/CONVENTIONS.md) §7 and will follow the family authors' answer when they give one. Those that bear on this task: **A1**, **A3**, **A6**, **A7**, **A8**, **A10** and **A14**.
