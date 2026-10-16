# Witnessed party — conventions for the vta/witness 0.2 tasks

**Witnessing** is a third party observing a relationship exchange and attesting, in a Verifiable Witness Credential (VWC), that it took place. The participating party opens a session with the witness under its relationship DID ([`witness/session/0.2`](../../../witness/session/0.2/spec.md)), presents its relationship credential bound to the session's challenge ([`witness/session/submit/0.2`](../../../witness/session/submit/0.2/spec.md)), and receives the VWC in the witness's response.

These tasks let a VTA be that party for its principal **without speaking to the witness**. The VTA holds the relationship DID's keys and the credentials; it builds and signs every document the party sends, and verifies and stores what the witness returns. The **client** — the principal's wallet or agent — carries the documents: it sends each one to the witness from the relationship DID and hands the witness's answer back.

The client carries them because the witness answers the relationship DID. A witness checks that the transport-proven sender of each document is its `issuer`, and replies to that sender. The relationship DID's transport — its one connection to its mediator — belongs to whoever holds the relationship, which is normally the client, so the VTA could only send as the relationship DID by taking that connection over. `vta/witness/request/0.1`, which had the VTA make both round trips itself, is retired for this reason.

| Step | Task | The VTA | The client |
|---|---|---|---|
| 1 | [`vta/witness/open`](../../witness/open/0.2/spec.md) | Checks custody and the credentials, builds the `witness/session` document as the relationship DID, signs it, and opens its record of the session | Sends the document to the witness |
| 2 | [`vta/witness/present`](../../witness/present/0.2/spec.md) | Verifies the witness's session response, builds the presentations, and returns the signed `witness/session/submit` document | Hands back the witness's session `#response`; sends the submit document |
| 3 | [`vta/witness/accept`](../../witness/accept/0.2/spec.md) | Verifies the VWC and stores it with its evidence | Hands back the witness's submit `#response` |

A stored VWC is read back, with its evidence, by [`vta/witness/list`](../../witness/list/0.1/spec.md), which keeps no session record and is not a step of a session.

Each of those specifications references this file rather than restating it, and conformance is anchored here. The shared wire shapes are in [`witness-party.schema.json`](witness-party.schema.json), and the outcome evidence in the witness family's [`outcome-evidence.schema.json`](../../../witness/_shared/0.1/outcome-evidence.schema.json).

These are the conventions of the **0.2** tasks, which conduct sessions under `witness/session/0.2` and `witness/session/submit/0.2` and carry the member presentation in the submit payload's own `memberPresentation`. The 0.1 tasks keep [their own conventions](../0.5/CONVENTIONS.md). A session opened by `vta/witness/open/0.2` is continued only by `vta/witness/present/0.2` and `vta/witness/accept/0.2`; a task of the other version **MUST** answer it with `<slug>:sessionUnknown`, as for any session in a state that does not admit it (§3 item 2).

## 1. Terms

- **Session.** One participating party's `witness/session` with one witness. Its **session id** is the `id` of the `witness/session` document the VTA built — the thread of every document of the session, and the VWC's `taskContext`.
- **Session record.** What the VTA keeps for a session between the steps (§3).
- **Relationship presentation.** The presentation, by the relationship DID, of the relationship credential (and the membership credential when it names the relationship DID), bound to the session's challenge and domain.
- **Member presentation.** Under the member link, the presentation by the member DID of the membership credential naming it, bound to the same challenge and domain, carried in the submit payload's `memberPresentation` ([`witness/session/submit/0.2`](../../../witness/session/submit/0.2/spec.md#the-member-presentation)).

## 2. Custody

1. The VTA **MUST** sign every document of the session it builds, and the relationship presentation, with a key of the relationship DID it holds in a context the caller may act in, and **MUST NOT** sign them as itself. It **MUST** sign the member presentation with a key of the member DID held on the same terms, and use that key for nothing else in the session.
2. The VTA **MUST** build every document and presentation itself, from the session record and the credentials in its vault. It **MUST NOT** sign or present over bytes the caller supplies. What the caller hands back is the **witness's** signed answer, which the VTA verifies and reads; it is never an input to a signature except as the source of the challenge and domain, taken from a response verified to be the witness's.
3. The VTA **MUST NOT** send any document of the session itself, and **MUST NOT** contact the witness. It **MUST** resolve the witness's DID, to verify the witness's proofs.
4. No task in this family returns a key, under any outcome.

## 3. The session record

The VTA binds the three steps to one session so that no response from another session — another witness, another party, another of this party's sessions — can be spliced into it.

