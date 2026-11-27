# Questions for the HCP editors (HCP 2.0-draft-3 carriage as Trust Tasks)

Context: the `hcp/*` Trust Task family in the Trust Tasks registry (`trustoverip/dtgwg-trust-tasks-tf`; all `hcp/*`
specs at 0.1, framework 0.6.0), carrying HCP 2.0-draft-3 (`loyalagents/hcp-spec` @ b790561), with working demos in
TypeScript, Go, Dart and Rust.

**For the HCP editors: these are the assumptions we made to get a working end-to-end implementation, for your
review.** Where the HCP text did not settle a point the implementation had to decide, we took the reading stated as
*Assumption taken*, marked it in the specs as pending your confirmation, and built on it. Each item says what we
assumed and why, and asks you to confirm it or tell us what HCP intends; none is a request to change HCP unless it
says so. Answers update the specs directly.

**Convention.** For each question the specs take the *Assumption taken* and mark it in the prose as "pending HCP
confirmation", linking to the spec's `## Pending HCP confirmation` section, which lists the questions by the numbers
below. When the editors answer, update that section and the marked sentences.

## Most important

1. **Question 37: may a carriage other than HCP's HTTP Binding carry governance operations?** If not, the 18
   governance specs have to be withdrawn or gated, and a wallet cannot manage a person's grants over a messaging channel.
2. **Question 46: is sending disclosed context to a model run by another organization onward disclosure?** Nearly
   every LLM-based client does this, and we have no assumption yet; the answer decides whether such clients can use
   HCP context at all.
3. **Question 56: may a Management Client be authenticated by a signed request under an enrolled key, with no bearer
   token?** Our governance specs and the first wallet integration depend on it, and it departs from HCP's token model.

## Status at a glance

- **Working assumption taken, pending HCP confirmation:** every numbered question except those listed as open
  below. The specs are built on these readings; an answer that differs changes the specs.
- **Taken in the 2026-11-25 review decisions:** question 56 (new). The same decisions also settled details now
  recorded under questions 16 (a 24-hour limit on notice delivery), 25 (why a decline is `permissionDenied`), 36
  (the trust anchor when a provider has no HCP discovery document) and 39 (an expired hold is denied, never
  reopened).
- **Still open, no assumption taken:** questions 44, 46, 48 and 57. Question 47 needs none yet, because we do not offer
  what it asks about.
- **Decided, not carried:** questions 35 and 36 (no Trust Task defines an operation HCP does not).
- **Appear in no spec:** questions 35, 36, 43 to 52 and 57, 13 in all (41 is unused). The others are cited in the
  `## Pending HCP confirmation` section of each spec they affect, question 56 included (the governance specs and
  `hcp/entries/create`, where `accessToken` is now optional).

**What changed from the old file.**
- One numbering, 1-56 (41 unused, so 55 questions), grouped by spec. Phase 2's lettered C1-C7 are integrated (they had also been
  described as reusing 8-14, which other branches held).
- The duplicate 23 is split: old 23 (import outcomes) is now 34; old "23 (phase 3, continued)", which the specs
  cited as 26 and two schema descriptions still cited as 23, is now 28.
- Export size: the carriage bound is **16 MiB** (16777216 bytes, the registry's `maxDocumentBytes` cap), not 64 MiB.
- Three readings that the specs stated without a mark are now marked and asked: 19, 12 and 14.
- 2026-11-25 review fixes: questions 37-40, 42 and 43 added (41 unused); 1, 3, 15, 20, 22, 25, 28, 29, 32 and 34
  amended. Number 41 was reserved for a question that turned out not to be needed. New questions sit under
  their spec group, so the numbers are out of order there.
- 2026-11-25, later: questions 44-52 added, from the HCP-in-VTI implementation plan, the conformance review and
  the wallet-side companion plan. 44, 45, 47 and 48 are in a new group, "Deployment and conformance (not tied to one
  spec)"; 46 and 50 are under `hcp/shares/report`; 49 is under "Every `hcp/*` spec"; 51 is under `hcp/rules/*`,
  `hcp/directives/*`; 52 is under `hcp/grants/request`. None was merged into an existing question: 44 overlaps 37
  (the provider side of the same carriage question), 49 questions the error code that 22's rule uses, and 52 questions
  22's refusal of an unregistered issuer; each asks something different and cross-refers.
- 2026-11-25, review of the PRs: question 53 added under "Every `hcp/*` spec" (a request proof on every request
  that carries an access token, stricter than HCP's bearer model). 22 and 33 amended (22 now cites 53; 33 states
  the 14 MiB export bound under the 16 MiB import bound), and the count in 22 and 49 corrected from "27 specs that
  carry `accessToken`" to 26 specs whose request carries it, plus `hcp/grants/request`.
- 2026-11-26, schema consistency scan: questions 54 and 55 added, in a new group, "HCP's own schemas (not tied to
  one spec)". Both are inconsistencies between HCP's published JSON Schemas, found by comparing them with the
  `hcp/*` schemas; neither changes an assumption the specs already took.
- 2026-11-25, external review: question 56 added under "Governance specs" (a Management Client authenticated by its
  request proof, with no bearer token). 33 updated for the export refusal code's rename to `hcp/export:tooLarge`. 16
  gains the 24-hour delivery limit; 25, 36 and 39 gain the readings the review settled; 49 states plainly that
  `scope-denied` is HCP's HTTP name and is never sent. A "Most important" list and a status summary open the file.
- 2026-11-26: 56's assumption gains the create made with no token (the Management Client's write, under no grant),
  and its specs now carry the change; 25, 36 and 39 point to the spec rules they summarise. The list is published
  in the registry beside the `hcp/*` shared schemas, and each spec's Pending section links to it.
- 2026-11-26, later: question 57 added under `hcp/entries/get`, `create`, `revise`, `withdraw` (health entries
  resting on supporting credentials), with no assumption taken; 56 questions now (1-57, 41 unused). The same change
  widens `hcp:credentialInvalid` in `hcp/entries/create` and `hcp/entries/revise` to 16.2's case, which HCP states
  and so needs no question.
