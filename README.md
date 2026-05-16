# Trust Tasks

A reference registry of **Trust Task** specifications — self-contained, transport-agnostic, JSON-based descriptions of the verifiable work that happens between parties.

Developed under the [Trust Over IP Foundation](https://trustoverip.org) (ToIP) Decentralized Trust Graph Working Group (DTGWG).

- **Live registry:** <https://trusttasks.org/registry>
- **Framework specification:** [`SPEC.md`](SPEC.md)
- **Individual specs:** [`specs/<slug>/<version>/`](specs/)
- **Contributor guide for spec authors:** [`CONTRIBUTING-SPECS.md`](CONTRIBUTING-SPECS.md)

## What is a Trust Task?

Two parties interoperate when they agree on the shape of the work they cooperate on. A *Trust Task* is the single, finite description of that work — a KYC handoff, a consent grant, an access-control change — captured as a JSON document that travels independently of the protocol carrying it.

Three properties make a Trust Task portable:

- **Self-contained.** The document carries everything needed to act on it: parties, criteria, schema, identifiers.
- **Transport-agnostic.** DIDComm, HTTPS, message queue, paper — the task is the task.
- **JSON-based.** The canonical serialization is a single JSON object validated against a published JSON Schema.

The framework that governs Trust Task documents — versioning, namespace, conformance rules, error responses, and the request/response convention — is defined in [`SPEC.md`](SPEC.md).

## Submitting a new Trust Task specification

The registry is organized as one folder per spec at `specs/<slug>/<MAJOR.MINOR>/`, mirroring the canonical *Type URI* (e.g., `https://trusttasks.org/spec/acl/grant/1.0` ↔ `specs/acl/grant/1.0/`). Each folder contains two files:

- `spec.md` — YAML front matter (the spec's normative declarations) followed by the human-readable specification.
- `payload.schema.json` — a JSON Schema 2020-12 document describing the `payload` member of conforming Trust Task documents.

To propose a new spec:

1. **Fork** this repository and create a working branch.
2. **Create your folder** under `specs/<slug>/<version>/`. The slug grammar (lowercase, hyphen-separated, optionally hierarchical) and reserved prefixes are defined in [SPEC.md §6.1](SPEC.md#61-type-uri).
3. **Add `spec.md`** with the required YAML front matter (slug, version, title, summary, status, target framework version, category, keywords, authors, parties, proof requirement; optional `errorCodes`, `related`) followed by your specification prose. Include a `## Request` section with at least one complete example *Trust Task document*, and — if your spec defines a success response — a `## Response` section with paired response examples. See `specs/acl/grant/1.0/spec.md` for a worked example.
4. **Add `payload.schema.json`** — `$id` set to your spec's Type URI; `$schema` set to `https://json-schema.org/draft/2020-12/schema`. Where you define a success response, place its sub-schema under `$defs.Response` with `$anchor: "response"` so the `#response` fragment on a response document's `type` resolves to the right shape.
5. **Validate locally:**
   ```sh
   npm install        # one-time
   npm run build      # parses front matter, validates schemas, regenerates the website registry
   ```
   `npm run validate` runs the same checks without touching the website tree.
6. **Commit with DCO sign-off** (`git commit -s`) and **open a pull request** against `main`. `CODEOWNERS` routes review to the editors responsible for your slug's namespace.

See [`CONTRIBUTING-SPECS.md`](CONTRIBUTING-SPECS.md) for the full authoring guide — slug grammar, version rules, the meta-schema, request/response schemas, error-code namespacing, and how a spec graduates from `draft` to `candidate` to `standard`.

## Building the registry locally

Requires Node 20 or later.

```sh
npm install
npm run build       # validate + generate website/assets/tasks.generated.js + copy specs/ → website/specs/
npm run validate    # validate only; no website writes
```

CI runs the build on every pull request (validation only) and on every push to `main` (build + deploy). See [`.github/workflows/deploy.yml`](.github/workflows/deploy.yml).

## Contributing

Contributions are governed by the ToIP Foundation's contribution process:

- Every commit **MUST** carry a DCO `Signed-off-by` trailer. Use `git commit -s`.
- By opening a pull request, contributors agree to the Open Web Foundation **Contributor License Agreement** in [`CONTRIBUTING.md`](CONTRIBUTING.md). That agreement grants the patent and copyright rights necessary to incorporate contributions into the published specification.
- Source code in this repository (`scripts/`, `website/`, build tooling) is contributed under the license in [`SOURCE_CODE.md`](SOURCE_CODE.md).

For framework-level changes (`SPEC.md`, the build pipeline, the registry website) the contributor guide is the present file plus [`CONTRIBUTING.md`](CONTRIBUTING.md). For individual Trust Task specifications, follow [`CONTRIBUTING-SPECS.md`](CONTRIBUTING-SPECS.md).

## Licensing

The published Trust Tasks specification is licensed under the Open Web Foundation **Final Specification Agreement** in [`LICENSE.md`](LICENSE.md). That license grants downstream implementers the patent and copyright rights needed to build conforming products against the specification. Source code in this repository is published under [`SOURCE_CODE.md`](SOURCE_CODE.md).

The distinction between [`CONTRIBUTING.md`](CONTRIBUTING.md) and [`LICENSE.md`](LICENSE.md) is the direction of the grant:

| File | Direction | What it covers |
|---|---|---|
| `CONTRIBUTING.md` | *Contributor → Working Group* | Rights you grant when you contribute to the specification. |
| `LICENSE.md` | *Working Group → Implementer* | Rights granted to anyone implementing the published specification. |
| `SOURCE_CODE.md` | Both directions, for code | License covering source code in this repository (tooling, website, scripts). |

## Getting involved

Join the community working on Trust Tasks and the wider verifiable-trust stack at <https://trustoverip.org/get-involved/membership/>.

Discussion, issues, and proposals live in this repository's [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).