1. `vta/witness/open` creates the record, keyed by the session id, in the context of the relationship DID's key. It holds: the witness, the relationship DID, the counterparty DID, the relationship thread, the `witness/session` document as built and signed, the vault handles of the credentials and their digests at open, the member DID if any, the `storeAs` handle if any, its state, and its expiry. It holds no presentation and no challenge.
2. Its state is `opened`, then `presented` once `vta/witness/present` has built a submit document, then `accepted`. Every task **MUST** find the record by session id **among the contexts the caller may act in**, and **MUST** answer an unknown, expired, out-of-scope or wrong-state record alike, with `<slug>:sessionUnknown`, so the answer reveals nothing about sessions the caller cannot see.
3. Every document the caller hands back **MUST** be checked against the record before anything is read from it: its `type` is the expected witness response, its `issuer` is the record's witness — the `recipient` of the session document — its `recipient` is the record's relationship DID, its `threadId` is the session id, and its proof verifies for the witness's DID. Anything else is `<slug>:responseInvalid`.
4. The record **MUST** expire. The VTA sets the bound, and **SHOULD** keep it to minutes, near the witness's own session lifetime: a witness session is opened for one occasion. `vta/witness/accept` consumes it.
5. After `accepted`, the VTA **SHOULD** keep a stub — the session id, the stored credential's handle and the VWC's digest — until the record's expiry, so that the same outcome handed back again is answered again (§5), and **MUST** then delete it.

A session the witness refuses — `witness/session:refused`, or a submission refused as `challengeMismatch` or `presentationInvalid` — ends at the client, which received the refusal. The VTA's record simply expires. The client opens a new session to try again.

## 4. Authorization

The authority is **standing over the contexts the task acts in**: the VTA has recorded the caller as entitled to act in the context holding the relationship DID's key, and in every context holding a credential or key the session uses — the member DID's key included. Where that entitlement is expressed as a capability, the registered value is **`witnessRequest`**, for all three tasks — see `Capability` in [`device/_shared`](../../../device/_shared/0.2/device-binding.schema.json).

It is not `sign`. A VTA **MUST NOT** require `sign` for any task in this family, nor satisfy one under a grant of `sign` alone: these tasks sign only documents and presentations the VTA builds itself, bound to one session with one witness, and they read the credential vault and write to it, which `sign` does not confer. A caller without standing, or whose contexts hold no key of the DID the session needs, is refused with the standard `permissionDenied`.

The act is consequential beyond the VTA. It discloses the principal's credentials, in full, to the witness the caller names, and under the member link it links the principal's relationship DID to its member DID there. A VTA's own policy may place a further gate on `vta/witness/open` or `vta/witness/present`; that gate is the VTA's to choose ([SPEC §7.3 item 13](/SPEC.md#73-specification-requirements)), and this family states only that the act is consequential.

Verifying the document's `issuer`, transport identity or `proof` establishes who asked ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)), never that they may.

## 5. Retries

Each task is safe to repeat, which is what lets the client retry a lost answer without an idempotency mechanism:

- **`vta/witness/open`** — a repeat opens a second record and builds a second session document. Neither is sent until the client sends it, and an unsent one expires. Its duplicate is inert.
- **`vta/witness/present`** — a repeat for a session that is `opened` or `presented` builds a fresh submit document. The witness spends its challenge on the first submission it accepts and refuses any other as `challengeMismatch`, so at most one VWC results whatever the client sends.
- **`vta/witness/accept`** — the same outcome handed back again, for a session it accepted, is answered again with the same response and stores nothing new (§3 item 5). A different outcome for an accepted session is `sessionUnknown`.

A VTA still applies duplicate-execution protection to every document ([SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements)). The client **MUST NOT** resend a document **to the witness** under a new `id`: the same document again is a retry, and a new one is a second request.

## 6. What the client sees

The client now carries the principal's presentations, inside the submit document. They are the principal's own material, carried for the principal by the principal's own delegate, and each is bound to one session's challenge and domain, so it is useless in any other session. The client **SHOULD** discard the submit document once the witness has answered it. It **MUST** send each document only to its `recipient`.

What the witness sees is unchanged from what the witness specifications describe: the presented credentials in full, and under the member link the member DID beside the pairwise relationship DID.

## 7. Error codes

Each task declares its own codes under its own slug. Where tasks share a condition, they use the same local part:

| Local part | Meaning |
|---|---|
| `notFound` | A named credential is not in the vault in a context the caller may act in, or is no longer active. |
| `credentialMismatch` | A credential does not fit its role, or has changed since the session was opened. |
| `sessionUnknown` | No session record with this id, in a state that admits this task, in a context the caller may act in. |
| `responseInvalid` | The document handed back is not the witness's answer for this session (§3 item 3). |
| `vwcInvalid` | The witness's answer is genuine, but the VWC it carries does not verify against the session. |

Every other refusal uses the framework's standard codes ([SPEC §8.3](/SPEC.md#83-standard-error-codes)): `permissionDenied` for a caller without standing or a key the VTA does not hold for it, `malformedRequest` for a payload the schema or a stated rule rejects.
