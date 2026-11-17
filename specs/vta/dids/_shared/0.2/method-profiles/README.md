# Method profiles for key-role identities

A method profile states what the method-neutral key-role family
([`CONVENTIONS.md`](../CONVENTIONS.md)) leaves to the DID method, under the headings of its
§3.3, and nothing else. Everything a profile does not say is the family's.

| Profile | `DidMethodName` | Status |
|---|---|---|
| [`webvh.md`](webvh.md) | `webvh`, `scid:vh` | **Eligible** |
| [`webs.md`](webs.md) | `webs` | **Eligible with the key-role designation extension** (proposed to the did:webs specification; reference implementation in `affinidi-did-webs` 0.13.1); ineligible without it — criteria 4 and 6 of §3.2 |
| [`key.md`](key.md) | `key` | **Ineligible** (§3.2: the identifier is the key). Listed and summarised (CONVENTIONS.md §20); never a key-role identity |

Ineligible by construction, with no profile needed (CONVENTIONS.md §3.2): `did:jwk`,
`did:peer` (every numalgo), `did:web` — and `did:key`, whose profile ([`key.md`](key.md))
exists only for the summary of §20. A VTA refuses each with
`vta/dids:methodNotEligible`.

To add a method, write `<name>.md` under the ten headings below and, where it takes
parameters, `<name>-parameters.schema.json` defining `CreateParameters` and `UpdateParameters`
(CONVENTIONS.md §18):

1. Eligibility
2. The change authority
3. Versions
4. Rendering
5. The cache horizon input
6. Publication
7. Recovery signing
8. Method parameters
9. Algorithms
10. Summary details (CONVENTIONS.md §18, §20.3)