- 2026-11-26, terminology pass: the three `hcp:` family codes (`invalidToken`, `versionConflict`,
  `credentialInvalid`) are defined once, in [the family conventions](CONVENTIONS.md#1-family-error-codes), and every
  spec declares each with the same meaning, so 2's reading now reaches every spec that declares `hcp:invalidToken`.
  Every working-assumption mark names its question; no question changed.

Old to new: 7→1, 1→2, 14→3, 6→4, 5→5, C1→6, C3→7, C2→8, C5→9, C4→10, 16→11, (new)→12, 20→13, (new)→14, 2→15,
3→16, 4→17, C6→18, (new)→19, 24→20, 8→21, 9→22, 10→23, 11→24, 12→25, C7→26, 13→27, 26 (the second 23)→28, 15→29,
21→30, 22→31, 25→32, 17→33, 23→34, 18→35, 19→36.

---

## Every `hcp/*` spec

1. **May a provider claim the carriage in `conformsTo`, and can HCP define an identifier for it?** HCP 1.7, 18 and
   19 give conformance identifiers only for Core, the HTTP and MCP Bindings and the profiles. A provider serving the
   `hcp/*` Trust Tasks conforms to neither binding.
   *Assumption taken:* a provider MUST NOT list any identifier for the carriage in `conformsTo`, and advertises the
   tasks through `trust-task-discovery`. *Request:* a third binding identifier for Trust Task carriage, beside `http`
   and `mcp`. See question 37: the binding identifier asked for here is the same question.
   *Specs:* all 28 (Conformance section).

<!-- Added 2026-11-25; a separate list, so the number renders as written. -->

43. **May a client present a DPoP-bound access token instead of a Bearer token (9.3, 13.6)?** 9.3 says "All
    requests MUST include a Bearer token in the Authorization header". Under the OAuth interop profile, 13.6 requires
    the OAuth 2.0 Security Best Current Practice (RFC 9700), which says access tokens SHOULD be sender-constrained,
    for example with DPoP (RFC 9449) or mutual TLS; HCP itself sender-constrains only public clients' refresh tokens
    (13.6, 21.3).
    *Assumption taken:* Bearer by default, with a DPoP-bound token (`Authorization: DPoP`, plus its `DPoP` proof
    header) accepted as an opt-in for clients that ask for one. This mainly affects implementations of HCP's HTTP
    binding. The `hcp/*` Trust Tasks carry the token value in `accessToken` and are unchanged: a DPoP proof is bound
    to an HTTP method and URI, so it has no place in a Trust Task document, whose own proof and `issuer` VID do the
    sender binding there (see question 22).
    *Ask:* does 9.3 permit `Authorization: DPoP` for clients that support it, or should it say "a Bearer or
    sender-constrained token"? If DPoP is allowed, is it signalled in discovery (19) or in the token response?
    *Specs:* none changed; it bears on how the `hcp/*` specs say the provider validates the token "exactly as over
    HCP's HTTP Binding".

<!-- Added 2026-11-25; a separate list, so the number renders as written. -->

49. **What error does a token presented by the wrong client get, and does it tell a thief the token is live (5.5,
    9.3, 6.2.2)?** Question 22's carriage rule refuses a token issued to a client other than the one the `issuer` VID
    identifies, with `permissionDenied`. That is how the carriage sends HCP's not authorized, with one fixed message;
    `scope-denied` is HCP's HTTP name for it and is never sent over the carriage. A revoked, expired or unknown token
    gets unauthenticated (HCP's `invalid-token`, sent as `hcp:invalidToken`, question 2). So someone who
    holds a stolen token and sends it from their own VID can learn from the code whether it is still live
    (`permissionDenied` if it is, `hcp:invalidToken` if not), unless the provider checks the binding before the
    token's status. HCP itself does not meet this, because it does not bind a token to a client (9.3).
    *Assumption taken:* question 22's rule as written: `permissionDenied`, with the one fixed message, revealing
    nothing about the grant beyond the code. Our implementation tests that the refusal says nothing more.
    *Ask:* should a token presented by the wrong client be refused as unauthenticated (`hcp:invalidToken`), like a
    revoked or unknown one, so that the code tells a thief nothing? Or should revoked, expired and wrong-client tokens
    all get not authorized? Or is it enough to check the binding first, so that every token the provider issued is
    refused the same way from the wrong client? The rule is the carriage's, but HCP's answer to question 22 decides it.
    *Specs:* the 26 specs whose request carries `accessToken` (Authorization, Errors); none changed yet.

<!-- Added 2026-11-25; a separate list, so the number renders as written. -->

53. **May the carriage require a request proof wherever a request carries an access token (9.3, 5.5)?** Question 22's
    token binding refuses a token issued to a client other than the one the `issuer` VID identifies, but the VID is
    authenticated only by the request's proof. Six reads (Query, Get, read own grant, List grants, and the standing-rule
    and directive lists) first made the request proof only RECOMMENDED, so a party that obtained one of those
    documents (from a proxy log, or a forwarded copy) could replay its token in a fresh, unsigned request, and the
    binding would not apply. On Query and Get that discloses the person's context.
    *Assumption taken:* a request proof is REQUIRED on every `hcp/*` request that carries `accessToken`, so the token
    binding always applies. This is a carriage rule of the `hcp/*` family, and it is stricter than HCP's bearer model,
    under which presenting a valid token is enough (9.3): over the carriage an unsigned request with a valid token is
    refused, where HCP's HTTP Binding would serve it.
    *Ask:* is HCP content for a carriage to require sender authentication on every token-carrying request, beyond its
    bearer model? Or would HCP rather a carriage accept an unsigned request as a plain bearer request, as over HTTP,
    with the binding protecting only signed requests?
    *Specs:* the 26 specs whose request carries `accessToken` (each lists it in its Pending section); the six reads
    named above changed their request proof from RECOMMENDED to REQUIRED. `CONTRIBUTING-SPECS.md` states it in the
    family's front-matter profile.

## Governance specs

37. **May a carriage other than the HTTP Binding carry governance operations (1.6, 5.4, 9.1, 10.1, 6.1.9)?** 1.6 gives
    the governance plane "Bindings: HTTP only", and says a Management Client reaches governance operations "over the
    HTTP Binding". 5.4 says they "are carried only by the HTTP Binding", 9.1 calls it "the only binding that carries
    governance operations", and 10.1 says MCP carries none. The `hcp/*` family carries them as Trust Tasks.
    *Assumption taken:* a carriage other than the HTTP Binding may carry governance operations where the caller is a
    Management Client acting for the Authorizing Principal (6.1.9). Every rule HCP places on the operation applies
    unchanged, and any other caller receives not authorized.
    *Ask:* do 1.6, 5.4 and 9.1 restrict governance to HCP's own bindings (so the Trust Task carriage needs HCP's
    sanction), or do they only say which of HCP's two bindings carries governance? Would HCP state the rule as "only
    by the HTTP Binding or a carriage HCP recognises (see question 1)"? The binding identifier question 1 asks for is
    the same question.
    *Specs:* the 18 governance specs: `hcp/grants/{list,revoke}`, `hcp/audit/read`, `hcp/disclosures/summary`,
    `hcp/export`, `hcp/import`, `hcp/entries/{hard-delete,confirm,set-categories,restrict-purposes}`,
    `hcp/rules/{list,set,remove}`, `hcp/directives/{list,set,remove}`, `hcp/delegations/{list,end}`.

<!-- Added 2026-11-25; a separate list, so the number renders as written. -->

56. **May a Management Client be authenticated by a proof under a key the Authorizing Principal enrolled (3.2.3
    "credentials"), with no bearer token, over a carriage that requires a request proof?** 3.2.3 defines a
    Management Client as acting "under credentials the Authorizing Principal obtained by authenticating for that
    purpose"; it does not say the credentials are a bearer token. HCP's HTTP Binding uses a token on every call (9.3),
    and no HCP operation issues a Management Client token. A wallet that the person pairs with their provider already
    proves its identity by signing every request (question 53), but holds no token.
    *Assumption taken:* `accessToken` is optional on governance requests and on `hcp/entries/create`. Where it is
    absent, the provider authenticates the request's `issuer` VID, by its REQUIRED request proof, as a Management
    Client the Authorizing Principal enrolled by authenticating (3.2.3), and refuses any other issuer with
    `permissionDenied` (the one fixed message). Where a token is present, the token rules apply as now, including
    question 22's binding. A provider keeps a record of the Management Clients each person enrolled. A create with no
    token is the Management Client's write for the person, under no grant.
    *Ask:* does an enrolled key, proved by a signature on each request, count as Management Client "credentials" under
    3.2.3? Or does HCP intend a bearer token on every governance call, whatever the carriage?
    This is not question 37, which asks whether governance may travel off HTTP at all; it asks how the caller is
    authenticated once it does. See also question 22 (the token's binding to the `issuer` VID) and question 53 (a
    request proof on every token-carrying request).
    *Specs:* the 18 governance specs and `hcp/entries/create`, whose requests now carry `accessToken` as OPTIONAL
    (Authorization, Pending HCP confirmation).

