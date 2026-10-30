# Method profiles for key-role identities

A method profile states what the method-neutral key-role family
([`CONVENTIONS.md`](../CONVENTIONS.md)) leaves to the DID method, under the headings of its
§3.3, and nothing else. Everything a profile does not say is the family's.

| Profile | `DidMethodName` | Status |
|---|---|---|
| [`webvh.md`](webvh.md) | `webvh`, `scid:vh` | **Eligible** |
| [`webs.md`](webs.md) | `webs` | **Ineligible as the method is published** — criteria 4 and 6 of §3.2; the profile states the mapping that applies once the method meets them |

Ineligible by construction, with no profile needed (CONVENTIONS.md §3.2): `did:key`,
`did:jwk`, `did:peer` (every numalgo), `did:web`. A VTA refuses each with
`vta/dids:methodNotEligible`.

To add a method, write `<name>.md` under the nine headings below and, where it takes
parameters, `<name>.schema.json` defining `CreateParameters` and `UpdateParameters`
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
