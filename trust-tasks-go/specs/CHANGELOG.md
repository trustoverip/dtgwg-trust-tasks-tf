# Changelog

All notable changes to `trust-tasks-go/specs` (the generated specification
packages) are documented here. Entries are generated from conventional commits
by the release automation (see RELEASING.md).

## 0.3.2 — 2026-11-28


### Added

- **vta/events**: Events hosted by a person (phase 8) (#1424)

The vta/events/*/0.1 family for VTI-PEV-001 – 043: events a VTA holds for
  its principal outside any community, under a persona identifier (the host).

  Principal side: create, update, publish, cancel, delete, list, show,
  invitations/send, invitations/revoke, registrations/list,
  registrations/decide. Invitee side, under vta/events/invitee/: show,
  register, withdraw, access/show, mine/list — proof REQUIRED, accepted only
  from an invited identifier, charged to the without-standing budget.

  Reuses vtc/events/_shared/0.4 for When, RegistrationSettings,
  AccessDetails, RegistrationStatus, EventRef; vta/events/_shared/0.1 adds
  VtaEvent* defs (roster is visible|hidden only). Bindings regenerated for
  all four libraries; no prebuilt Rust module.

## 0.3.1 — 2026-11-28


### Added

- **vtc/events**: Event rooms, boards, meetings and polls (phase 6) (#1420)

Twenty new draft tasks (0.1) under vtc/events, for VTI-EVT-240 – 282:

  - attachments/{add,remove,list}: a community room bound to the event's
    roster (visible rosters only; `admits: goingAndMaybe` only there), an
    opt-in room with a disclosure on a hidden roster, a board whose audience
    is the event, a community election, or a material released to each
    registrant who is going, as access details are.
  - meeting/show, meeting/agenda/update, meeting/roles/{appoint,remove}
    (chair, secretary — grants, as hosts are), meeting/quorum/{set,show}
    (count or percentage of the roster or of a role; counted from the
    check-ins), meeting/motions/open (a community election through the
    registrar, roll `listed` = members present at opening, refused unless
    quorum is met; board ElectionTemplate, so never binding),
    meeting/minutes/{record,approve}, meeting/actions/{add,update,list}
    (`assignedToMe` is what a member's VTA reads for its calendar).
  - polls/{create,show,answer,close}: candidate times answered yes / if need
    be / no, a voters setting fixed at creation (visible, hostsOnly, hidden),
    closed optionally into a draft event.
  - publication/set: mark a calendar or an event public for the website's
    schema.org Event data and the public calendar feed (read without a task).

  Shared definitions in new sub-family folders (meeting/_shared/0.1,
  polls/_shared/0.1, attachments/_shared/0.1), so no event shape changes.

  An event as a board's audience arrives with vtc/boards/_shared/0.2
  (BoardAudience gains `{kind: "event", eventId}`; ElectionTemplate's roll
  text says how an event board's elections are enfranchised), carried by
  vtc/boards/{create,update,show,list}/0.2 — the four tasks whose payload or
  response carries BoardAudience - otherwise identical to 0.1. Boards 0.1 is
  untouched, so the released TypeScript union is not widened.
  attachments/add and attachments/_shared name boards 0.2. The 0.2 versions
  declare the bounds the 0.1 baseline exempted (auditors, show's `state`),
  so those two allowlist entries go; facilitators/list 0.1 keeps naming
  show/0.1 under a stale-reference exception.

  All four libraries regenerated; no prebuilt Rust module for the new
  versions.

- **vtc/booking**: Booking pages — book a meeting with me (phase 7) (#1419)

New family vtc/booking/*/0.1 (VTI Community Booking Pages, VTI-BKG-001 – 053):
  pages/{create,update,delete,show,list}, slots/list, bookings/{book,reschedule,cancel,list}
  and busy/put, over _shared/0.1/booking.schema.json.

  A booking is an unlisted meeting event (vtc/events/*): the book/reschedule replies
  carry the event reference and registration as the receipt. Tiers (members, known,
  guests) are proven by membership, a partner presentation or a relationship, never
  asserted — no request carries a tier. busy/put takes merged, quarter-hour intervals
  at most ninety days ahead from the host's own agent (VTI-CAL-060/061) and replaces
  the last supply; no response ever says why a slot is not offered.

  Its own family rather than vtc/events/booking-pages/*: a page is not an event and
  has its own lifecycle, authority (the host alone) and tiers; only bookings become
  events. Consumer-side only — no prebuilt Rust module (#1411). TS, Go and Dart
  bindings regenerated.

- **vtc/events**: Check tickets at the door (#1418)
- **vtc/events**: Recurring series, anonymous and persona registration (phase 5) (#1417)

Implements VTI-EVT-190 – 198 (series), 210 – 218 (anonymous registration on
  the predicate credential system) and 230 – 232 (persona registration).

- **vtc/events**: Check-in, tickets, guests, attendance, event identity (phase 4) (#1413)

VTI-EVT-140 - 186 (trustoverip/dtgwg-vti-spec#130).

  - _shared/0.3: the 0.2 shapes plus EventIdentity, EventGuestAdmission,
    EventSelfCheckIn, EventRegistrantKind, EventCheckInMethod,
    EventDoorChallengeValue, EventCheckIn, EventAttendanceTally; the Event
    gains identity, guests, tickets, countsAttendance, attendanceBadge;
    the Calendar gains selfCheckIn and countsAttendance; an invitation
    link may be forGuests.
  - 0.3 of create (identity, with noKeyHolder / keyHolderUnavailable,
    badgeNotFound), update, show, list, publish, cancel; 0.2 of
    calendars/create, update, list; invitations/links/create 0.2
    (forGuests, guestsNotAdmitted).
  - New 0.1: checkin/challenge, checkin/record, checkin/list,
    tickets/show, attendance/show, guests/register, guests/withdraw.
  - groups/_shared/0.1 RuleCondition: eventsAttended appended to the
    fact enum, with optional calendarId and withinDays (additive).

  New versions are consumer-side only (no prebuilt Rust module, #1411).
  Bindings regenerated for Rust, TypeScript, Go and Dart.

- **vta**: Vta/hosting-account/get/0.1 — a VTA's principal reads its self-registered hosting account (#1410)

A VTA that self-registered its DID with a third-party DID hosting service
  (did-management/account/register) keeps the account alive itself; this task
  carries what it knows about that account — the host's lifecycle state and
  dates as last reported, renewal time, the last expiry notice, the last error
  reaching the host — to the principal that can act on it. refresh reads the
  account from the host with account/status (no activity); it never renews.

  Bindings regenerated (Rust, TS, Go, Dart).

- **vta/calendar**: The member's calendar across communities (#1409)

vta/calendar/*/0.1 (13 tasks) for the VTI The Member's Calendar chapter
  (VTI-CAL-001 - 071, dtgwg-vti-spec #128): list (the agenda across the
  contexts the caller has standing in, mine and notified tiers), sync
  (read each community's vtc/events/mine/list as the member), settings
  show/update (sync per context, default reminders, quiet hours),
  alerts set/clear (per-event reminders), feeds create/list/revoke
  (iCalendar subscriptions; the address returned once), export (ICS and
  receipts), receipts list/deposit (the communities' signed replies kept
  as received), busy/share (coarse busy times to a booking page).

  Shared definitions are prefixed VtaCalendar (the events family already
  has Calendar and CalendarId). Bindings regenerated for Rust, TypeScript,
  Go and Dart.

- **vtc/events**: Publish/cancel 0.2, withdraw by presentation, invitation revoke (#1407)

Closes three gaps OpenVTC VTI's phase-2 implementation reported
  (OpenVTC/verifiable-trust-infrastructure#2846):

  - vtc/events/publish/0.2 and cancel/0.2 answer the _shared/0.2 Event
    (invite mode, membersMayInvite, audience.partners), as create/update
    0.2 do; their requests are 0.1's.
  - registrations/withdraw/0.2 gains an optional presentation, in place
    (an additive optional member): a partner community's member withdraws
    the registration they made as one (VTI-EVT-037, VTI-EVT-135), verified
    as register/0.2 verifies it.
  - vtc/events/invitations/revoke/0.1: a host revokes an invitation
    (VTI-EVT-103); its notification is withdrawn, it no longer puts the
    member in the audience, and a registration the member made stands.
    Codes: invitationNotFound, alreadyRevoked.

  Stale references to cancel/0.1 rewritten. Bindings regenerated for
  Rust, TypeScript, Go and Dart.

- **did-management/account**: Self-registration of hosted DIDs, with invites, liveness and expiry notices (#1405)

* feat(did-management/account): self-registration of hosted DIDs, with invites, liveness and expiry notices

  Adds the did-management/account family, for a party with no account on a
  DID hosting service (an agent that minted its own DID before it had
  anywhere to publish it):

  - account/register/0.1: register a self-certifying did:webvh. Identity
    comes from the log itself (the proof is checked against the document the
    log implies, never a network resolution); entitlement from an operator
    invite or a trust community's signed voucher.
  - account/renew/0.1, account/status/0.1: the holder keeps its account alive
    and reads where it stands in the inactivity lifecycle.
  - account/expiry-notice/0.1: the service warns the holder before its DIDs
    are disabled, at disablement, and before purge.
  - account/invite/{issue,list,revoke}/0.1: operator invites (token + claim
    code, single use, stored hashed).
  - server/info/0.2 (extends 0.1): publishes each domain's self-registration
    terms, which a registrant must read before minting, since the path is
    fixed into the DID.
  - did-management/_shared/0.4: HostingAccount and AccountInvite.

  Bindings regenerated for Rust, TypeScript, Go and Dart.

- **vtc/events**: Invitations, opt-outs, announcements (phase 2) (#1406)

VTI Community Events phase 2 (VTI-EVT-100 - 135, dtgwg-vti-spec #126).

  New 0.1 tasks: invitations/send, invitations/list, invitations/respond,
  invitations/links/create, invitations/links/revoke,
  invitations/links/redeem, optouts/set, optouts/list,
  announcements/send, announcements/list.

  0.2 of create, update (invite: none | notify | expect,
  membersMayInvite, audience.partners, partnerInvalid), show and
  registrations/register (a partner member's MembershipPresentation),
  registrations/list (othersGoing in the roster view), and list,
  mine/list, registrations/withdraw, decide, remove (the 0.2 Event and
  Registration). _shared/0.2: EventInviteMode, EventInvitation(+Id,
  Inviter, Answer), EventInvitationLink(+Id), EventOptOut(+Scope),
  EventAnnouncement(+Id), MembershipPresentation, PartnerDid;
  Registration names a partner's member by registrantDid.

  Bindings regenerated for Rust, TypeScript, Go and Dart.

- **vtc/events**: Community events and calendars, phase 1 (#1402)

* feat(vtc/events): community events and calendars, phase 1 (21 tasks)

  vtc/events/*/0.1, the VTI Community Events chapter (VTI-EVT-001 - 095,
  dtgwg-vti-spec #125): calendars (create, update, list), events (create,
  update, publish, cancel, delete, show, list), the member's own events
  (mine/list), access details released to registrants who are going
  (access/show), registrations (register, withdraw, list, decide, remove),
  hosts by appointment (appoint, remove, list), and reports.

  Time is JSCalendar's: local start, IANA time zone, duration. The roster
  setting (visible | hidden) is fixed at publish and recorded on each
  registration; register and withdraw answer the registrant's receipt.
  Reminder offsets are integer minutes; the community never sends a
  reminder itself (VTI-EVT-040).

  Bindings regenerated for Rust, TypeScript, Go and Dart.



### Specifications

- **process-attestation**: Link the CPoE drafts and the process-proof assertion the profile names (#1414)

* spec(process-attestation): link the CPoE drafts and the process-proof assertion the profile names

- **vtc/events**: EventRef carries suggested reminders and the place summary (#1415)

EventRef (_shared/0.2 and 0.3) gains two optional members, in place as a
  released version may: suggestedReminderMinutes and placeSummary. The
  member's VTA reads its events only through mine/list (VTI-CAL-011/021),
  so without them it cannot fire the hosts' suggested reminders
  (VTI-CAL-041) or put the coarse place in a titlesAndPlaces feed.
  mine/list/0.2 item 4 now says it carries them where the event has them.
  Never an address: placeSummary stays the coarse hint (VTI-EVT-062).

## 0.3.0 — 2026-11-27


### Specifications

- **hcp**: A re-delivered grant request gets a new token; questions 58 and 59 for the HCP editors (#1401)

* spec(hcp): a re-delivered or resubmitted grant request gets a new token, and the TSP references name binding 0.3

  hcp/grants/request 0.1 told the provider both to answer a re-delivered request with
  what the first delivery was answered with and to keep the access token in no record of
  the exchange; after a response that carried a token it could not do both. SPEC §7.2 item
  11 asks only that the document is not executed twice. A re-delivered copy of an answered
  request is now answered as a resubmission within the hour is: the same grant with a newly
  issued token, and a resubmission no longer MAY repeat the token it sent.

  hcp/grants/request and hcp/grants/revoked cite TSP binding 0.3 §6 (the text is unchanged
  from 0.2). QUESTIONS.md question 39 and its status line are reworded to match.

## 0.2.0 — 2026-11-27


### Added

- **build**: Declare the rules a payload schema does not check (uncheckedRules) and check the declarations (#1381)

* feat(hcp): carry HCP 2.0-draft-3 query, grant revoke and revocation notice as Trust Tasks

  Add the hcp/* protocol family (SPEC §9.4) with three draft 0.1 specs
  targeting framework 0.6.0, each carrying one Human Context Protocol
  2.0-draft-3 operation:

  - hcp/entries/query: Query (HCP 5.2.1). The HCP access token travels
    opaquely in `accessToken`; the result's `timestamp` is renamed
    `respondedAt`; `categories` gets a framework bound of 256.
  - hcp/grants/revoke: Revoke grant (HCP 6.2, 9.2), destructive and acting
    for the Authorizing Principal, with an empty response for HCP's 204.
  - hcp/grants/revoked: the revocation notice (HCP 6.2.3, 9.9). The SET
    claims map to the envelope, and an empty response carries RFC 8935's
    202. The notice is signed with a provider VID key that the provider's
    JWKS must also publish.

  HCP's invalid-token condition maps to the family code hcp:invalidToken.
  Shared shapes (common, context entry, disclosure, directive) live in
  specs/hcp/_shared/0.1. They were written from the CC BY 4.0 HCP text,
  with attribution, not copied from HCP's schema files. HCP's kebab-case
  enum values are allowlisted as external.

- **hcp**: Add the hcp/* specifications, carrying the Human Context Protocol (HCP 2.0-draft-3) as Trust Tasks (#1380)

* feat(hcp): carry HCP 2.0-draft-3 query, grant revoke and revocation notice as Trust Tasks

  Add the hcp/* protocol family (SPEC §9.4) with three draft 0.1 specs
  targeting framework 0.6.0, each carrying one Human Context Protocol
  2.0-draft-3 operation:

  - hcp/entries/query: Query (HCP 5.2.1). The HCP access token travels
    opaquely in `accessToken`; the result's `timestamp` is renamed
    `respondedAt`; `categories` gets a framework bound of 256.
  - hcp/grants/revoke: Revoke grant (HCP 6.2, 9.2), destructive and acting
    for the Authorizing Principal, with an empty response for HCP's 204.
  - hcp/grants/revoked: the revocation notice (HCP 6.2.3, 9.9). The SET
    claims map to the envelope, and an empty response carries RFC 8935's
    202. The notice is signed with a provider VID key that the provider's
    JWKS must also publish.

  HCP's invalid-token condition maps to the family code hcp:invalidToken.
  Shared shapes (common, context entry, disclosure, directive) live in
  specs/hcp/_shared/0.1. They were written from the CC BY 4.0 HCP text,
  with attribution, not copied from HCP's schema files. HCP's kebab-case
  enum values are allowlisted as external.



### Build & CI

- **go,go-specs,go-proof,go-didcomm,go-capabilityclient**: Require Go 1.24 (#1374)

Raise the `go` directive from 1.22 to 1.24 in the core, specs, proof,
  didcomm and capabilityclient modules and in the two Go examples
  (consumer-app-go, getting-started-go). tsp stays at 1.27, the floor it
  inherits from affinidi-tsp-go. The go.yml matrices that prove the floor
  move from ["1.22", "stable"] to ["1.24", "stable"], and the comments, the
  dependabot note, go.md and the go.mod the generator and smoke tests write
  for their throwaway modules say 1.24.

  1.24 is the first release whose encoding/json honours `omitzero`, which an
  optional member that admits null needs in order to be a comparable struct
  that is still left out when absent.

  A consumer whose toolchain is older than 1.24 and that runs with
  GOTOOLCHAIN=local (a distribution's Go, a pinned CI image) can no longer
  build these modules; with the default GOTOOLCHAIN=auto the go command
  fetches 1.24 itself. Go supports only its two newest releases, and 1.22
  and 1.23 are long out of support.



### Fixed

- **go,go-specs**: Write a required nullable member's null instead of dropping its key (#1378)

* test(go): show a required nullable member losing its null on marshal

  A required member whose schema admits null is generated with omitempty, so
  a nil pointer drops the key and the document fails its own schema's
  required. vta/contexts/update-did 1.2's did is the case: a null did means
  "no DID of its own", and Go writes the member out entirely.

  specs/smoke drives the prebuilt package through a marshal and a round trip,
  and trust-tasks-gen's test checks the tags the per-module generator emits.



### Specifications

- **auth/oob**: DeviceEnrol: a client asks for scoped, long-lived access to a person's VTA, approved from their wallet (#1386)

* spec(auth/oob): copy the versions deviceEnrol changes, unchanged

  _shared 0.1 to 0.2, request 0.2 to 0.3, claim 0.2 to 0.3, identify 0.1
  to 0.2, prove 0.3 to 0.4, grant 0.3 to 0.4, respond 0.3 to 0.4 and
  redeem 0.2 to 0.3, copied byte for byte so the next commit shows only
  what the new versions change. The copies still carry their old version
  numbers and $ids; this commit does not build on its own.

- **vtc/directory/query**: 0.2 answers the member's directory entry (#1383)

* spec(vtc/directory/query): 0.2 answers the member's directory entry

  A caller that meets a member by their DID (the author of a record in a data
  room, the signer of a document) can read their projection with
  vtc/directory/query, but every other directory read is addressed by entry:
  the member's picture (images/get), their entry with shared groups
  (entries/show), and the card a member surface opens. Without the entry
  identifier such a caller can name the member but not show their picture or
  open their card.

  0.2 = 0.1 + a required `entryId` (directory-card `EntryId`) on every answer,
  never on a notFound or in a refusal's details. The notFound rule is now
  stated normatively: an answer is given only where the projection shows the
  caller the member's DID, to the member themselves, or to an administrator,
  so the identifier links the named DID only to an entry that caller could
  already open; Security & Privacy says so. Targets framework 0.6.0.

  All four libraries regenerated.

## 0.1.9 — 2026-11-25


### Specifications

- **vtc/rooms/community**: An administrator sets a community room's record members (#1373)

A community-owned room's profile is written by the community with its label
  and description only, so a room meant as a conversation started with no record
  members, and turning replies and reactions on took a member holding `curate`,
  which most of a community room's members do not hold. The community holds
  `admin` in every room it owns but had no task to set them.

  - create/0.6: optional `recordMembers`, written into the room's profile with
    its label and description; absent is none, as in 0.5. The response carries
    what was recorded.
  - update/0.5: optional `recordMembers` (the whole set) or
    `removeRecordMembers: true`, mutually exclusive; either re-writes the
    profile. Absent, the profile's record members are carried over as before.
  - get/0.6: optional `recordMembers`, as the community last wrote or read back
    the profile.
  - vtc/_shared/0.5: `RoomRecordMembers`, the constraints of
    `RoomProfile.recordMembers`.

  An administrator's surface may offer them as one choice (replies and
  reactions: kind, inReplyTo, writtenAt) and may start it on for a new room;
  the request it sends is still the administrator's explicit choice. Other
  draft specs now point at the new versions. All four libraries regenerated.

## 0.1.8 — 2026-11-25


### Specifications

- **vta/dids**: A step-up hold is a next step, never an error (vta/dids next versions) (#1360)

SPEC §8.6 requires a held task to be answered with trust-task-next-step,
  never an error response. Ruling 1 on trustoverip/dtgwg-trust-tasks-spec#36
  (reading (a)) confirmed that inline step-up delivery in an error response
  was not an intended exception. auth/step-up/approve-request/0.6, which this
  builds on, landed in #1362.

  - vta/dids create 1.2, keys/add 1.1, keys/retire 1.1, keys/revoke 1.1,
    keys/rotate 1.1, recover 1.2, update 1.1, upgrade 1.1: drop
    vta/dids:stepUpRequired. A request with a previewId that is held for a
    step-up is answered with a next step naming approve-response, its hint
    an approve-request/0.6 payload bound to the previewId as before. A
    request with no previewId is answered with a next step naming the task
    itself (continuation: proceed), its hint the dry-run #response carrying
    the stored preview. keys/rotate, recover and upgrade declare the maxItems
    their baselined request arrays lacked.

  Earlier versions stay published and are superseded by the new ones.
  Bindings regenerated for all four languages.

## 0.1.7 — 2026-11-25


### Specifications

- **auth/step-up**: Approve-request 0.6 — a step-up hold is a next step, never an error (#1362)

Implements ruling 1, reading (a), on trustoverip/dtgwg-trust-tasks-spec#36
  for the step-up request itself. Split out of #1360 so it can land while the
  trust-tasks-rs crate is near crates.io's upload limit.

  - `auth/step-up/approve-request/0.6` (`extends: "0.5"`): inline delivery is a
    `trust-task-next-step` whose `expects` entry names
    `auth/step-up/approve-response/0.7` and carries this payload as its `hint`;
    after approval is recorded the producer re-sends the operation
    (`continuation: resubmit`). Delivery in an error response is forbidden. The
    schema is unchanged.
  - The schema-bounds `accepts` baseline moves from 0.5 to 0.6: a version that
    extends 0.5 cannot narrow a bound, and `uniqueItems` over a three-value enum
    already caps it at 3.
  - `auth/oob/redeem/0.1` is `status: retired`, `supersededBy:
    auth/oob/redeem/0.2`.

  The new `vta/dids` versions from #1360 follow once the crate is smaller.
  Libraries regenerated.

## 0.1.6 — 2026-11-25


### Specifications

- **credential-exchange/present**: 0.2 — OID4VP keyed vp_token, either Data Integrity nonce placement (#1359)

`credential-exchange/present/0.2` settles the two readings #1358 found in 0.1.

  - `vp_token` is the OID4VP 1.0 DCQL response (§8.1): an object keyed by
    credential query `id`, each value an array of one or more presentations
    (SD-JWT-VC string or Data Integrity VP object). A bare SD-JWT-VC string is
    still accepted for compatibility; producers send the keyed form.
  - Data Integrity nonce and audience: a consumer MUST accept them as the holder
    proof's `challenge`/`domain` (OID4VP §14.1.2) or as the signed presentation's
    own `nonce`/`domain`; where both are stated they MUST agree, and a
    presentation stating neither is refused. Producers SHOULD use the proof's
    members. "The proof's `nonce`" in 0.1 was a wording slip for `challenge`.
  - Worked SD-JWT-VC and Data Integrity examples (the latter using
    `proof.challenge`/`proof.domain`).

  0.1 stays published and is superseded by 0.2 (the libraries deprecate it).

  `vtc/directory/card/present/0.1` takes the same placement rule in place: it is
  a draft whose wire shape does not change (only the presentation member's
  description), which SPEC §5.2 allows within a draft version, as #831 did for
  the same sentence.

  Prose links to present/0.1 in other specs move to 0.2 (fix-stale-references).
  Libraries regenerated.

## 0.1.5 — 2026-11-24


### Specifications

- **vtc,vetting**: A community's correlation scope is directed or public, and its profile declares it (#1356)

DTG Credentials Working Draft 0.7.0 makes a community's own correlation
  scope `directed` at minimum and `public` where the community is open to
  outsiders; it is no longer always `public`.

  - vtc/_shared/0.1 community: CommunityProfile and CommunityProfileView
    gain an optional `correlationScope` (`directed` | `public`), the
    `issuerScope` the community's credentials carry. Absent, implementations
    issue `public`, as before.
  - vtc/community/profile/update 0.1 accepts it (and refuses `pairwise`);
    profile/show 0.1 returns it through the shared view.
  - vetting/request 0.1, vtc/join-requests/decide 0.1, vtc/endorsements/issue
    0.1, vtc/vetting/vetters/grant 0.1, vtc/members/{credentials,rotate,renew}
    0.1, vtc/members/personhood/{assert 0.2,revoke 0.1} and the shared
    endorsement schema: the community-issued credential's `issuerScope` is
    `directed` or `public`, not `public` only.

  All changes are in place: an optional member and descriptions/prose on
  draft versions, which CONTRIBUTING-SPECS allows for a released version
  (check-published-schemas: additive only).

## 0.1.4 — 2026-11-24


### Fixed

- **build,rs,ts,ts-gen,go,dart**: Keep the free-standing $defs a specification declares in the request schema constant (#1353)

#1243 pruned every root $defs entry the request root does not reach. That also
  dropped definitions a specification declares for clients and references from
  nowhere, such as vtc/storage/credentials/set's SealedCredentialCleartext, which
  travels only inside a seal. VTI reads it, and the OpenAPI components its console
  uses, out of PAYLOAD_SCHEMA, and so stayed on trust-tasks-build 0.1.

  The request constant now also keeps each free-standing definition (one other
  than Response that no #/$defs/<name> reference anywhere in the schema names)
  and what it references. A shared definition spliced in from another document is
  always referenced, so it is still pruned when only the response uses it. The
  response document keeps only what it reaches.

  Applied in all seven generator copies and re-derived in check-bundles from
  defOrigins; regenerated, five specifications change in all four languages.

## 0.1.3 — 2026-11-19


### Specifications

- **vtc/rooms/community**: A community-owned room states and changes its signal delay (create 0.5, update 0.4, get 0.5) (#1336)

A room a member owns states its signal delay at rooms/create 0.3 and its
  administrators change it at the host with rooms/signal-delay/set 0.1. A
  room the community owns could do neither: the community is both the
  holder of `admin` and the host, and vtc/rooms/community/create and
  update carried no such member, so every community room waited a
  randomised few seconds before each rooms/changed signal however its
  members used it.

  - create 0.5 takes an optional `signalDelay` (absent = randomised),
    records it in the room's host record and always echoes it. A `none`
    create is audited with the create.
  - update 0.4 takes an optional `signalDelay`, applied to the host record
    with the other details, together or not at all; a change of the delay
    alone is a change (audited, details timestamps move) that needs no
    profile write. It touches no watch.
  - get 0.5 always serves the room's `signalDelay`.
  - Both writes refuse `signalDelayNotPermitted` when the community's
    governance as host keeps the delay everywhere, applying nothing.
  - A surface offering `none` SHOULD say what it gives up.

  Links to create 0.4, update 0.3 and get 0.4 move to the new versions.

## 0.1.2 — 2026-11-17


### Added

- **vta/dids**: List, get and methods for every identity a VTA holds; method-neutral context deletion (#1332)

A VTA holds identities of several methods, and the only way to ask which
  was once per method: vta/webvh/dids/list, vta/webs/dids/list. Every client
  asked only the methods it knew about, so a VTA whose identities were all
  did:webs reported none, and a new method would add a question to every
  client.

  - vta/dids/_shared/0.2: CONVENTIONS §20 (every identity: which are
    summarised, who sees what, the summary, methods), DidSummary and
    MethodInfo, a did:key profile, and a Summary details heading in each
    profile. §3.1's level-1 refusal is scoped to the key-role tasks.
  - vta/dids/list/0.1, get/0.1, methods/list/0.1.
  - vta/contexts/update-did/1.2: the DID must be one the VTA holds in that
    context (notHeld), and its class may be the context's (didClassConflict).
  - vta/contexts/delete/1.2: every DID in the subtree is deleted as its
    method's delete task deletes it, host work settled first. 1.1 named
    did:webvh only, so a did:webs record was removed while its key event log
    stayed published as a live identity.
  - vta/contexts/preview-delete/2.0: `dids` (every method) replaces
    `webvhDids`, which left did:webs out of the confirmation.
  - vta/webvh/dids/list/1.0 and vta/webs/dids/list/0.1 retired in favour of
    vta/dids/list/0.1.

  Bindings regenerated for all four languages.

- **vtc/storage/credentials/recipient**: Add vtc/storage/credentials/recipient/0.1 (#1329)

A room-hosting administrator's client asks the community for the key a
  vtc/storage/credentials/set sealed block is sealed to: the Ed25519 key it
  opens with, as its did:key, and the verification method in its DID
  document that publishes it.

  A client that cannot resolve and verify the community's DID could not
  seal at all. A browser console reading a did:webs would need KERI
  verification, and a hosted did:webvh would need a log from another
  origin. The VTC console read only the same-origin did.jsonl, which a
  did:webs community never serves. The answer comes over the same
  authenticated channel the credential then goes over, and is the same
  for every DID method. `notPublished` covers a key the community's
  document no longer lists, so nothing is sealed past a rotation.

  credentials/set 0.3's sealed-block definition and `related` now point
  at it (prose only).

## 0.1.1 — 2026-11-17


### Added

- **vtc/community/logo**: Add vtc/community/logo/update/0.1 and vtc/community/logo/show/0.1 (#1324)

* feat(vtc/community/logo): add vtc/community/logo/update/0.1

  An administrator hands the community a new logo, which the community
  re-encodes, strips of metadata and stores in place of the one it holds;
  `image: null` removes it. The answer is the stored re-encoding's digest,
  media type, size and dimensions.