## Several specs

2. **Which Core error does a request under a revoked grant get?** HCP 6.2.2 says no request presenting evidence
   issued under a revoked grant succeeds, and that such a failure is a notice of revocation, but names no error.
   5.5 defines `invalid-token` ("Missing, malformed, or expired authorization evidence") and `scope-denied`.
   *Assumption taken:* `invalid-token`, carried as `hcp:invalidToken`.
   *Specs:* the `hcp:invalidToken` definition in [the family conventions](CONVENTIONS.md#1-family-error-codes),
   which every spec declaring the code carries unchanged; `hcp/entries/query` (the revoked-grant example, Errors),
   `hcp/grants/revoke` (Revocation rules), `hcp/entries/{get,create,revise,withdraw}`,
   `hcp/grants/{read-own,relinquish}`, `hcp/shares/report` (Pending sections).

3. **Pagination and `nextCursor: null` (9.5).** The audit log is cursor-paginated, and grants (9.4.6) and the
   disclosure summary (9.4.12) may be, with `nextCursor` null when the set is exhausted. The Trust Task wire
   conventions never send `null` for it.
   *Assumption taken:* `cursor` and `limit` as request members, `limit` capped at 1000 (a framework bound);
   `nextCursor` is **absent**, not null, when exhausted (a declared departure, no rename); the cursor stays opaque and
   valid for at least 300 s; cursors are bounded at 2048 characters (a framework bound), and a provider paginates
   wherever a page would exceed the task's `maxDocumentBytes`. Rules, directives and delegations lists are not
   paginated (9.4.13-9.4.15).
   *Specs:* `hcp/audit/read`, `hcp/grants/list`, `hcp/disclosures/summary`, `hcp/rules/list`, `hcp/directives/list`.

4. **Is a re-delivered read exempt from a second `context.read` audit event?** 8.1 requires a `context.read` event
   for every read; 9.6 forbids a second event for a replayed create or revise (MUST) and withdraw (SHOULD), but says
   nothing of Query or Get.
   *Assumption taken:* the Trust Task duplicate-execution rule applies: a re-delivered document with the same `id`
   and content is not executed again, so there is no second disclosure and no second event.
   *Specs:* `hcp/entries/query`, `hcp/entries/get` (Repeated requests).

<!-- Added 2026-11-25; a separate list, so the number renders as written. -->

42. **Presence signals the carriage adds (5.2.1, 5.2.2, 5.4.1, 5.4.3, 6.4, 8.2, 17.7).** Where HCP gives an
    empty list, a null or an absent member different meanings, the carriage makes the document say which, so that the
    meaning never rests on whether a member is present. HCP's own shapes leave these members optional. (How any one
    library encodes empty, null and absent is the carriage's own implementation matter and is not asked here.)
    *Assumption taken:* the carriage makes these members required:
    (1) every audit event carries `entryId`, `relatedEntryId`, `purpose`, `purposeDescription` and `grantId`, each a
    value or null, as the 8.2 template writes them; an event redacted for a former delegate carries `entryId` and
    `relatedEntryId` as null, not omitted (17.7). (2) An export carries `options`, with `includeVersions` and
    `includeAudit` each true or false; a reader learns from `options`, not from whether the arrays are present,
    whether `versions` and `audit` were asked for. That is required of the export a provider produces only:
    import accepts an export written to HCP's schema alone, with `options` or any of its members absent, reads an
    absent one as "unknown whether versions or the audit log were requested", and never infers either from the arrays.
    Import likewise accepts audit events written to HCP's audit-event schema alone, with any of the five members of (1) absent, reads an absent one as "unknown what the exporting provider recorded", and never reads it as null. (3) A get-entry or query result always carries
    `directives`: a list, empty when none applies, for a Context Client, and null for any other requester. (4) A query's `categories`, where present, is never empty.
    *Ask:* would HCP make these members required in its own schemas, as its templates already show them (`options` is
    a listed export member)? Does a former delegate's "never `entryId`/`relatedEntryId`" (17.7) allow them to be
    carried as null? Does an empty `categories` in a query mean anything, or may it be refused? May a result to a
    non-Context Client carry `directives: null`, or should the member be absent?
    *Specs:* `hcp/audit/read`, `hcp/export`, `hcp/import`, `hcp/entries/get`, `hcp/entries/query`.

## `hcp/entries/query`

5. **May a Management Client call Query, and receive whole entries?** 5.1 offers context operations to "HCP Clients
   acting under a grant"; 5.2.1, 7.4 and `query-result.json` foresee a requester that is not a Context Client,
   receiving entries whole and no `directives`. Nothing forbids a Management Client from querying.
   *Assumption taken:* a Management Client may query and receives whole `Entry` objects, with no `directives`.
   *Specs:* `hcp/entries/query` (Authorization).

## `hcp/entries/get`, `create`, `revise`, `withdraw`

6. **`ref` or `entryId`: both, neither, or the wrong one?** 5.2.2, 5.2.4 and 5.2.5 say "`ref` or `entryId` — Yes,
   one of them", and the published get-entry request schema enforces neither. Is a request carrying both invalid?
   What does a Context Client that sends an `entryId` (which 7.4 says it never learns) get?
   *Assumption taken:* exactly one; both or neither is `malformed-request` (`malformedRequest`); a Context Client
   naming an entry by `entryId` gets `scope-denied` (`permissionDenied`, the one fixed not-authorized text). Stated in
   member descriptions, not the schema.
   *Specs:* `hcp/entries/get`, `hcp/entries/revise`, `hcp/entries/withdraw` (Request, Authorization).

7. **`classification.permittedPurposes` on a create.** 4.2.6 says only the Authorizing Principal sets it, by a
   governance operation, but `operations/create-request.json` references the common `classification`, which allows it.
   *Assumption taken:* a create carrying it gets `malformed-request` (`malformedRequest`); stated in the description,
   not checked by the schema.
   *Specs:* `hcp/entries/create` (Request).

8. **Revise: `basis`, `permittedPurposes` and `proposal`.** 5.2.4 gives a revision `assertion.basis` and
   `assertion.permittedPurposes` "as for create", and 4.7.4 lets a client propose a resolution with the create or
   revise that asserts it, but `operations/revise-request.json` omits `basis`, `permittedPurposes` and
   `relationships[].proposal`. Is the schema behind the text?
   *Assumption taken:* the text governs; all three are carried.
   *Specs:* `hcp/entries/revise` (HCP field mapping).

9. **Revising a rejected version: which error?** 5.2.4: "A Context Client MUST NOT revise an entry whose current
   version the Context Subject has rejected." The error is not named.
   *Assumption taken:* `scope-denied` (`permissionDenied`).
   *Specs:* `hcp/entries/revise` (Authorization).

10. **Withdraw: `hardDeleted`, and withdrawing a withdrawn entry.** `operations/withdraw-result.json` and the 9.4.5
    example carry `hardDeleted`, which 5.2.5's result list does not name. And what does a new withdrawal of an entry
    already `deleted` return? 9.6 says repeated deletion is "harmless to entry state but appends a duplicate audit
    event", while 5.2.2 treats a non-`active` entry as not found for reads.
    *Assumption taken:* `hardDeleted` is an OPTIONAL member, always `false` (hard delete is governance); a new
    withdrawal of a `deleted` entry succeeds with the same result and appends another `context.withdrawn` event.
    *Specs:* `hcp/entries/withdraw` (HCP field mapping, Repeated requests).

<!-- Added 2026-11-26; a separate list, so the number renders as written. -->

57. **Health entries resting on supporting credentials (16.2, 14.6, 14.7).** 16.2 requires an `issuer-attested`
    entry in `health_context` to carry a credential, as its content or in support of it, at least one of which passes
    verification at ingest, and rejects a write where none passes with credential invalid. Two points that rule
    leaves open:
    (a) 14.6 says the credential-invalid error's detail names which check failed (proof, securing mechanism,
    verification method or status), which assumes one credential. Where the entry carries several supporting
    credentials and each fails a different check, which does the detail name: each credential's failure, the first,
    or only the fact that none passed?
    (b) An entry admitted only because a supporting credential passed at ingest: if that credential is later revoked
    or suspended, should the entry still be disclosed? 14.7 withholds disclosure, and sets `revoked` or `suspended`,
    only for a *content* credential; for a supporting credential it changes only its `verification` (14.8). 16.2's "at
    least one credential passes" then holds at ingest but not afterwards.
    *Assumption taken:* none. `hcp/entries/create` and `hcp/entries/revise` refuse such a write with
    `hcp:credentialInvalid` as 16.2 says, and their `message` names the failed check without choosing among several;
    disclosure follows 14.7 as written.
    *Ask:* for (a), what should the detail say; for (b), is disclosure after the last passing supporting credential is
    revoked or suspended intended, and if not, which state should the entry take?
    *Specs:* none yet (`hcp/entries/create`, `hcp/entries/revise` and the disclosing specs would follow the answer).

