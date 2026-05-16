# Trust Tasks Specification

A specification developed under the [Trust over IP Foundation](https://trustoverip.org) Decentralized Trust Graph Working Group (DTGWG) Task Force.

## About Trust Tasks

Trust Tasks define the specifications and standards for achieving a particular task between two or more parties. A Trust Task is:

- **Self-contained** — A Trust Task contains all relevant information needed to complete the task within the Trust Task definition itself.
- **Transport-agnostic** — A Trust Task is agnostic as to how it is delivered or transported between the involved parties.
- **JSON-based** — Trust Tasks use JSON format as their data representation.

By decoupling the task definition from its delivery mechanism, Trust Tasks enable interoperability across different transport protocols and technology stacks while ensuring that every party involved has a complete, unambiguous description of what is required to fulfill the task.

## Specification

The framework specification — defining the Trust Task document structure, version scheme, namespace, conformance rules, and response types — is maintained in [`SPEC.md`](SPEC.md) in this repository.

Individual Trust Task specifications (e.g. `kyc-handoff`, `consent-receipt`) are conforming refinements of the framework and are addressable at `https://trusttasks.org/spec/<slug>/<MAJOR.MINOR>` under HTTP content negotiation.

## Editor's Copy

The latest rendered version of the registry is available at:
[https://trustoverip.github.io/dtgwg-trust-tasks-tf/](https://trustoverip.github.io/dtgwg-trust-tasks-tf/)

## Contributing

All Trust over IP Foundation Decentralized Trust Graph Working Group contributions are made under the following licenses:

- [Patent and Copyright Grants](CONTRIBUTING.md)
- [Source Code](SOURCE_CODE.md)

## Licensing

All Trust over IP Foundation Decentralized Trust Graph Working Group deliverables are published under the following licenses:

- [Patent and Copyright Grants](LICENSE.md)
- [Source Code](SOURCE_CODE.md)

## Getting Involved

Join a community of individuals and organizations solving the toughest technical and human-centric problems in digital trust. [https://trustoverip.org/get-involved/membership/](https://trustoverip.org/get-involved/membership/)
