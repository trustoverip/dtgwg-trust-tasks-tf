# Community voting — conventions for the voting, vtc/elections and vta/pcs/voter tasks

A community runs polls, surveys and elections among its members. A member proves only that
they may vote, never who they are, and anyone allowed to read the ballots can count them again.
The normative requirements are **VTI-VOT-001 – 100** of the Verifiable Trust Infrastructure
specification (*Community Voting* chapter); these tasks are its wire.

Three nodes take part, each with its own family:

| Node | Family | Role |
|---|---|---|
| A **voting service** (a host service) | `voting/*` | The **ballot box**: registers elections, admits casts, keeps each election's **board**, publishes the result. Never learns who voted. |
| The community's **VTC** | `vtc/elections/*` | The **registrar**: creates elections, issues **tickets**, verifies the board, certifies the result. Never handles a ballot. |
| The member's **VTA** | `vta/pcs/voter/*`, `vta/pcs/vote` | The **voter's agent**: holds the voter's key for each election, unblinds tickets, builds casts. The member's client carries every message. |

The tasks are:

| Node | Task | What it does |
|---|---|---|
| Ballot box | [`voting/election/register`](../../election/register/0.1/spec.md) | Registers an election the community signed |
| Ballot box | [`voting/election/show`](../../election/show/0.1/spec.md) | Reads an election's state, manifest digest and current head |
| Ballot box | [`voting/election/close`](../../election/close/0.1/spec.md) | The community closes voting early, where the manifest allows it |
| Ballot box | [`voting/election/cancel`](../../election/cancel/0.1/spec.md) | The community cancels an election before it closes |
| Ballot box | [`voting/ballot/cast`](../../ballot/cast/0.1/spec.md) | Admits a cast and answers with a signed receipt |
| Ballot box | [`voting/board/head`](../../board/head/0.1/spec.md) | The board's current signed head |
| Ballot box | [`voting/board/read`](../../board/read/0.1/spec.md) | A range of board entries |
| Ballot box | [`voting/board/inclusion`](../../board/inclusion/0.1/spec.md) | Proof that one entry is in a head |
| Ballot box | [`voting/board/consistency`](../../board/consistency/0.1/spec.md) | Proof that one head extends another |
| Ballot box | [`voting/result/show`](../../result/show/0.1/spec.md) | The result, once tallied |
| VTC | [`vtc/elections/create`](../../../vtc/elections/create/0.1/spec.md) | Creates an election and registers it with its ballot box |
| VTC | [`vtc/elections/list`](../../../vtc/elections/list/0.1/spec.md) | The community's elections a caller may see |
| VTC | [`vtc/elections/show`](../../../vtc/elections/show/0.1/spec.md) | One election: manifest, state, checkpoints, certified result |
| VTC | [`vtc/elections/close`](../../../vtc/elections/close/0.1/spec.md) | Closes voting early |
| VTC | [`vtc/elections/cancel`](../../../vtc/elections/cancel/0.1/spec.md) | Cancels an election |
| VTC | [`vtc/elections/certify`](../../../vtc/elections/certify/0.1/spec.md) | Verifies the board and certifies the result |
| VTC | [`vtc/elections/tickets/issue`](../../../vtc/elections/tickets/issue/0.1/spec.md) | Blind-issues one eligible party's ticket |
| VTA | [`vta/pcs/voter/register`](../../../vta/pcs/voter/register/0.1/spec.md) | Derives the election's voter key and builds the blinded ticket request |
| VTA | [`vta/pcs/voter/accept`](../../../vta/pcs/voter/accept/0.1/spec.md) | Unblinds the VTC's answer into the ticket |
| VTA | [`vta/pcs/vote`](../../../vta/pcs/vote/0.1/spec.md) | Validates a ballot against the manifest and builds the signed cast |
| VTA | [`vta/pcs/voter/receipt`](../../../vta/pcs/voter/receipt/0.1/spec.md) | Keeps a cast's receipt and checks its inclusion |

Each of those specifications references this file rather than restating it, and conformance is
anchored here. The wire shapes are in [`voting.schema.json`](voting.schema.json).