## `hcp/entries/hard-delete`, `confirm`, `set-categories`, `restrict-purposes`

11. **Hard-delete result (5.4.2, 9.4.5).** 9.4.5 shows the result of a withdraw (`entryId`, `status`,
    `hardDeleted`, `timestamp`) but not of `?hardDelete=true`.
    *Assumption taken:* the same shape with `hardDeleted: true` and no `status` (the entry is no longer held);
    `timestamp` is renamed `respondedAt` as in `hcp/entries/query`.
    *Specs:* `hcp/entries/hard-delete`.

12. **May a delegate confirm or reject an entry? (new)** 3.8 says confirming or rejecting is "performed by the
    Context Subject through a Management Client"; 17.4 lets a delegate perform governance operations on entries in
    categories within the delegation's scope, and 5.4 lists confirm or reject among the governance operations.
    *Assumption taken:* a delegate's Management Client may confirm or reject an entry in categories within the
    delegation's scope, and no other (an entry outside scope is refused with `permissionDenied`, as one that does not
    exist).
    *Specs:* `hcp/entries/confirm` (Authorization).

13. **Change categories: an unsupported category, and a delegate's scope (4.5, 4.2.1, 17.4, 9.4.10).** 4.5 requires a
    provider to reject a write any of whose categories it neither holds nor supports, and 5.2.3 applies that to
    create; 9.4.10 does not say whether it covers the governance operation. 17.4 does not say whether the new
    categories must also be within a delegate's scope.
    *Assumption taken:* (a) an unheld, unsupported category is `malformedRequest`, as for a write; (b) under a
    delegation, both the entry's current categories and every new one must be within scope, otherwise
    `permissionDenied` (the one fixed message), so a delegate can neither reach an entry outside scope nor move one
    out of it.
    *Specs:* `hcp/entries/set-categories`.

14. **The entry returned after a purpose narrowing is removed (new).** 9.4.10 removes the Authorizing Principal's
    narrowing with `permittedPurposes: null` and returns the entry, without saying whether that entry omits
    `classification.permittedPurposes` or carries it as null. 4.2 says "Absent means no limit".
    *Assumption taken:* the returned entry omits `classification.permittedPurposes` (the entry schema has no null for
    it).
    *Specs:* `hcp/entries/restrict-purposes` (Response).

## `hcp/grants/revoke`, `hcp/grants/revoked`

15. **What does revoking an already-revoked, expired or unknown `grantId` return?** 9.4.6
    (`DELETE /v2/grants/{grantId}`, 204) and 6.2.1 do not say.
    *Assumption taken:* revoking an already-revoked grant succeeds idempotently with the empty response (no second
    `grant.revoked` event, no second notice). Revoking an `expired` grant (its `expiresAt` passed, or its delegation
    ended, 17.6) succeeds the same way and changes nothing: the status stays `expired`, never becomes `revoked`, and
    there is no `grant.revoked` event and no notice. An unknown `grantId`, or one the caller cannot see, gets
    `permissionDenied` with the one fixed text (5.5 existence hiding). The visibility check comes first.
    *Specs:* `hcp/grants/revoke` (Repeated requests).

16. **The `iat` freshness rule versus retries (9.9).** Clients SHOULD reject a SET whose `iat` differs from local time
    by more than 300 s, yet delivery is retried at least 5 times over at least 1 hour and every attempt MUST carry the
    same token, so a client that missed the first attempts rejects every later one.
    *Assumption taken:* each delivery attempt is a fresh Trust Task document (new `id`, `issuedAt` and proof); the
    SET's `jti` is carried as a REQUIRED payload member `noticeId` (a `urn:uuid`), the same on every attempt, with the
    same `threadId` and payload. The client applies the 300 s check to each attempt's `issuedAt` and deduplicates on
    `noticeId`, answering a repeat with the normal empty acknowledgement, which the provider treats as delivery
    success. A declared departure from 9.9's "same token" wording. The provider makes no attempt more than 24 hours
    after the first (a carriage bound; HCP sets only the minimum), and the client keeps each processed `noticeId` for
    at least 25 hours, so its deduplication record has a known size.
    *Ask as:* may a retry carry a fresh `iat` (and signature) with the same `jti`? And may a carriage stop delivery
    after 24 hours?
    *Specs:* `hcp/grants/revoked` (Request, Field mapping, Delivery rules, Errors).

17. **Does a grant that ends because its delegation ended get the `grant-revoked` notice?** 17.6: its status becomes
    `expired` (`grant.expired`, reason `delegation-ended`), and "invalidation and notification follow 6.2.2-6.2.4, as
    for a revocation"; 9.9 names only the `grant-revoked` event type.
    *Assumption taken:* the same notice, with an OPTIONAL payload member `reason` (`revoked` | `delegation-ended`,
    absent = `revoked`) and `revokedAt` = when the delegation ended; the delegation's own reason (`majority`,
    `expiry`, ...) is never sent.
    *Ask:* a `reason` member on the event, or a separate event type?
    *Specs:* `hcp/grants/revoked`, `hcp/delegations/end`.

## `hcp/grants/read-own`, `hcp/grants/list`

18. **Read own grant: where do the directives go?** 5.2.6 says the client "receives that grant only, with the
    directives that apply to it", but `operations/read-own-grant-result.json` is the bare grant and 9.4.6 names no
    directives member.
    *Assumption taken:* `{ "grant": <grant>, "directives": [...] }`, `directives` REQUIRED and empty when none applies.
    *Specs:* `hcp/grants/read-own` (HCP field mapping).

