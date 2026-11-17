# Method profile: did:key

`DidMethodName`: `key`. This profile exists for CONVENTIONS.md §20 alone: a VTA mints
`did:key` identities — for a client's bootstrap identity (VTI-KEY-002), an integration, a
context's own identifier — and a caller asking what the VTA holds is owed them. A
`did:key` is never a key-role identity.

## 1. Eligibility

**Ineligible** for key roles: the identifier is the key, so it has no history, no change
authority, no commitment to a successor and no rotation (§3.2, criteria 1 – 6). A VTA
refuses `vta/dids/create` and `vta/dids/upgrade` for it with `vta/dids:methodNotEligible`,
`details.missing` `[1, 2, 3, 4, 5, 6]`.

Headings 2 – 9 do not apply.

## 10. Summary details

A VTA summarises a `did:key` it holds as a key record whose id is the DID's own
`did:key:<multibase>#<multibase>` form, and only while that key is active. A key record
naming another identity's verification method is that identity's key, not a `did:key`.

| Member | Value |
|---|---|
| `contextId` | The key record's context; absent where it has none. |
| `level` | `1`. |
| `primary` | Whether a context names the DID as its own. |
| `deactivated` | `false` — a `did:key` cannot be ended, only forgotten. |
| `serverId` | Absent: nothing is published. |
| `label` | The key record's label, where it has one. |
| `signingKeyId` | The key record's id. |
| `keyAgreementKeyId` | Absent. |
| `alsoKnownAs` | Empty. |
| `createdAt`, `updatedAt` | The key record's. |

`methodDetails` is absent: a `did:key` has nothing beyond the common members.