**Version 0.1 carries open elections only.** The sealed and tallied-encrypted levels need the
trustee tasks (joint key generation, partial decryption), which will join the family in their
own versions. Until then a ballot box refuses to register any other level
(`voting/election/register:unsupportedSecrecy`).

## 1. Terms

- **Election.** One poll, survey or election: one manifest, one roll, one board.
- **Manifest.** What an election asks and how it is decided (`Manifest`). Its **digest** is the
  SHA-256 multihash of its RFC 8785 (JCS) canonicalization.
- **Ticket.** A predicate credential system (PCS) class credential, issued blind by the
  community under the class `voter/<electionId>` and its **voting key**. It admits its holder to
  vote in one election and identifies nobody.
- **Cast.** A ballot together with the PCS attestation that admits it (`Cast`).
- **Tag.** The attestation's tag: the same for every cast made with one ticket in one election,
  unlinkable to the voter and across elections. The election's nullifier.
- **Board.** The election's append-only RFC 9162 log of what the ballot box admitted
  (`BoardEntry`), and its signed heads (`BoardHead`).
- **Receipt.** The ballot box's signed `voting/ballot/cast` response.

## 2. Separation

1. A VTC **MUST NOT** send, receive or relay a cast or a ballot under any task (VTI-VOT-001).
2. No task served by a ballot box has a member that carries a voter's identity, and a ballot
   box **MUST NOT** ask for one (VTI-VOT-002).
3. A `voting/ballot/cast` document **MUST** be issued and signed by a `did:key` generated for
   that cast alone (VTI-VOT-003), never by the voter's membership DID or any DID the voter uses
   elsewhere. The VTA generates it inside `vta/pcs/vote` and never stores it.
4. A ballot box **MUST** accept `voting/ballot/cast` over the HTTPS binding
   ([bindings/https](../../../../bindings/https/0.2/spec.md)) and **MAY** accept it over any
   other binding it serves (VTI-VOT-004). Every other task in these families follows the
   transport preference of its node.

## 3. Cryptographic binding

The suite is `ps-ddh-bls12381`: the PCS instance hidden vetting uses (BLS12-381, Σ-PS,
`Tag_DDH`), under the community's **voting** deployment, which is separate from its
hidden-vetting deployment.

1. **Deployment label.** `<community DID>#voting-pcs`, as UTF-8.
2. **Ticket class.** The root predicate whose label is `voter/<electionId>`, as UTF-8.
3. **Election point.** `id_E` is the suite's hash-to-G1 oracle with suffix `election`, under the
   domain `vti/vote/election-point/v1`, over `frame(community) ‖ frame(electionId) ‖
   frame(manifestDigest)`.
4. **Ballot context.** `app` is `"vti/vote/ballot/v1\0" ‖ frame(community) ‖ frame(electionId) ‖
   frame(manifestDigest) ‖ frame(JCS(ballot))`.
5. `frame(x)` is the length of `x` as an unsigned 64-bit big-endian integer followed by `x`.
   Strings are UTF-8. `manifestDigest` is framed as the 32 raw SHA-256 bytes, not as its
   multibase text.
6. **Cast.** `attestation` is `attest_in_context(votingKey, usk, voter/<electionId>, ticket,
   id_E, app)`; `tag` is its tag. Both are the suite's canonical compressed encoding, multibase
   base58btc.
7. **Admission.** A ballot box admits a cast only if the attestation's class encoding is that of
   `voter/<electionId>`, its `tag` equals `tag`, and `check_attestation_in_context(votingKey,
   id_E, attestation, app)` holds, with `app` rebuilt from the ballot **as received**.
8. **Board hashing.** A leaf hash is RFC 9162's `HASH(0x00 ‖ JCS(entry))`; interior nodes,
   heads and proofs are RFC 9162 §2.1 with SHA-256.

The reference implementation is the `vti-voting` crate of verifiable-trust-infrastructure.

## 4. Authorization