19. **List grants with a Context Client's token (new).** Over HTTP, `GET /v2/grants` with a Context Client's token
    returns the one grant under which the token was issued (9.4.6), while 6.1.9 says a Context Client's grant never
    authorizes a governance operation, and 5.2.6 makes reading one's own grant a context operation.
    *Assumption taken:* as over HTTP: a Context Client presenting its own token gets a `grants` list holding only that
    grant (if it has the `status` asked for; otherwise empty), with `authorizingPrincipal.delegation` withheld (17.4)
    and no `nextCursor`; any other caller that is not a Management Client acting for the Context Subject or a delegate
    gets `permissionDenied`. (Replaces the earlier reading, which refused every Context Client.)
    *Specs:* `hcp/grants/list` (Authorization, Errors).

20. **Which grants does a delegate's Management Client see in List grants (9.4.6, 6.1.5, 17.4, 17.5)?** 9.4.6 and
    6.1.5 say "all grants for the instance"; 17.4 limits a delegate's governance operations to the delegation's
    scope; 17.5 lets a delegate revoke "any grant within the delegation's scope".
    *Assumption taken:* the narrowest reading: a delegate is listed only the grants every one of whose `access`
    details' categories lies within the delegation's scope (which includes every grant they authorized under it); the
    Context Subject's Management Client is listed every grant.
    *Specs:* `hcp/grants/list` (Authorization), `hcp/grants/revoke` (Authorization).

## `hcp/grants/request` (not a carried operation; declares no `carries`)

21. **May a Trust Task be the channel through which a grant is requested?** 6.1.6 allows "another authenticated
    channel" meeting its outcome rules, and 12.2 mandates no first-party wire format; under the OAuth Interop Profile
    the request is the OAuth request of 13.3.
    *Assumption taken:* `hcp/grants/request` is an authenticated channel meeting 6.1.6 in full; a First-Party provider
    may use it as its channel (12.2), and an OAuth Interop provider may offer it only beside 13.3. It carries RFC
    9396-shaped `access` details.
    *Ask:* would HCP sanction this channel (or a profile for it), and should it live under the `hcp/` namespace?

22. **Can the client's VID be its HCP Client identifier?** With no token yet, the request's proof is the only client
    authentication.
    *Assumption taken:* the request `issuer` VID, authenticated by its proof, is the client identifier recorded in the
    grant's `client.id`; an unregistered issuer gets `permissionDenied`; the person is shown the registered
    `client.name` and operator, never a name from the request. Across the `hcp/*` tasks, a provider **MUST** refuse with
    `permissionDenied` a token issued to a client other than the one the `issuer` VID identifies; every request
    that carries a token also carries a proof, so the binding always applies (question 53). HCP does not bind a token to a VID (9.3 validates the token alone): the specs state this
    token binding as a carriage rule of the `hcp/*` family, not HCP's, and note in each Conformance section that it
    is a check HCP does not make, which the Trust Task framework would otherwise put in a separate task (its SPEC 9.4
    item 6); it stays in the carriage because it protects the token that the carriage moves from the
    `Authorization` header into the document, where the document can be stored or forwarded.
    Retained copies: the proof is verified over the document as received, at receipt. A provider that keeps a request
    (as a duplicate-execution record or as evidence) removes `accessToken` from the copy and records in its place
    `accessTokenDigest`, the SHA-256 multihash of the token's UTF-8 bytes in multibase base64url without padding
    (the framework's digest form), together with a record that the proof was verified at receipt (when, under which
    verification method, with what result); it relies on that record and never re-verifies the proof over the
    altered copy. A client keeping a `hcp/grants/request` response does the same.
    *Ask:* would HCP bind an access token to the client identity that presented it, or does HCP see the binding as
    something a carriage may add?
    *Specs:* the 26 specs whose request carries `accessToken`, as well as `hcp/grants/request`, which issues the
    token in its response.

23. **How does a non-redirect request name the person?** In OAuth the person signs in at the Authorization Service;
    a Trust Task reaches the provider without them, and a provider may host many instances.
    *Assumption taken:* an OPTIONAL `subjectHint` (like OIDC `login_hint`); an unknown or unreachable person is answered
    as an undecided request (a next step), then refused with the same `permissionDenied` as a decline, so a request
    cannot probe for an account.

24. **May a client ask for a persistent grant?** 6.1.4: persistence MUST NOT be pre-selected "by any means, including
    ... a pre-filled value", and OAuth has no standard parameter for it.
    *Assumption taken:* `grantType: persistent` only lets the provider offer persistence as 6.1.4's separate act
    after the grant is confirmed; the result is temporary unless the person elects it. `expiresIn` (seconds, max
    86400, default 4 h per 6.1.3) may be shortened by the provider.

25. **Is a token returned in a Trust Task response acceptable grant evidence?** 13.3: the OAuth token response carries
    the authorization details granted.
    *Assumption taken:* the signed `#response` carries `accessToken`, `tokenExpiresAt` (13.3, 13.4) and `grant`, HCP's
    grant object as approved (6.1.5, as Read own grant returns it, `expiresAt` null for a persistent grant,
    `authorizingPrincipal.delegation` withheld per 17.4); no refresh token in 0.1; the `hcp/*` tasks accept the token
    as any HCP access token. A decline (6.1.7) is `permissionDenied` with the one fixed message, the same answer an
    unknown person gets (question 23). The Trust Task framework sanctions this: a task that exists to issue a grant,
    not to produce a decision, answers a deny after a declined approval as `permissionDenied` (SPEC §4.13.3 rule 2,
    its last sentence; the reasoning is in `hcp/grants/request`, Errors). While there is no refresh token,
    `tokenExpiresAt` SHOULD equal the grant's `expiresAt`.

   *Specs for 21-25:* `hcp/grants/request` (Question 1 also applies).

39. **Who may collect a held grant request's outcome, and what does a resubmission after approval get (6.1.6, 6.1.7,
    9.9)?** HCP 6.1.6 and 6.1.7 define the outcome of a grant request, but no wire format for a channel that holds a
    request while the person decides. HCP 9.9 retries a best-effort delivery for at least 1 hour, and over TSP without
    a return path a reply is best-effort, so a client may resubmit after approval.
    *Assumption taken:* a resubmission is matched to the held request by (`threadId`, `issuer`), where the request's
    proof authenticates the `issuer`. A document on a held or approved thread from another issuer is refused with
    `permissionDenied` (one fixed message) and never gets a next step or the token. Every next step and the response
    go to the held request's `issuer`. For 1 hour after the response is first sent (a carriage bound), a resubmission
    from that issuer with the held payload gets a response for the same grant (same `grantId`, with a newly issued token
    under that grant, since the provider keeps no copy of the token it sent; a re-delivered copy of the answered
    request is answered the same way). It never gets a second grant, a second `grant.issued` event or a second prompt to
    the person, and it is `permissionDenied` if the grant is no longer active. After that hour it gets
    `permissionDenied`, and a new request on a new thread is put to the person again. A hold has a lifetime set by the
    provider's policy; when it passes, the policy denies, and the next resubmission on that thread gets the same
    `permissionDenied` as a decline or an unknown person. A thread that has ended (declined, expired, or approved
    and past its hour) is never reopened, so a client cannot tell an expired hold from an unknown person (the rule
    is `hcp/grants/request`, Holding while the person decides, rule 6).
    *Ask:* is binding the outcome to the requesting client's authenticated identity, plus a bounded same-grant retry
    window, consistent with 6.1.6 and 6.1.7? Should HCP name such a window, or the repeat behaviour, for non-redirect
    channels?
    *Specs:* `hcp/grants/request`.

