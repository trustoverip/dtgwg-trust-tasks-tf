# provision/vta — family conventions

This document records the rules shared by every specification under
`provision/vta/`. Where it could conflict with a specific spec, **that spec's
own front matter and payload schema are authoritative.** How a Farm hands a
reservation to a wallet is the `vta-claim` trigger link of the Verifiable Trust
Infrastructure specification, chapter *Trigger Links* (VTI-LNK-110 to
VTI-LNK-117); this family does not restate it.

## 1. Roles and tasks

A **Farm** sets up Verifiable Trust Agents for people who have none and hands
them over. It shows a `vta-claim` trigger link on a page. A **claimant** (a
person's wallet) reads it and, after the person continues, claims the
reservation with a fresh key that becomes the VTA's administrator.

| Task | From → to | Signed by | Answered with |
|---|---|---|---|
| [`provision/vta/claim`](../../claim/0.1/spec.md) | claimant → Farm | administrator key `K_a`, `authentication` | `trust-task-next-step` expecting `redeem`; never a `#response` |
| [`provision/vta/redeem`](../../redeem/0.1/spec.md) | claimant → Farm | `K_a`, `authentication` | `pending`, `rejected`, a family code, or the terminal `#response` (the hand-over) |

The claim is the first request of the trigger link (VTI-LNK-054). It locks the
reservation to `K_a` and nothing more; the hand-over is the response to
`redeem`, which exists only once the person has confirmed the claim check on the
Farm's page and the key has been written into the VTA.

## 2. Shared schema components

[`vta.schema.json`](vta.schema.json) defines `Handle`, referenced by both tasks
([SPEC §6.6](/SPEC.md#66-shared-schema-components)).

## 3. Reservation state

A Farm keeps one record per reservation:

```
open → claimed → confirmed → provisioning → released
```

`rejected` and `lapsed` are final. Every change of state is a compare-and-set,
so only one caller can make it.

- `open`: the code is shown; the handle's window (`_exp`) is running.
- `claimed`: a valid claim locked the reservation to `K_a` (VTI-LNK-114). The
  Farm's page shows the claim check and the wallet shows its own (VTI-LNK-115).
  Nothing has been written into the VTA.
- `confirmed`: the person said the two checks match. The Farm writes `K_a` into
  the VTA as an administrator.
- `provisioning`: the VTA is being set up or restarted with the new key. A VTA
  that already runs passes through this state at once.
- `released`: the key is in the VTA and the VTA answers. The hand-over is
  available to `redeem`.
- `rejected`: the person said "Not mine". `K_a` is discarded and the VTA
  returns to the pool or stays as it was. Final.
- `lapsed`: the window passed before a claim, or the hold passed before a
  confirmation. `K_a`, if any, is discarded. Final.

A failed claim leaves the reservation as it was. `redeem` never changes state;
it reports it.

## 4. Rules for the whole family

1. **Keys.** The administrator key is an Ed25519 `did:key`, generated for one
   claim and used for no other VTA or exchange (VTI-LNK-113). A Farm refuses any
   other with `provision/vta:keyUnsupported`.
2. **One key per reservation.** Every document after the claim must be signed
   by `K_a`; anything else is `provision/vta:notClaimant`, with no details.
3. **Nothing before confirmation.** A Farm writes `K_a` nowhere, and hands over
   nothing, until the person confirms the claim check on the page (VTI-LNK-115;
   mandatory when adding a device, VTI-LNK-117). No reply before the terminal
   `redeem` response names the VTA as the claimant's.
4. **Proof purposes.** `K_a` signs with `authentication`. The Farm's next step
   and its `redeem` response are attestations the wallet relies on, and are
   signed with the Farm's `assertionMethod` key
   ([SPEC §4.7.3](/SPEC.md#473-proof-purpose-and-verification-relationship)).
5. **Times.** The handle's window is the trigger link's `_exp`, integer UTC
   epoch seconds, at most 300 s after the code is made (VTI-LNK-110). The hold
   after a claim is the Farm's choice; this family recommends 300 s.
6. **No caching.** Every response, including every error, is sent with
   `Cache-Control: no-store` where the transport has such a header.
7. **Generic failures.** A spent handle and a handle nobody reserved get codes
   that carry no details, so a second scanner learns nothing about the first.
8. **Rate limits.** A Farm applies per-network limits on `claim` and allows one
   open `redeem` poll per reservation. Refusals use the framework's
   `unavailable` with `retryAfter`.

## 5. Family error codes

These codes mean the same in every task that declares them, so they use the
family namespace ([SPEC §8.5](/SPEC.md#85-extension-by-individual-trust-task-specifications)
rule 2):

| Code | Meaning |
|---|---|
| `provision/vta:keyUnsupported` | The document's issuer is not an Ed25519 `did:key`. |
| `provision/vta:handleNotFound` | No reservation has this handle. |
| `provision/vta:handleExpired` | The handle's window passed before a claim, the hold passed before a confirmation, or the reservation has otherwise ended. |
| `provision/vta:notClaimant` | The document is not signed by the key that claimed the reservation. |

Task-specific codes (`alreadyClaimed`, `pending`, `rejected`) are declared on
the task they belong to.