| Task | Authority |
|---|---|
| `voting/election/register`, `close`, `cancel` | The proven signer is the manifest's `community` |
| `voting/election/show`, `voting/result/show`, `voting/board/*` | Public board: anyone. Private board: the community, a DID in the manifest's `board.auditors` and, for `voting/board/inclusion` of one entry, the holder of that entry's `receiptToken` |
| `voting/ballot/cast` | Nobody's identity: the cast's attestation is the authority (§3.7) |
| `vtc/elections/create`, `close`, `cancel`, `certify` | `vtc.elections.manage` (VTI-VOT-080); `create` of a `binding` manifest takes another administrator's consent (VTI-VOT-081) |
| `vtc/elections/list`, `show` | A member, for an election whose roll admits them; `vtc.elections.manage` for every election |
| `vtc/elections/tickets/issue` | The proven signer is eligible under the roll rule and holds no ticket for the election (VTI-VOT-021, VTI-VOT-022) |
| `vta/pcs/voter/*`, `vta/pcs/vote` | Standing over the context; as a capability, **`pcsVoter`** (`device/_shared/0.2` `Capability`). Not `sign`; a VTA **MUST NOT** require `sign` |

Verifying the `issuer`, the transport identity or the `proof` establishes who asked, never that
they may ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)).

## 5. Custody (VTA)

1. A VTA **MUST NOT** return a voter key, a ticket, the blinding state of a ticket request, or
   the one-time key a cast is signed with, under any task.
2. A voter key is derived from a path allocated under the context's base for that election
   alone (VTI-VOT-023), re-derived on every use and never stored.
3. A ticket is kept wrapped like every other PCS secret of the engine, and discarded once the
   election is tallied or cancelled.

## 6. Privacy

1. Board entries carry no time of arrival (`BoardEntry`).
2. A ballot box **MUST NOT** record a cast's network address against its tag beyond the
   retention its operator discloses, and never after the election is tallied.
3. A VTA **SHOULD NOT** build a cast in the same exchange in which it accepted the ticket, and a
   client **SHOULD** let a random interval pass before sending it (VTI-VOT-033).
4. No audit record of a VTC, a VTA or a ballot box pairs a voter's identifier with a tag, a cast
   or a ballot (VTI-VOT-100).

## 7. Error codes

Each task declares its own codes with its slug. These local parts mean the same in every task
that declares them:

| Local part | Meaning |
|---|---|
| `electionUnknown` | No election with this identifier exists for this community, or the caller may not see it |
| `notOpen` | The operation needs a state the election is not in (voting not yet open, already closed, cancelled) |
| `classMismatch` | The attestation is not under this election's ticket class |
| `castInvalid` | The attestation does not verify for this election and this ballot |
| `ballotInvalid` | The ballot does not answer the manifest's questions as the manifest admits |
| `alreadyVoted` | A different ballot under this tag is counted and the manifest refuses revotes |
| `manifestMismatch` | A digest does not match the registered manifest |
| `unsupportedSecrecy` | The secrecy level is not served by this version |
| `notEligible` | The roll rule does not admit the requester |
| `alreadyIssued` | The requester already holds a ticket for this election |

Every other refusal uses the framework's standard codes (SPEC §8.3), such as
`permissionDenied` and `malformedRequest`. A repeat that conflicts with what a node already holds
has a task-specific code (`voting/election/register:alreadyRegistered`,
`vtc/elections/create:electionExists`, `vtc/elections/certify:alreadyCertified`,
`vta/pcs/voter/register:electionChanged`): the framework defines no standard `conflict`.

## 8. Retry safety

1. `voting/ballot/cast` is **idempotent**: a cast resent unchanged is answered with its first
   receipt and counted once (VTI-VOT-042).
2. `voting/election/register` is idempotent for the same `(community, electionId,
   manifestDigest)`; a registration of the same identifier with another digest is
   `voting/election/register:alreadyRegistered`.
3. `vtc/elections/tickets/issue` answers a repeat of the same signed request with the answer it
   first gave, and refuses any other request from a party that already holds a ticket
   (`alreadyIssued`).