<!-- Added 2026-11-25; a separate list, so the number renders as written. -->

52. **May a provider register a client when the person first approves its grant request, with no earlier
    registration (3.2.8, 6.1.5, 6.1.6, 12.2)?** HCP takes `client.id`, `client.name`, `client.operator` and
    `client.claims` from "client registration" (6.1.5), and 3.2.8 says registration "MUST identify the Client Operator
    by an identifier and a name the person can recognize". HCP does not say when registration happens, or that it must
    come before a grant request; under First-Party the channel is the provider's own (12.2). Question 22's assumption,
    and `hcp/grants/request`, refuse a request whose `issuer` is not a registered client.
    *Assumption taken:* the specs keep question 22's rule. Our demo departs from it, as a declared divergence: a
    request from an unregistered issuer is held, and the registration is recorded when the person approves it, with
    the client name the person confirms and the Client Operator from the provider's configuration. Nothing the request
    supplies becomes the registered name.
    *Ask:* may registration happen at the first approval, provided 6.1.6 is met (the person is shown the Client
    Operator's name, and whether the client claims to follow directives)? Or must a client be registered before it
    may request a grant? If it may, should `hcp/grants/request` hold such a request rather than refuse it?
    *Specs:* `hcp/grants/request` (Authorization, Errors), which today refuses an unregistered issuer; unchanged until
    answered.

## `hcp/shares/report`

26. **Is a re-delivered onward-sharing report recorded twice?** 5.2.8 requires a `context.shared` event "for every
    report", and 9.6 defines no idempotency for `POST /v2/shares`.
    *Assumption taken:* as question 4: a re-delivered document with the same `id` and content is not executed again
    and appends no second event.
    *Specs:* `hcp/shares/report` (Repeated requests).

<!-- Added 2026-11-25; a separate list, so the number renders as written. -->

46. **Is sending disclosed context to a model run by another organization onward disclosure (3.12, 3.11 invariant
    14)?** 3.12: a client "MUST NOT pass context disclosed to it under a grant to a party operated by a different
    organization", but "MAY pass disclosed context to components and services operated by its own Client Operator,
    and remains responsible for that processing under invariant 14". Invariant 14: "The HCP Client is responsible for
    downstream processing performed by models and other components operating under it." Most LLM-based clients send
    what they read to a model hosted by another organization. Onward sharing under the grant does not help: it lets a
    client state only "what the task needs, in its own words", and never pass on entries.
    *Assumption taken:* none yet. Our demo client avoids the question: it answers deterministically from the entries
    it reads, citing them and their purposes, and its optional model hook is off.
    *Ask:* is a model run by another organization, called under the client's contract with it, a component "operating
    under" the client (invariant 14), or a party operated by a different organization (3.12)? If the latter, what
    must a client do to use such a model: name the model's operator at approval, or have that operator obtain its own
    grant?
    *Specs:* none changed; bears on every LLM-based client. If it is onward disclosure, `hcp/shares/report` does not
    cover it, since onward sharing never passes entries.

<!-- Added 2026-11-25; a separate list, so the number renders as written. -->

50. **Is a person in a group conversation "a party operated by a different organization" for onward sharing (3.12,
    6.1.5, 5.2.8)?** 3.12 bars passing disclosed context "to a party operated by a different organization — that is,
    by anyone other than its own Client Operator", and its exception lets a client tell "an organization it deals
    with". `onwardSharing.domains` limits sharing "to organizations at those DNS domains" (6.1.5), and a report names
    its recipient by a domain (5.2.8). An agent that answers other people in a group conversation from its owner's
    context tells individual people, who are not organizations and may have no domain.
    *Assumption taken:* a person other than the Context Subject is such a party. An agent may use disclosed context in
    a group conversation only under `onwardSharing`, elected by a separate act, with each share reported; otherwise
    it refuses. Not exercised in our demo.
    *Ask:* is a person who is not acting for an organization a party under 3.12? If so, how is that person named in
    `domains`, `notTo` and a report's `recipient`, which expect a DNS domain? If not, what governs telling another
    person?
    *Specs:* none changed; bears on `hcp/shares/report` (`recipient`) and on the `onwardSharing` a
    `hcp/grants/request` may ask for.

## `hcp/rules/*`, `hcp/directives/*`

27. **What does removing an unknown or already-removed standing rule or directive return?** 9.4.14 and 9.4.15 give
    the `DELETE`s a 204 and say a delegate's `DELETE` succeeds only for a rule they set; nothing on unknown or
    already-removed ids.
    *Assumption taken:* both are refused with `permissionDenied` (the one fixed message), so a caller cannot tell an
    unknown rule from one it may not remove; a re-delivered document with the same `id` gets the original empty
    response. For `hcp/delegations/end`, ending an already-ended delegation succeeds idempotently with the empty
    response and no second `delegation.ended` event (as question 15 for grants).
    *Specs:* `hcp/rules/remove`, `hcp/directives/remove`, `hcp/delegations/end`.

28. **Which standing rules and directives does a delegate's list return (6.3, 6.4, 9.4.14, 9.4.15, 17.4)?** 9.4.14 lets
    `GET /v2/rules` (and `GET /v2/directives`) be called "for a delegate within the delegation's scope", but neither
    says which rules or directives a delegate sees.
    *Assumption taken (narrowest):* a delegate's list returns each rule or directive whose `categories` are absent or
    all within scope; one naming any category outside scope is withheld whole (never trimmed). The Context Subject's
    list returns every one. A delegate may therefore not see a rule that still affects a grant they authorize; such a
    grant is refused at issuance (17.4), and the Authorizing Principal is told which categories a rule excludes
    (6.1.6). A delegate's export (5.4.1) carries the same rules and directives.
    *Specs:* `hcp/rules/list`, `hcp/directives/list`, `hcp/export`. (This was the old file's second "23"; the specs
    cited it as 26.)

<!-- Added 2026-11-25; a separate list, so the number renders as written. -->

51. **Is a custom directive action an extension point, and who defines it (6.4, A.3)?** 6.4: "Any other action is an
    absolute URI ([A.3])", and a client that does not recognize the `action` reads the directive from `text` and
    "MUST NOT treat [it] as a permission". A.3, which lists each extension point and what an implementation does with
    an unrecognized value, has no row for directive actions. We want a directive for "answers other people" (whether
    the agent may reply to people other than its owner), which none of `share`, `sign` or `transact` names.
    *Assumption taken:* a custom action is an absolute URI under a domain its definer controls (A.3's general rule),
    defined by the organization offering the setting, with effect `deny` (off) or `permit` (on). The client applies it
    in code, never by placing it in a model's context (10.5). It only narrows: a `permit` does not relax 3.12 (6.4), so
    replying from the owner's context still needs onward sharing (question 50).
    *Ask:* would HCP add a directive-action row to A.3 (custom value: URI; an unrecognized one: read from `text`, never
    a permission)? Who may define one: any organization under its own domain, or only through an HCP registry?
    *Specs:* none changed; bears on `hcp/directives/set`, whose schema already allows an absolute URI.

## `hcp/audit/read`, `hcp/disclosures/summary`

29. **Audit log filter names, and what `client` matches (8.3, 9.4.7).** 8.3 requires filtering by HCP Client,
    category, event type and date range; 9.4.7 names no query parameters.
    *Assumption taken:* `client`, `category`, `eventType`, `from` and `to` (the last four as 9.4.12 names its own),
    each single-valued and optional. `client` matches an event whose initiator is that client, or whose grant was
    issued to it; 8.3 says only "filter by HCP Client".
    *Specs:* `hcp/audit/read`.

30. **Audit log result member, and the date-range bounds (8.3, 9.4.7, 9.4.12, 9.5).** 9.4.7 names no member for a
    page's events, and neither 8.3 nor 9.4.12 says whether `from` and `to` are inclusive.
    *Assumption taken:* `{ "events": [AuditEvent], "nextCursor"? }`, oldest first; `from` and `to` both inclusive,
    compared with each event's `timestamp` (audit) or each disclosure's time (summary).
    *Specs:* `hcp/audit/read`, `hcp/disclosures/summary`.

<!-- Added 2026-11-25; a separate list, so the number renders as written. -->

38. **May a delegate read the disclosure summary, and what does theirs cover (8.6, 3.2.2, 8.3, 17.6, 17.7, 17.10)?**
    8.6 gives the summary to the Authorizing Principal, who may be a delegate (3.2.2). 8.3 and 17.7 say what a
    delegate and a former delegate read in the log, but nothing says what a delegate's summary covers, or whether a
    former delegate has one. A former delegate's summary would show what 17.7 withholds from them, since `entries` and
    `usableUntil` are computed from `disclosed`.
    *Assumption taken:* a delegate, while the delegation holds, gets a summary computed only from the events of the
    grants they authorized under it and of their sub-delegates' grants. They read every member of those events, so the
    summary discloses nothing more. A former delegate is refused as not authorized and reads the record of their
    grants through the audit log.
    *Ask:* confirm. If a former delegate is meant to have a summary (17.6 says they "can read the record of the grants
    they authorized"), which members may it carry?
    *Specs:* `hcp/disclosures/summary`.

## `hcp/delegations/*`

31. **What `status=ended` returns on `GET /v2/delegations` (9.4.13).** "with ended ones where `status=ended` is asked"
    can mean ended delegations in addition to those that hold, or only ended ones.
    *Assumption taken:* additive: without `status`, only delegations that hold; with `status: "ended"`, both. No other
    `status` value is defined.
    *Specs:* `hcp/delegations/list`.

32. **Who may end a sub-delegation, and does the appointing delegate see it (9.4.13, 17.6, 17.10)?** 9.4.13 allows
    only the delegate the delegation names, or the Context Subject where the basis is `appointed`; 17.6 and 17.10 also
    let the appointing delegate end it (`ended-by-delegate`) and the Context Subject end a sub-delegation "wherever
    they may end the parent", though a sub-delegation's basis is `delegated`.
    *Assumption taken:* 17.10 governs: the operation also accepts the appointing delegate (`ended-by-delegate`) and,
    where the parent's basis is `appointed`, the Context Subject (`ended-by-subject`); everyone else gets not
    authorized; and `GET /v2/delegations` for a delegate also returns the sub-delegations they appointed, so they can
    learn the id to end; a delegate's export carries the same.
    *Specs:* `hcp/delegations/end`, `hcp/delegations/list`, `hcp/export`.

## `hcp/export`, `hcp/import`

33. **Export size (5.4.1, 9.4.8).** An export has no size bound and no paging; a Trust Task document has a declared
    size bound.
    *Assumption taken:* `hcp/export` (response) and `hcp/import` (request) carry the export in one document. The
    import request is bounded at **16 MiB** (16777216 bytes, `maxDocumentBytes`; a framework bound, not HCP's) and
    the export response at **14 MiB** (14680064 bytes), so that an export always fits an import request with the
    token, `categoryMap`, envelope and proof it adds (about 1.03 MiB at their schema bounds). A provider whose export
    would exceed 14 MiB refuses with the task's own error code `hcp/export:tooLarge` (renamed from `hcp:exportTooLarge`,
    so that the code is scoped to the task that raises it), and the Management Client uses HCP's HTTP Binding or asks
    again without versions or the audit log.
    *Ask:* is a paged or chunked export planned?
    *Specs:* `hcp/export`, `hcp/import`.

34. **Import: is an entry that cannot be imported a `rejected` outcome, or does the import fail, and what does
    `skipped` mean (5.4.3, 14.6, 5.5, 17.4)?** 5.4.3 and 14.6 say a credential that fails verification is rejected as
    credential invalid, but the import result has one outcome per entry, with `rejected` and a reason. The same arises
    for an unaccepted schema (4.3) and an entry outside a delegate's scope (17.4).
    *Assumption taken:* each such entry is a `rejected` outcome and the rest proceeds; for a credential failure the
    `reason` names `credential-invalid` and which check failed (proof, securing mechanism, verification method,
    status), with no status-list internals (5.5). No Core error is returned for it. Under a delegation the scope is
    checked first, after `categoryMap`, and every out-of-scope entry is `rejected` with one fixed `reason` whether or
    not its `id` is held, so a dry run, which appends no audit event, is not an existence oracle. A deleted entry is
    `skipped` where the provider chooses to omit it: 5.4.3 says "the request chose", but the request has no member
    that chooses, and 5.4.3 lets the provider preserve the status or omit the entry.
    *Specs:* `hcp/import` (Outcomes).

<!-- Added 2026-11-25; a separate list, so the number renders as written. -->

40. **How are a delegate's export and import scoped beyond entries (5.4.1, 5.4.3, 6.3, 9.4.14, 17.4)?** 17.4 limits a
    delegate's governance operations to entries in categories within scope. It does not say whether a superseded
    version is judged by its own categories. 5.4.3 applies the export's standing rules "as the Context Subject's own",
    while 17.4 and 9.4.14 let a delegate set rules only within scope.
    *Assumption taken:* a delegate's export carries only superseded versions whose own categories are all within
    scope. A delegate's import applies only the standing rules the delegate could set through `POST /v2/rules` (each
    naming `categories`, all within scope), records them as set by the delegate, and lists every other rule by
    `ruleId` in `rulesNotApplied`. That member is the carriage's, not HCP's, and lives outside HCP's result: in the
    Trust Task framework's extension member, as `ext["vnd.trusttasks.hcp-carriage"].rulesNotApplied` on the import
    response, a namespace the `hcp/*` family defines for itself. The result proper stays exactly HCP's (`outcomes`,
    `dryRun`), and a client that does not know the namespace ignores it. It was first a top-level result member;
    it moved so that nothing beside HCP's members reads as an HCP member.
    *Ask:* confirm. Should HCP's import result report rules not applied? And what does an import do with the export's
    directives, which 5.4.3 does not mention?
    *Specs:* `hcp/export`, `hcp/import`.

## Deployment and conformance (not tied to one spec)

<!-- Added 2026-11-25; a separate list, so the number renders as written. -->

44. **May a local bridge reach the provider over a carriage other than the HTTP Binding (10.5)?** 10.5: "A local
    stdio bridge to a remote HCP Provider is an HCP Client in its own right. It holds the token, calls the provider
    over the HTTP Binding, has its own Client Operator ([3.2.8]), and is bound by every rule for HCP Clients". Our
    MCP bridge runs locally over stdio and reaches the provider as `hcp/*` Trust Tasks over DIDComm or TSP, not over
    HTTP.
    *Assumption taken:* none yet. We treat such a bridge as an HCP Client in its own right, with its own Client
    Operator and bound by every client rule, as 10.5 says, and no conformance claim we make relies on it.
    *Ask:* does "calls the provider over the HTTP Binding" require HTTP, or describe the usual case? If a carriage HCP
    recognises is allowed (questions 1 and 37), would 10.5 read "over the HTTP Binding or a carriage HCP recognises"?
    Question 37 asks the same for governance operations on the provider's side; this is the client's side.
    *Specs:* none changed; bears on deployment only (whether such a bridge is a conformant client).

45. **Who is the Client Operator when a person runs a published agent plugin inside a commercial agent host (3.2.8,
    12.1, 10.5)?** 3.2.8: the Client Operator is "the organization accountable for an HCP Client", and "A product name
    alone does not satisfy this." 12.1: First-Party accepts only clients "whose Client Operator is the Provider
    Operator". Our agent client is a plugin, published by one organization, that a person installs in an agent host run
    by another, against a provider that the person or a hosting organization operates. 10.5 places the grant with "the
    MCP host or its HCP integration".
    *Assumption taken:* the Client Operator is the organization that publishes the plugin. Its registered name is shown
    at approval and wherever grants are listed, never a name the request supplies (question 22). A deployment claims
    First-Party only where that organization is also the Provider Operator; a provider the person runs themselves
    states that it is aligned with HCP, not conformant, until this is answered.
    *Ask:* is the Client Operator the plugin's publisher, the host's operator, or (where a person runs both for
    themselves) the person? Can a person be a Client Operator or Provider Operator, given that 3.2.8 speaks of
    organizations?
    *Specs:* none changed; bears on deployment only (what approval shows, and where First-Party may be claimed).

<!-- Added 2026-11-25; a separate list, so the number renders as written. -->

47. **May the MCP Binding run over Streamable HTTP under the First-Party package, without OAuth (10.1, 12.2)?** 10.1:
    stdio "Qualifies for the First-Party package only"; over Streamable HTTP, "MCP's authorization, in which the server
    is an OAuth resource server, is mapped onto the OAuth Interop Profile". 12.2: a First-Party implementation "MUST
    NOT list the OAuth Interop Profile's" identifier. A provider remote from the host cannot serve stdio, so under
    First-Party it seems to have no MCP transport.
    *Assumption taken:* none needed yet. We do not offer the MCP Binding under First-Party: we claim First-Party through
    the HTTP Binding, and add Streamable HTTP only with OAuth Interop, for the Agent Interop package.
    *Ask:* may a First-Party provider serve MCP over Streamable HTTP with first-party credentials (for example, tokens
    issued through the 12.2 channel) instead of MCP's OAuth authorization? If not, would 10.1 say so?
    *Specs:* none changed; bears on deployment only.

48. **Where is the 20.2 test hook defined, and is the conformance suite published (20.2, 12.2)?** 20.2: "The suite
    specification defines a test hook for this purpose", and 12.2 points to it. We have not found a suite
    specification or a published suite.
    *Assumption taken:* none yet. We plan to offer a test hook only on lab builds, never where a real person's context
    is held (20.2); its form waits on the suite's definition.
    *Ask:* where is the suite specification, or when is it expected? Does it fix the hook's interface, so that one
    hook serves any run of the suite?
    *Specs:* none changed; bears on conformance testing only.

## HCP's own schemas (not tied to one spec)

<!-- Added 2026-11-26; a separate list, so the number renders as written. -->

54. **Should `common.json#/$defs/category` use the same URI pattern as `#uri` and `#purposeCode`
    (`schemas/2.0-draft-3/common.json`, 4.5)?** A custom category is "an absolute URI" (4.5). The schema's URI arm
    for `category` is `^[A-Za-z][A-Za-z0-9+.-]*:` with `format: uri`, so a bare `scheme:` with nothing after the
    colon matches it, where `#uri` and `#purposeCode` both require `:[^\s]+`, at least one character after the scheme.
    The three patterns describe the same thing, an absolute URI, and disagree.
    *Assumption taken:* the `#uri` reading. Every `hcp/*` category, including those in an export `hcp/import` takes,
    needs at least one non-space character after the scheme, so a category that is a bare `scheme:` is refused with
    `malformedRequest`. In `hcp/import`, which otherwise accepts any export valid under HCP's export schema, this is
    the one constraint kept beyond HCP's schema.
    *Ask:* confirm, and align `#category`'s URI arm with `#uri`?
    *Specs:* `hcp/import` (Request; the shared `Category` in `hcp/_shared/0.1/common` is used by every spec that
    takes a category).

55. **HCP's grant and delegation schemas disagree on two constraints (`grant.json`, `delegation.json`,
    `common.json`; 6.1.5, 17.2, 3.5).** Low priority. First, `grant.json` requires `client.name` to be non-empty
    (`minLength: 1`), but `common.json#/$defs/clientDescription`, which describes the same client and its operator as
    recorded (an entry's `provenance.submittedByClient`), sets no `minLength` on `name`. Second, a grant's
    `access[].categories` and an entry's `classification.categories` are `uniqueItems`, but a delegation's
    `scope.categories` is not, though it lists categories the same way.
    *Assumption taken:* each shape follows its own HCP schema where the two disagree on uniqueness: the shared
    `Grant` requires unique categories and the shared `Delegation` does not. For the client name, both the shared
    `Grant` and the shared `ClientDescription` require it to be non-empty (a framework requirement on what an
    `hcp/*` provider writes, stated in the schemas).
    *Ask:* is either difference intended? If not, which constraint should the two schemas share?
    *Specs:* `hcp/grants/read-own`, `hcp/grants/list`, `hcp/grants/request` (the client name); `hcp/delegations/list`,
    `hcp/export`, `hcp/import` (delegation categories).

## Not carried (no spec marks these)

35. **Dismissing a conflict and declining a proposal (4.7.3, 4.7.4).** HCP names "keeping both" (`dismissed`) and "a
    proposal the Context Subject declines is removed" as the Context Subject's acts, but neither 5.4 nor 9.2 defines
    an operation for them.
    *Decision taken:* not carried; no Trust Task defines an operation HCP does not (SPEC 9.4 item 1).
    *Ask:* are they meant to be governance operations?

36. **Discovery (19).** HCP 19 defines a published document, not an operation.
    *Decision taken:* no `hcp/*` discovery task; a provider advertises the `hcp/*` Type URIs with
    `trust-task-discovery` and keeps its HCP discovery document at `/.well-known/hcp-configuration`. A provider
    reached only over the carriage cannot publish a valid HCP discovery document (it may not claim the carriage in
    `conformsTo`, question 1), so for such a provider the client's trust anchor is the provider VID it obtained when
    the person connected it, and the revocation-notice key is one the VID's DID document authorizes. Where a
    discovery document exists, the binding between it and the VID still applies. (The rule is in `hcp/entries/query`,
    Parties, which every `hcp/*` spec applies, and in `hcp/grants/request`, Parties.)
    *Ask:* should a binding be able to carry the discovery document itself (relevant to a TSP-only client)?

## Noted for the editors (not a question)

- A client whose grant is revoked after it shared has no way to report the share (5.2.8 requires the grant's token;
  6.2.2 invalidates it). `hcp/shares/report` says so and does not fill the gap.
