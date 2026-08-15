# Trust Tasks

**Trust Over IP Foundation — DTGWG Task Force**

| | |
|---|---|
| **Document version** | 0.3 |
| **Date** | 2026-08-07 |
| **This version** | `https://trustoverip.github.io/dtgwg-trust-tasks-tf/SPEC.html` |
| **Latest published version** | None — this document has not yet been published as a Working Group Deliverable. |
| **Latest editor's draft** | <https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md> |
| **Editor** | [Glenn Gore](https://github.com/stormer78) |
| **Repository** | <https://github.com/trustoverip/dtgwg-trust-tasks-tf> |
| **Feedback** | <https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues> |
| **License** | See [LICENSE.md](LICENSE.md) and [SOURCE_CODE.md](SOURCE_CODE.md) |

---

## Abstract

This document defines the **Trust Tasks** framework: a specification for the verifiable work that occurs between two or more parties. A Trust Task is a self-contained, transport-agnostic, JSON-based description of an outcome that two parties agree to achieve. This framework specification defines the document structure, version scheme, namespace, and conformance requirements that every individual Trust Task specification — published under the registry at `https://trusttasks.org/` — is expected to satisfy. Individual Trust Task specifications (for example, the `kyc-handoff` specification at `https://trusttasks.org/spec/kyc-handoff/1.0`) are conforming refinements of this framework.

## Status of This Document

This section describes the status of this document at the time of its publication. Other documents may supersede this document.

This is a **Working Draft** prepared by the Trust Tasks Task Force of the Decentralized Trust Graph Working Group (DTGWG) of the [Trust Over IP Foundation](https://trustoverip.org). It has been produced by the editor listed above and has not yet been reviewed or endorsed by the DTGWG as a whole. Publication as a Working Draft does not imply endorsement by the Trust Over IP Foundation membership.

Comments on this document are welcome via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues). The editor expects substantive sections — in particular [§7. Minimum requirements](#7-minimum-requirements), [§8. Error responses](#8-error-responses), [§9. Transport bindings](#9-transport-bindings), and [§10. Security and Privacy Considerations](#10-security-and-privacy-considerations) — to evolve as individual Trust Task specifications progress through [§5.3 Maturity levels](#53-maturity-levels) and surface gaps in this framework.

This document is governed by the [Trust Over IP Foundation Patent and Copyright Grants](CONTRIBUTING.md).

---

## Table of Contents

1. [Introduction](#1-introduction)
2. [Terminology](#2-terminology)
3. [Conformance](#3-conformance)
4. [Trust Task documents](#4-trust-task-documents)
5. [Versioning](#5-versioning)
6. [Namespace](#6-namespace)
7. [Minimum requirements](#7-minimum-requirements)
8. [Error responses](#8-error-responses)
9. [Transport bindings](#9-transport-bindings)
10. [Security and Privacy Considerations](#10-security-and-privacy-considerations)
11. [Discovery and capability negotiation](#11-discovery-and-capability-negotiation)
12. [References](#12-references)
13. [Acknowledgments](#13-acknowledgments)
14. [Appendix A — Example Trust Task specification](#appendix-a--example-trust-task-specification)
15. [Appendix B — Changelog](#appendix-b--changelog)

---

## 1. Introduction

*This section is non-normative.*

Two parties interoperate when they agree on the shape of the work they cooperate on. Today that agreement is reached ad-hoc: every onboarding flow, every consent receipt, every credential exchange is described in a vendor-specific schema, carried over a vendor-specific protocol, and validated by vendor-specific code. The result is a combinatorial explosion of pairwise integrations.

A **Trust Task** is a single, finite description of an outcome between two parties — a KYC handoff, a consent grant, a payment commitment — that is portable across implementations because the task definition is decoupled from the transport that delivers it.

Three properties make a Trust Task portable:

1. **Self-contained** — the document carries everything needed to act on it: parties, criteria, schema, identifiers. No hidden context.
2. **Transport-agnostic** — the document makes no assumption about the protocol that delivers it. DIDComm, HTTPS, message queue, paper — the task is the task.
3. **JSON-based** — the canonical serialization is a single JSON object validated against a published JSON Schema.

The body of this framework specification defines the document structure, version scheme, and namespace shared by every individual Trust Task specification published under the registry.

### 1.1 Design goals

*This section is non-normative.*

The framework aims to solve four related problems that arise wherever two or more parties cooperate over a network.

1. **A common task vocabulary across any transport.** In a decentralized ecosystem there is no single message bus or RPC framework: parties speak DIDComm, HTTPS, message queues, paper, and anything else. *Trust Tasks* let two parties agree on *what* they are doing without first agreeing on *how* the bits move between them. The same task specification works regardless of the transport carrying it.

2. **Security, privacy, and identity that scale to the transport.** A *Trust Task document* can rely on the integrity, authentication, and party-identity guarantees already provided by the transport in use — for example, mutually-authenticated TLS or a signed DIDComm envelope — and where those guarantees are absent, the document's own `proof`, `issuer`, and `recipient` members ([§4.7](#47-proof), [§4.8](#48-the-issuer-and-recipient-members)) supply them in-band. Implementers can match cryptographic work to the threat model in front of them rather than always paying the worst-case cost.

3. **Payload freedom, declared at the boundaries.** The framework defines the outer document shape and deliberately leaves the `payload` unconstrained. Each *Trust Task specification* chooses its own payload structure, JSON Schema, and — where useful — JSON-LD context. The framework only requires that each choice be declared explicitly ([§7.3](#73-specification-requirements)) and be machine-validatable.

4. **A standard family of response types.** Many tasks need a structured way for a *recipient party* to report what happened. The framework reserves a small set of response-type *Trust Task specifications* addressing the common cases — failure ([§8](#8-error-responses)), success with metadata (`trust-task-ok`), and a recipient-suggested continuation (`trust-task-next-step`) — each itself a *Trust Task* so that one validation, signing, and transport pipeline serves both the task and its response. Failure and continuation are specified in this revision; `trust-task-ok` remains reserved (see [§8.6](#86-reserved-response-type-slugs)) and will be specified in a future revision.

## 2. Terminology

The key terms in this document are defined here. Where a term is *italicized* on subsequent use, the definition in this section applies.

* *Trust Task* — A unit of verifiable work between two parties, formally defined by a *Trust Task specification* and exchanged as *Trust Task documents*. Each instance of work — a KYC handoff, a consent grant, a payment commitment — is a *Trust Task*. The document model defined by this framework is bilateral: each *Trust Task document* names at most one *issuer* and one *recipient*. Exchanges involving more than two parties are modeled as multiple bilateral *Trust Tasks* linked by the framework's `threadId` member (see [§4.9](#49-the-threadid-member)).
* *Trust Task document* — A single JSON object that is an instance of a *Trust Task*. It carries its own type, identifier, and integrity metadata alongside a task-specific *payload*. The structure is defined normatively in [§4](#4-trust-task-documents).
* *Trust Task specification* — A document, conforming to this framework, that defines a single *Trust Task* — its slug, version, target framework version, parties, normative payload schema, proof requirement, and any task-specific error extensions. See [§7.3](#73-specification-requirements) for the full set of declarations a specification **MUST** publish.
* *Consequential Trust Task* — A *Trust Task* whose successful execution has an effect the *consumer* cannot silently undo: it alters *recipient party* state, releases confidential material, or exercises the subject's own authority. Precisely, a *Trust Task* is *consequential* where any of the following holds — `sideEffects.level` is `mutating` or `destructive`, `exposure.discloses` is `secret`, or `exposure.actsAsSubject` is `true` (see [§7.3](#73-specification-requirements) items 13 and 14). Where a *consumer* determines the authoritative values from the handler it is about to invoke rather than from the values the *Trust Task specification* declares, the authoritative values govern. An absent, unrecognized, or unresolvable declaration is *consequential*, consistent with the fail-safe defaults of [§7.3](#73-specification-requirements) items 13 and 14. The term names a class of task for which several rules in this specification are stricter than the general case; it carries no authorization meaning of its own (see [§7.2](#72-consumer-requirements) item 10).
* *Party* — An entity that participates in a *Trust Task*. Each party is identified by a *Verifiable Identifier*.
* *Verifiable Identifier (VID)* — A string identifier whose controller is verifiable under a trust framework. Decentralized Identifiers (DIDs) [[DID-CORE]] are one realization of VIDs; others include X.509 subjects, OIDC subject identifiers, and key thumbprints. The framework does not constrain the VID scheme; the *consumer*'s trust framework determines which schemes are accepted and how each is resolved to verification material.
* *Producer* — A *party* that emits a *Trust Task document*. Synonym: *issuer* when referring to the value carried in the document's `issuer` member.
* *Consumer* — A *party* that receives and processes a *Trust Task document*. Synonym: *recipient party* when emphasizing the consumer's acceptance role (for example, in error-response prose). The two terms refer to the same entity and are used interchangeably throughout this specification.
* *Document identifier* — The string carried in the `id` member of a *Trust Task document* that uniquely identifies that instance.
* *Thread identifier* — An optional string carried in the `threadId` member that correlates a *Trust Task document* with other documents belonging to the same logical exchange. See [§4.9](#49-the-threadid-member).
* *Parent thread identifier* — An optional string carried in the `parentThreadId` member that names the exchange containing this one, where a *Trust Task* is conducted inside a broader exchange. See [§4.9.2](#492-the-parentthreadid-member).
* *Trust Ceremony* — A flow composed of several *Trust Tasks*, optionally described by a published *ceremony definition*. Named here because a *Trust Task document* may record its membership of one; see [§4.11](#411-the-ceremony-member).
* *Ceremony definition* — The published, versioned description of a *Trust Ceremony*, identified by a URI in the `/ceremony/` subtree ([§6.7](#67-ceremony-namespace)). It is **not** a *Trust Task specification* and no document's `type` resolves to one. Its content is out of scope for this version.
* *Enactment* — One run of a *Trust Ceremony*, identified by a globally unique, non-reusable string carried in `ceremony.enactment`. An *enactment* is to a *ceremony definition* what a *Trust Task document* is to a *Trust Task specification*.
* *Payload* — The task-specific portion of a *Trust Task document*, carried in the `payload` member. Its internal structure is defined by the *Trust Task specification* identified by the document's `type`.
* *Type URI* — A URI that identifies a *Trust Task specification* at a specific version and serves as the single resolvable namespace for that version. The canonical form is defined in [§6.1](#61-type-uri).
* *Proof* — An optional integrity-providing object attached to a *Trust Task document*, in the form of a W3C *Data Integrity Proof* (see [§4.7](#47-proof)).
* *Error response* — A *Trust Task document* whose `type` resolves to the framework's reserved `trust-task-error` specification and that reports a failure with respect to a previously received *Trust Task document*. See [§8](#8-error-responses).
* *Transport binding* — A specification that defines how *Trust Task documents* are exchanged over a specific transport protocol, including how transport-derived identity, integrity, and freshness are mapped into framework members. See [§9](#9-transport-bindings).

## 3. Conformance

As well as sections marked as non-normative, all authoring guidelines, diagrams, examples, and notes in this specification are non-normative. Everything else in this specification is normative.

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY**, and **OPTIONAL** in this document are to be interpreted as described in BCP 14 [[RFC2119]] [[RFC8174]] when, and only when, they appear in all capitals, as shown here.

This specification defines the following conformance classes:

* A **conforming Trust Task specification** is a document that satisfies [§4. Trust Task documents](#4-trust-task-documents), [§5. Versioning](#5-versioning), [§6. Namespace](#6-namespace), and [§7. Minimum requirements](#7-minimum-requirements).
* A **conforming producer** is an implementation that emits *Trust Task documents* satisfying [§7.1](#71-producer-requirements).
* A **conforming consumer** is an implementation that processes *Trust Task documents* satisfying [§7.2](#72-consumer-requirements).

## 4. Trust Task documents

A *Trust Task document* is a single JSON object. The framework deliberately does **not** define a separate envelope: type, version, identifier, and integrity metadata are members of the document itself. This simplifies validation — one object, one schema composition — and removes the ambiguity of "is this field on the wrapper or the body?"

### 4.1 Encoding

A *Trust Task document* **MUST** be a JSON object as defined in [[RFC8259]]. The document **MUST** be encoded as UTF-8 without a byte-order mark.

### 4.2 Top-level members

A *Trust Task document* has the following top-level members.

| Member | Required | Type | Description |
|---|---|---|---|
| `id` | **MUST** | string | The *Document identifier* — a globally unique string for this instance of the task. UUIDv4 is **RECOMMENDED**; any uniquely-assignable string is permitted. See [§4.3](#43-the-id-member). |
| `threadId` | **MAY** | string | The *Thread identifier* — correlates this document with others in the same logical exchange (e.g. a response back to its originating request). See [§4.9](#49-the-threadid-member). |
| `parentThreadId` | **MAY** | string | The *Parent thread identifier* — the `threadId` of the exchange that contains this one, where this exchange is conducted inside another. See [§4.9.2](#492-the-parentthreadid-member). |
| `ceremony` | **MAY** | object | Records that this document is a step of a *Trust Ceremony* — a flow composed of several *Trust Tasks*. See [§4.11](#411-the-ceremony-member). |
| `type` | **MUST** | string (URI) | The *Type URI* identifying the *Trust Task specification* and version this document conforms to. See [§4.4](#44-the-type-member). |
| `issuer` | **MAY** | string (VID) | A *Verifiable Identifier* identifying the *party* responsible for the document's content. See [§4.8](#48-the-issuer-and-recipient-members). |
| `recipient` | **MAY** | string (VID) | A *Verifiable Identifier* identifying the *party* the *issuer* expects to act upon the document. See [§4.8](#48-the-issuer-and-recipient-members). |
| `issuedAt` | **SHOULD** | string (date-time) | An [[RFC3339]] timestamp recording when the document was produced. |
| `expiresAt` | **MAY** | string (date-time) | An [[RFC3339]] timestamp after which the document is no longer valid **to accept**. Where `expiresAt` is specified, the *recipient party* **MUST** honor the expiry: a *consumer* **MUST NOT** act upon a document for which `now ≥ expiresAt` (inclusive bound; the instant `expiresAt` is itself treated as expired). A *consumer* **MAY** apply a small clock-skew tolerance, typically ≤ 60 seconds, when evaluating this comparison. `expiresAt` bounds **acceptance**, not execution: it does not abort work already under way (see [§7.2](#72-consumer-requirements) item 12). See [§7.2](#72-consumer-requirements). |
| `payload` | **MUST** | object | The task-specific body. Its internal structure is governed by the *Trust Task specification* identified by `type`. See [§4.5](#45-the-payload-member). |
| `@context` | **MAY** | string \| array \| object | If present, enables JSON-LD processing of the document. See [§4.6](#46-json-ld-compatibility). |
| `proof` | **MAY** | object | An optional integrity proof. See [§4.7](#47-proof). |

A *Trust Task document* **MAY** contain additional top-level members beyond those listed above. Member names beginning with `x-` are **RESERVED** for experimental extensions and **MUST NOT** be used in a published *Trust Task specification*.

> **Example 1 — A complete Trust Task document** *(non-normative)*
>
> ```json
> {
>   "id": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
>   "type": "https://trusttasks.org/spec/kyc-handoff/1.0",
>   "issuer": "did:web:verifier.example",
>   "recipient": "did:web:bank.example",
>   "issuedAt": "2026-04-12T09:31:00Z",
>   "expiresAt": "2027-04-12T09:31:00Z",
>   "payload": {
>     "subject": "did:key:z6Mk...",
>     "result": "passed",
>     "level": "LOA2"
>   }
> }
> ```
>
> The `payload` member is the only part whose internal shape is defined by the per-task specification; everything else is framework-defined. This example carries no `proof` member; it therefore assumes delivery over a transport that provides end-to-end integrity and authentication between *producer* and *consumer*. A document delivered over a less protective transport, or one expected to be relied upon by third parties beyond the original *consumer*, would carry a `proof` member (see [§4.7.1](#471-when-to-include-a-proof)).

### 4.3 The `id` member

The `id` member's value **MUST** be a string that is globally unique to this instance of the task. The framework places no further constraint on its form: UUIDv4 [[RFC9562]] is **RECOMMENDED** as a low-coordination default that requires no namespace ownership, but any string an implementation can guarantee unique is permitted — for example, a DID URL, a UUIDv7, a URN, or an opaque content-addressed identifier. Producers **MUST NOT** reuse an `id` value across documents.

The `id` is opaque to the framework. Resolvability of the `id` (the ability to dereference it back to the document) is not required. Where resolvability is needed for a particular use case, an individual *Trust Task specification* or transport binding **MAY** require a more specific form (for example, a DID URL).

### 4.4 The `type` member

The `type` member's value **MUST** be a *Type URI* in the form defined in [§6.1](#61-type-uri). The version of the *Trust Task specification* a document conforms to is conveyed by the trailing `<MAJOR.MINOR>` segment of this URI; no separate version member is carried in the document.

A `type` URI **MAY** carry a fragment identifier. The framework reserves the fragments `#request` and `#response` to disambiguate the two directions of a request/response exchange that share a single *Trust Task specification*; see [§4.4.1](#441-request-and-response-variants).

#### 4.4.1 Request and response variants

A single *Trust Task specification* often describes both a *request* document (the document a *producer* sends to initiate the task) and a *response* document (the document the *consumer* returns when the task completes successfully). The framework distinguishes the two via the fragment of the `type` URI:

| `type` form | Meaning |
|---|---|
| `https://trusttasks.org/spec/<slug>/<MAJOR.MINOR>` | Request document. Implicitly equivalent to the explicit form below. |
| `https://trusttasks.org/spec/<slug>/<MAJOR.MINOR>#request` | Request document, explicit form. |
| `https://trusttasks.org/spec/<slug>/<MAJOR.MINOR>#response` | Success-response document for the same specification. |

The rules:

1. A *Trust Task document* whose `type` URI carries **no fragment** or the fragment `#request` is a *request*. The two forms are semantically equivalent; producers **MAY** emit either, consumers **MUST** accept both.
2. A *Trust Task document* whose `type` URI carries the fragment `#response` is the *success response* of a request whose `type` is the same URI with the fragment stripped. The request and response are correlated by `threadId` per [§4.9](#49-the-threadid-member).
3. The fragments `#request` and `#response` are **RESERVED** for this purpose. An individual *Trust Task specification* **MUST NOT** assign other fragment meanings to its `type` URI.
4. A *failure* response is **not** a `#response`-variant document of the request's *Type URI*. Failures are reported via the framework's distinct `trust-task-error` *Type URI* per [§8](#8-error-responses).
5. Consumers **MUST** preserve the fragment when comparing `type` URIs, when routing documents internally, and when keying hash maps on `type`. A consumer that strips the fragment before keying will conflate request and response documents.
6. The payload JSON Schema for a request/response pair is published as a single schema document at the bare *Type URI* (no fragment). Within that schema, the request payload shape is the top-level schema (or the schema reachable via `$anchor: "request"`); the response payload shape is reachable via `$anchor: "response"`. See [§7.3](#73-specification-requirements) for the publishing requirements.

A specification that defines a fire-and-forget task — one with no expected success response document — declares no response sub-schema. Its consumers signal success implicitly (by the absence of a `trust-task-error` reply) and **MUST NOT** emit a `#response`-variant document for that specification.

### 4.5 The `payload` member

The `payload` member's value **MUST** be a JSON object whose internal structure is defined by the *Trust Task specification* identified by the document's `type`. This framework places no constraint on the contents of `payload` beyond requiring that it be an object.

The framework separates document-level metadata (`id`, `threadId`, `type`, `issuer`, `recipient`, `issuedAt`, `expiresAt`, `proof`) from task-specific data (`payload`) so that a single framework-level schema validates the outer structure, with per-task schemas applied only to `payload`. Schema scope is defined in [§6.3](#63-schema-scope).

#### 4.5.1 The `ext` extension member

A *Trust Task specification* **MAY** allow an `ext` member at the top level of `payload`, at any nested object whose contents the specification controls, or both. The `ext` member is the framework's sanctioned extension point for ecosystem-defined data that the base specification does not enumerate.

The framework reserves the following normative rules for any `ext` member, in any specification:

1. `ext` **MUST** be a JSON object when present.
2. Each *immediate* key of `ext` **MUST** match the reverse-DNS grammar `^[a-z][a-z0-9-]*(\.[a-z0-9-]+)+$` — lowercase, at least one dot. Examples: `vnd.affinidi.webvh`, `org.example.acl`. Bare keys without a namespace are non-conforming.
3. The structure under each namespace is opaque to the framework. Producers MAY place any JSON value the namespace's controller chooses to define.
4. A *producer* **MUST NOT** rely on any framework-level meaning for the contents of any `ext.*` namespace.
5. A *consumer* **MUST** ignore namespaces it does not recognize, consistent with the unrecognized-member rule of [§7.2](#72-consumer-requirements). A *consumer* **MAY** require its own namespace as a matter of local policy and reject documents lacking that namespace with `malformedRequest`.
6. The framework reserves **no** `ext.*` namespace today. *Trust Task specifications* **MUST NOT** define cross-specification semantics for any `ext` key; ecosystem semantics belong to the namespace controller.

A *Trust Task specification* opts into `ext` at a given object level by including a property named `ext` (typically a `$ref` to the framework's published `Ext` `$def`) and adjusting that level's `additionalProperties` declaration accordingly. Specifications that do not include `ext` at a given level reject the member at that level under their existing `additionalProperties: false`.

The signed envelope covers `ext` in the same way it covers any other member of `payload`, so `ext` inherits the integrity guarantees of [§4.7](#47-proof) when a `proof` is present.

`ext` is distinct from the task-specific `details` member of a `trust-task-error` response ([§8.5](#85-extension-by-individual-trust-task-specifications)). `details` carries structured data tied to a specific error `code` defined by the spec author; `ext` carries vendor-namespaced extension data defined by the ecosystem. Both members **MAY** appear on the same document and are not interchangeable.

### 4.6 JSON-LD compatibility

A *Trust Task document* **MAY** include an `@context` member. If present, the document **MUST** be processable as JSON-LD; the framework places no further constraint on the contents of `@context` beyond requiring it to be a string, an array of strings or objects, or an object, in line with the JSON-LD specification. A *Trust Task specification* that wishes to declare a canonical JSON-LD context **MUST** publish it at its *Type URI* under content negotiation for `application/ld+json` (see [§6.2](#62-content-negotiation)).

A *consumer* that does not implement JSON-LD processing **MUST** ignore the `@context` member and process the document as plain JSON. JSON-LD support is therefore strictly additive — no consumer is required to implement it, and no document is required to include it.

> **Example 2 — A Trust Task document with a JSON-LD context** *(non-normative)*
>
> ```json
> {
>   "@context": [
>     "https://www.w3.org/ns/credentials/v2",
>     "https://trusttasks.org/spec/kyc-handoff/1.0"
>   ],
>   "id": "urn:uuid:7d8b1e3a-9a72-4f86-9d04-2a4b6c2c5e10",
>   "type": "https://trusttasks.org/spec/kyc-handoff/1.0",
>   "issuer": "did:web:verifier.example",
>   "recipient": "did:web:bank.example",
>   "issuedAt": "2026-04-12T09:31:00Z",
>   "payload": {
>     "subject": "did:key:z6Mk...",
>     "result": "passed",
>     "level": "LOA2"
>   }
> }
> ```
>
> A *consumer* that implements JSON-LD processes the document accordingly; a *consumer* that does not implement JSON-LD ignores `@context` and processes the same document as plain JSON. The two interpretations validate against the same payload schema.

### 4.7 Proof

A *Trust Task document* **MAY** include a `proof` member whose value is a W3C *Data Integrity Proof* object as defined in [[VC-DATA-INTEGRITY]]. When present, the `proof` binds the document's content to its `issuer`.

The choice of cryptographic suite is open: any suite registered by the W3C Verifiable Credential Working Group (for example, `eddsa-rdfc-2022` or `ecdsa-rdfc-2019`, or any future suite) **MAY** be used. The `verificationMethod` of the proof **MUST** resolve to verification material controlled by the *party* identified by the document's `issuer` member (see [§4.8](#48-the-issuer-and-recipient-members)).

When `proof` is present, it covers the document with `proof` itself excluded from the signed content, per the canonicalization rules of the chosen Data Integrity suite.

#### 4.7.1 When to include a proof

The default rules governing the presence of `proof` in a *Trust Task document* are:

* If the document is delivered over a transport that already provides end-to-end integrity and authentication between *producer* and *consumer* — for example, mutually-authenticated TLS or a signed DIDComm envelope — `proof` **MAY** be omitted.
* If the document is delivered over a transport that does not provide such guarantees, or where tampering or substitution by intermediaries is possible, `proof` **SHOULD** be included.
* If a strong, transport-independent guarantee of non-tampering and of *producer* identity is required — typically because the document is intended to be retained, replayed, or relied on by parties beyond the original *consumer* — `proof` **MUST** be included.

Whenever `proof` is included, the audience-binding rule of [§4.8.2](#482-audience-binding) also applies: the *producer* commits to an in-band `recipient` so that the proof binds not only the content but also the intended audience.

An individual *Trust Task specification* **MAY** strengthen these defaults (for example, mandate `proof` regardless of transport) but **MUST NOT** weaken them. The declaration each *Trust Task specification* makes about its own `proof` requirement is governed by [§7.3](#73-specification-requirements).

### 4.8 The `issuer` and `recipient` members

A *Trust Task document* **MAY** identify the parties involved by including the `issuer` and `recipient` members at the top level of the document.

* `issuer` — a *Verifiable Identifier* (see [§2](#2-terminology)) identifying the *party* responsible for the document's content. When `proof` is present, the `issuer` **MUST** identify the entity to which the proof's `verificationMethod` resolves.
* `recipient` — a *Verifiable Identifier* identifying the *party* the *issuer* expects to act upon the document.

The framework does not constrain the VID scheme used: a DID, an X.509 subject, an OIDC subject identifier, a key thumbprint, or any other identifier whose controller is verifiable under the *consumer*'s trust framework is acceptable.

A *VID* is compared by exact string equality wherever this framework requires a VID-to-VID comparison (notably the in-band-vs-transport cross-check in [§4.8.1](#481-precedence-of-in-band-over-transport-derived-identity), the recipient-enforcement rule in [§7.2](#72-consumer-requirements) item 5, and the proof-binding rule in [§4.7](#47-proof)). *Producers* **SHOULD** emit *VID*s in their canonical form for the scheme in use — no leading or trailing whitespace, no normalization of case-sensitive segments, and (for schemes that admit equivalent forms) the form that the scheme's authority designates as canonical. A *consumer* **MAY** reject a *Trust Task document* whose `issuer`, `recipient`, or any *VID*-typed `payload` member is not in canonical form with `malformedRequest`; a *consumer* that accepts non-canonical input **MUST NOT** silently normalize before applying any framework rule that compares the value — normalization changes the string, and the framework's comparisons are over the unchanged bytes.

Both members are **OPTIONAL**. Their purpose is to let the parties be identified in-band where the transport in use does not already convey strong, authenticated party identity — for example, an unauthenticated HTTP POST, a public message queue, or paper hand-off.

Where a secure transport already conveys authenticated party identity (such as mutually-authenticated TLS or a signed DIDComm envelope), these in-band members **MAY** be omitted.

#### 4.8.1 Precedence of in-band over transport-derived identity

The framework treats the in-band `issuer` and `recipient` members as **authoritative** for party identity. Specifically, for each party:

1. **If the in-band member is present**, its value is the party identity that the *consumer* **MUST** apply for every subsequent framework rule that references that party — including, but not limited to, `proof` verification (where applicable, see [§4.7](#47-proof)), recipient enforcement (see [§7.2](#72-consumer-requirements), item 5), and *Trust Task specification* requirements that reference the party. The transport-derived identity is, in this case, **only a cross-check**: where both an in-band identity and a transport-derived identity are present for the same party, they **MUST** be consistent, and a *consumer* **MUST** treat a mismatch as a validation failure (see [§7.2](#72-consumer-requirements), item 6).
2. **If the in-band member is absent**, a *consumer* **MAY** derive the party identity from the transport — typically via the *transport binding* in use (see [§9](#9-transport-bindings)) — and **MAY** treat the derived value as if it had been carried in-band for the purposes of subsequent rules. A *consumer* with no in-band value and no transport-derived value for a party that the *Trust Task specification* declares as **REQUIRED** **MUST** reject the document.

In short: the document is the source of truth for who the parties are. The transport, when it provides authenticated identity, is used either to fill in what the document omits, or to verify what the document asserts — never to override it.

An individual *Trust Task specification* **MAY** require either or both members to be present — for example, to support audit, third-party replay, or forwarding — but **MUST NOT** prohibit a *consumer* from comparing them with transport-derived identity.

> **Example 3 — A Trust Task document using non-DID *Verifiable Identifiers*** *(non-normative)*
>
> ```json
> {
>   "id": "urn:uuid:0e9d4c2b-5f81-4d3e-9b51-7a3c89e3d1f2",
>   "type": "https://trusttasks.org/spec/kyc-handoff/1.0",
>   "issuer": "x509:CN=Verifier,O=Example Verifier Ltd,C=NL",
>   "recipient": "x509:CN=Bank,O=Example Bank,C=NL",
>   "issuedAt": "2026-04-12T09:31:00Z",
>   "payload": {
>     "subject": "oidc:https://issuer.example/sub#user-94217",
>     "result": "passed",
>     "level": "LOA2"
>   }
> }
> ```
>
> Here `issuer` and `recipient` are X.509 subject distinguished names and `payload.subject` is an OIDC subject identifier. The framework treats any string identifier whose controller is verifiable under the *consumer*'s trust framework as a valid *VID*; DIDs are one realization among several.

#### 4.8.2 Audience binding

When a *Trust Task document* carries a `proof` member, the document **MUST** also carry an in-band `recipient` member, unless the *Trust Task specification* identified by the document's `type` declares itself a *bearer specification* (see [§4.8.3](#483-bearer-specifications)).

This rule exists because a *Data Integrity Proof* covers the signed bytes — the *issuer*, *payload*, and other framework members — but does **not** cover any transport-derived identity. A document signed without an in-band `recipient` therefore provides no cryptographic binding between the *producer*'s assertion and the intended audience: an attacker who obtains the document — from a *consumer*'s storage, an intermediate cache, or an exfiltration — can replay the bytes to a different *consumer* without any signal that the original *producer* did not intend that audience to act upon them. A consumer receiving such a replayed document would otherwise verify the proof successfully, observe that no `recipient` constrains the assertion, and apply the producer's claim to its own context.

A *consumer* receiving a `proof`-carrying document with no in-band `recipient`, where the originating *Trust Task specification* is not a *bearer specification*, **MUST** reject the document with a `malformedRequest` *error response* (see [§8](#8-error-responses)).

Specifications that declare `proof` as **REQUIRED** (see [§7.3](#73-specification-requirements) item 8) implicitly require `recipient` in-band for all non-bearer cases; the audience-binding rule and the proof requirement combine to ensure the document is self-contained for both producer identity and intended audience.

#### 4.8.3 Bearer specifications

A *Trust Task specification* whose `payload` carries an assertion meaningful to any *consumer* that can verify the *producer*'s identity — for example, a public attestation, a heartbeat, or a schema-publication announcement — **MAY** opt out of the audience-binding rule of [§4.8.2](#482-audience-binding) by declaring itself a *bearer specification*. The opt-out is published in the specification's front matter (see [§7.3](#73-specification-requirements) item 12).

A *bearer specification* makes an explicit, normative claim that documents conforming to it are intended for unspecified consumption: any party that can verify the document's `proof` (where present) is a legitimate recipient.

A *bearer specification* **MUST**:

1. Declare `bearer: true` in its front matter.
2. Declare its `recipient` party requirement as **OPTIONAL** (the audience-binding rule no longer applies).
3. State in its prose what assertion the document conveys and why audience binding is inappropriate for it.

A *bearer specification* **SHOULD NOT** carry any field in `payload` whose interpretation depends on the receiving party's identity (for example, "balance owed *to you*"); such fields belong in audience-bound specifications.

The default for any *Trust Task specification* is **non-bearer**. Specifications **MUST NOT** declare themselves bearer unless the audience-free property is intrinsic to the assertion they publish.

### 4.9 The `threadId` member

Every *Trust Task document* carries its own unique `id` ([§4.3](#43-the-id-member)); a response document **MUST NOT** reuse the `id` of the document it is responding to. Correlating documents back to one another — for example, linking a response to its originating request — is the purpose of the `threadId` member, not the `id` member.

A *Trust Task document* **MAY** include a `threadId` member that correlates it with other *Trust Task documents* belonging to the same logical exchange — for example, a request and its response, or a request, an intermediate `trust-task-next-step` response, and the final result.

A *producer* that emits a *Trust Task document* in response to another *Trust Task document* **SHOULD** set `threadId` to the value of the originating document's `threadId`. If the originating document carried no `threadId`, the *producer* **SHOULD** set `threadId` to the value of the originating document's `id`. The effect of this convention is that every document in a logical exchange carries the same `threadId`, and that value can always be traced back to the `id` of the document that started the thread.

The framework places no constraint on the form of a `threadId` beyond requiring it to be a string. Producers initiating a new exchange **MAY** omit `threadId` entirely (single-shot tasks need no thread), **MAY** mint a fresh value (e.g. a UUID), or **MAY** reuse the document's own `id`.

`threadId` carries no normative validation semantics. *Consumers* **MUST NOT** reject a document on the basis of `threadId` alone, but **MAY** use it for routing, correlation, aggregation, or audit.

#### 4.9.1 Naming an exchange from outside the framework

A `threadId` names one exchange and expresses no relationship to any other. Exchanges nest in practice — a *Trust Task* conducted to complete a step of some broader interaction is still its own exchange, with its own `threadId`. The optional `parentThreadId` member ([§4.9.2](#492-the-parentthreadid-member)) records that containment, but it is a navigation aid: it does not change which exchange attests an event, and the rule below holds whether or not it is present.

This matters whenever something outside the framework refers to an exchange as evidence that an event occurred: a credential that cites the exchange which established what it attests, an audit record, a governance decision that turns on some task having been performed. Nesting makes the reference ambiguous, because more than one thread was open when the event happened, and only one of them attests it.

The rule is that such a reference **MUST** name the *innermost* exchange whose documents attest the event being cited, and **MUST** name it by the `id` of the document that initiated that exchange — the value every document in the thread traces back to under the convention above ([§4.3](#43-the-id-member) makes that `id` globally unique and non-reusable, which a `threadId` is not required to be).

Naming an enclosing exchange instead collects evidence of the wrong event. Where a witnessing ceremony is conducted inside a broader relationship exchange, for example, only the ceremony's own response attests that the witnessing took place; the enclosing exchange's response attests the relationship interaction and says nothing about the witnessing. A consumer verifying the outer reference would conclude something the documents do not support.

#### 4.9.2 The `parentThreadId` member

A *Trust Task document* **MAY** include a `parentThreadId` member whose value is the `threadId` of the exchange that contains this one. Its purpose is navigation: it lets a party holding a document from the inner exchange find the exchange it was conducted within, which a flat `threadId` cannot express.

The member takes the same posture as `threadId` ([§4.9](#49-the-threadid-member)):

* A *producer* that emits a *Trust Task document* within an enclosing exchange **SHOULD** set `parentThreadId` to that exchange's `threadId`, and **SHOULD** carry the same value onto every document of the inner exchange — including its *response* and any *error response* — since the whole exchange shares one parent.
* A *producer* **MUST NOT** set `parentThreadId` equal to the document's own `threadId`. An exchange cannot contain itself.
* `parentThreadId` carries no normative validation semantics. *Consumers* **MUST NOT** reject a document on the basis of `parentThreadId` alone, but **MAY** use it for routing, correlation, aggregation, or audit.

The member records **one** level of containment. Reconstructing a deeper ancestry requires the intervening documents, and the framework defines no representation for a full chain; a specification needing one is better served by an explicit payload structure than by inferring it from thread metadata.

Where the transport carries its own parent-thread concept, the two **MUST** agree when both are present, and the in-band member remains authoritative for framework-level processing; see [§9.1](#91-what-a-transport-binding-specifies). A transport binding that maps the two states the rule for its own protocol.

> **Example 4a — A ceremony nested inside a broader exchange** *(non-normative)*
>
> A relationship exchange is under way on thread `9b1d…`. Completing it requires a witnessing ceremony, which is its own *Trust Task* exchange with its own thread:
>
> ```json
> {
>   "id": "urn:uuid:2c7f5e10-6a4b-4f8e-9d31-0b6a2f4c8e15",
>   "type": "https://trusttasks.org/spec/webvh/witness/publish/0.1",
>   "threadId": "urn:uuid:4a0e2b77-88c1-4d55-9f2a-6c3d1e5b7a92",
>   "parentThreadId": "9b1d3f60-52a8-4c17-8e44-1d9c7b05f3ae",
>   "issuer": "did:web:witness.example",
>   "recipient": "did:web:host.example",
>   "issuedAt": "2026-08-08T10:15:00Z",
>   "payload": { "…": "…" }
> }
> ```
>
> A credential citing the witnessing as evidence anchors to this inner exchange, per [§4.9.1](#491-naming-an-exchange-from-outside-the-framework) — the enclosing exchange attests the relationship interaction, not the witnessing. The `parentThreadId` is what lets a holder of this document find that enclosing exchange; it is not what the citation names.

> **Example 4 — Request and response correlated by `threadId`** *(non-normative)*
>
> A *producer* issues an initiating *Trust Task document*:
>
> ```json
> {
>   "id": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
>   "type": "https://trusttasks.org/spec/kyc-handoff/1.0",
>   "issuer": "did:web:verifier.example",
>   "recipient": "did:web:bank.example",
>   "issuedAt": "2026-04-12T09:31:00Z",
>   "payload": { "subject": "did:key:z6Mk...", "result": "passed", "level": "LOA2" }
> }
> ```
>
> The original document carried no `threadId`, so the responding *party* sets `threadId` to the originating document's `id`:
>
> ```json
> {
>   "id": "8a91c7b3-2e62-4a91-a3a4-9d61b75e2f01",
>   "type": "https://trusttasks.org/spec/trust-task-error/0.2",
>   "threadId": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
>   "issuer": "did:web:bank.example",
>   "recipient": "did:web:verifier.example",
>   "issuedAt": "2026-04-12T09:33:00Z",
>   "payload": { "code": "proofRequired", "retryable": false }
> }
> ```
>
> Both documents now share `threadId = 4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2`; any subsequent document in this exchange — for example, a retry with a fresh `id` and a valid `proof` — would carry the same `threadId`.

### 4.10 Naming conventions

JSON member names and enumerated string values in *Trust Task documents* follow the casing rules below, so that documents are consistent across specifications both for human readers and for code generators.

1. **Framework-defined members.** Every member defined by this framework — `id`, `threadId`, `parentThreadId`, `type`, `issuer`, `recipient`, `issuedAt`, `expiresAt`, `payload`, `proof`, and the members of the error payload in [§8.2](#82-error-payload) — **MUST** be named in **lowerCamelCase**. The sole exception is `@context`, which is named as required by JSON-LD.

2. **Framework-defined values.** Enumerated string values defined by this framework — notably the standard error `code` identifiers of [§8.3](#83-standard-error-codes) — **MUST** be expressed in **lowerCamelCase**.

3. **Payload member names.** A *Trust Task specification* **SHOULD** name the members of its `payload` in lowerCamelCase. A specification **MAY** deviate only where it embeds a member whose name is fixed by an external vocabulary (for example, a field copied verbatim from a WebAuthn or JOSE structure), and it **SHOULD** confine such foreign naming to the embedded sub-object.

4. **Specification-defined values.** String values drawn from a closed set that a *Trust Task specification* itself defines — statuses, kinds, decisions, event types, extended error `code` identifiers — **SHOULD** be expressed in lowerCamelCase (for example, `cacheAndKeys`, `stepUp`, `proofInvalid`).

5. **Externally-owned values.** A value whose canonical form is fixed by an external specification **MUST** be carried verbatim and **MUST NOT** be re-cased, because the framework compares such values by exact string equality (see [§4.8](#48-the-issuer-and-recipient-members)). Examples include WebAuthn enumerations (`public-key`, `cross-platform`), JOSE algorithm identifiers (`EdDSA`, `ES256`), cookie `SameSite` values (`Lax`, `Strict`), and W3C *Data Integrity* type and purpose values (`DataIntegrityProof`, `assertionMethod`).

6. **Out of scope.** This section does not govern *slugs* (lowercase, hyphen-separated; [§6.1](#61-type-uri)) or `ext` namespace keys (reverse-DNS; [§4.5.1](#451-the-ext-extension-member)); each retains its own grammar.

A change to the casing of an existing member name or specification-defined value is a breaking change and follows the versioning rules of [§5](#5-versioning); the re-casing introduced in framework version 0.2 is recorded in [Appendix B](#appendix-b--changelog).

### 4.11 The `ceremony` member

Some outcomes take more than one *Trust Task*. A governance decision may need several endorsements; an onboarding may span a witness and a registry. The framework's model for these is settled in [§2](#2-terminology) — they are multiple bilateral *Trust Tasks* — but the collection itself has, until this version, had no name, no identifier, and no way to be evidenced.

A *Trust Ceremony* is such a collection: a flow of *Trust Tasks*, optionally described by a published *ceremony definition* ([§6.7](#67-ceremony-namespace)), of which one run is an *enactment*. A *Trust Task document* **MAY** carry a `ceremony` member recording that it is one *step* of an enactment.

The member's value is an object with the following members. Its full schema is published with the framework envelope schema for this version.

| Member | Required | Type | Description |
|---|---|---|---|
| `enactment` | **MUST** | string | Identifies one run of a ceremony. Globally unique and never reused, on the same terms as `id` ([§4.3](#43-the-id-member)). |
| `step` | **MUST** | string | Names this step within the ceremony. |
| `definition` | **MAY** | string (URI) | The *ceremony definition* this step is enacted under, rooted at [§6.7](#67-ceremony-namespace). |
| `definitionDigest` | **MUST** where `definition` is present | string | A multibase-encoded multihash over the [[RFC8785]] canonicalization of that definition. |
| `parentEnactment` | **MAY** | string | The enactment containing this one, where a ceremony is conducted as a step of another. |
| `round` | **MAY** | integer | Distinguishes repetitions of the same step by the same party. Absent means `1`. |
| `terminal` | **MAY** | boolean | Marks a step that ends the enactment. |
| `prev` | **MAY** | array | The steps this one follows, each an object of `id` and `digestMultibase`. |

#### 4.11.1 Optionality

The `ceremony` member is optional in every sense that matters, and this is a normative property rather than a convenience:

1. A *Trust Task specification* **MUST NOT** declare anything about ceremonies, and needs no awareness of them. The member is carried on the document, not in `payload`, so any existing specification may be used as a ceremony step with no change to its schema and no new version.
2. A *Trust Task document* without the member is fully conforming.
3. A *consumer* that does not implement ceremonies **MUST** process such a document exactly as it processes any other, under the unrecognized-member rule of [§7.2](#72-consumer-requirements).

#### 4.11.2 The identifiers are orthogonal

`enactment` does not replace `threadId` and is not a form of `parentThreadId`. Within a ceremony, `threadId` scopes one step's request/response exchange exactly as it does elsewhere, and `enactment` scopes the flow across all of its steps; a *producer* sets both. The steps of an enactment are typically *siblings* — several top-level exchanges, none conducted inside another — which is containment's opposite and not what `parentThreadId` records.

The distinction that matters for evidence is that `enactment` **MUST** be globally unique and non-reusable where `threadId` need not be ([§4.9](#49-the-threadid-member)). A reference naming a flow as evidence therefore names the `enactment`, under the rule of [§4.9.1](#491-naming-an-exchange-from-outside-the-framework).

#### 4.11.3 Integrity

Where a `proof` is present it covers the `ceremony` member as it covers any other ([§4.7](#47-proof)). This is the member's placement rationale, not an incidental consequence: a signed `enactment` cannot be lifted into a different flow, and a signed `definitionDigest` cannot be reinterpreted under a definition that gives the step's name another meaning. Carried as transport metadata or as an unsigned sidecar, the member would provide neither guarantee.

A *producer* **MUST NOT** set `parentEnactment` equal to the document's own `enactment`; an enactment cannot contain itself. A *producer* **SHOULD** carry the same `ceremony.enactment` onto every document of the step it names, including any *error response*.

#### 4.11.4 Membership is a claim, not a permission

A `ceremony` member is an assertion by the document's *issuer* that this document belongs to the named enactment. A *consumer* can check what it holds — that a step matches the definition, that a `prev` digest resolves — but cannot verify from one document that the enactment exists as described.

Accordingly:

> A *consumer* **MUST NOT** grant any authority on the basis of ceremony membership alone.

Every authorization decision continues to be reached under [§7.2](#72-consumer-requirements) item 10, exactly as for a document carrying no `ceremony` member — and note that verifying `issuer` and `proof` is not by itself such a decision. Without this rule the member would be a confused-deputy vector: "you are in the onboarding ceremony, so perform this step" is an unauthenticated assertion by whoever composed the document. The rule is also what makes [§4.11.1](#4111-optionality) item 3 safe — because membership authorizes nothing, a *consumer* that ignores the member entirely omits nothing it was entitled to do.

`ceremony` otherwise carries no normative validation semantics: a *consumer* **MUST NOT** reject a document on the basis of the member alone, and **MAY** use it for routing, correlation, aggregation, or audit.

## 5. Versioning

### 5.1 Scheme

Every *Trust Task specification* **MUST** carry a version of the form `MAJOR.MINOR`, where `MAJOR` and `MINOR` are non-negative decimal integers without leading zeros (except for the value `0` itself). Patch-level versions are not used. The grammar, in [[ABNF]], is:

```abnf
version    = major "." minor
major      = "0" / nonzero *DIGIT
minor      = "0" / nonzero *DIGIT
nonzero    = %x31-39                 ; "1".."9"
```

### 5.2 Compatibility rules

A change to a *Trust Task specification* **MUST** be classified as either backwards-compatible or breaking:

* A **backwards-compatible** change — for example, adding an optional member, relaxing a constraint, adding a permitted enumeration value to a non-discriminating field, or clarifying prose — **MUST** result in a `MINOR` increment.
* A **breaking** change — for example, adding or removing a required member, removing a permitted enumeration value, narrowing a constraint, or changing the semantics of an existing member — **MUST** result in a `MAJOR` increment, with `MINOR` reset to `0`.

Implementations of a given *Trust Task specification* at version `M.N` **MUST** accept documents conforming to any version `M.K` where `K ≤ N`.

Forward minor-version compatibility is also intended: because a `MINOR` increment is by definition backwards-compatible, an implementation at `M.N` **SHOULD** accept a document at `M.K` where `K > N`, provided it can ignore any payload members it does not recognize and the document otherwise validates against the framework schema and the `M.N` payload schema known to the implementation. A *consumer* that elects not to support forward minor-version processing **MUST** reject such documents with an `unsupportedVersion` *error response* (see [§8.3](#83-standard-error-codes)).

A `MAJOR` mismatch is never forward-compatible: a *consumer* at `M.N` **MUST** reject any document whose *Type URI* carries a `MAJOR` segment it does not implement, returning `unsupportedVersion` where the transport permits a response.

*This paragraph is non-normative.* Consumers that implement forward-minor compatibility typically route documents by matching the *Type URI*'s slug and `MAJOR` segment and selecting the highest `MINOR` they implement. A consumer that routes by exact-URI equality (slug + `MAJOR.MINOR`) is conformant — strict matching is permitted by [§5.2](#52-compatibility-rules) — but precludes the forward-minor SHOULD; downstream implementations choosing strict matching SHOULD document the trade-off.

While a *Trust Task specification* — or a *shared schema component* ([§6.6](#66-shared-schema-components)) — is at `draft` status ([§5.3](#53-maturity-levels)), its schema and prose **MAY** change without notice. Accordingly, a breaking change to a `draft` artifact **MAY** be released as a `MINOR` increment rather than a `MAJOR` one. Once an artifact reaches `candidate`, `standard`, or `retired`, the classification above applies strictly: every breaking change **MUST** increment `MAJOR`.

A narrower rule applies within `draft`: an **editorial or normalization change** to a `draft` artifact — re-casing an enumerated value or member name into conformance with [§4.10](#410-naming-conventions), re-pinning a `$ref` to a newer framework or *shared schema component* version where the re-pin does not change the payload's effective wire shape, or rewording descriptions and other prose — **MUST** be made in place, errata-style, within the existing version, and **MUST NOT** mint a new version. Such a change carries no semantic difference on the wire; publishing it as a new version inflates the registry, grows the generated libraries, and forces implementations to straddle wire-identical versions for no behavioural gain. At `draft` status this in-place rule takes precedence over the version-coupling rule of [§6.6](#66-shared-schema-components); from `candidate` onward the classification above applies strictly — a re-cased value, in particular, is a breaking change ([§4.10](#410-naming-conventions)). A version that was nonetheless minted for a purely editorial change **MAY** declare the optional `wireCompatibleWith` front-matter field, naming the wire-identical predecessor version, so that *consumers* can dual-accept documents of the predecessor by mechanical normalization rather than a hand-written adapter.

### 5.3 Maturity levels

A *Trust Task specification* progresses through a defined lifecycle, captured by its `status` value. The lifecycle is normative: implementations and the registry use `status` to decide whether a specification can change underfoot, whether new documents **SHOULD** be issued against it, and how the bare-URL redirect in [§6.1](#61-type-uri) resolves. The maturity level is independent of the `MAJOR.MINOR` version number.

| Status | Meaning | Schema stability (§6.4) |
|---|---|---|
| `draft` | Working draft. The schema and prose **MAY** change without notice. | Not stable. |
| `candidate` | Schema is frozen except for editorial clarifications. The specification **MUST** demonstrate two independent, interoperable implementations to enter this status. | Stable. |
| `standard` | Stable in the long term. A `candidate` specification **MUST** complete a continuous 90-day stability window with no breaking changes before promotion to `standard`. | Stable. |
| `retired` | Specification is no longer recommended for new use; preserved for historical reference and to keep already-issued documents verifiable. The schema and prose are frozen at the moment of retirement. | Stable. |

#### Permitted transitions

A `status` value **MUST** change only along one of the transitions below:

1. `draft` → `candidate` — once the entry criteria for `candidate` are met.
2. `candidate` → `standard` — once the 90-day stability window has elapsed without breaking changes.
3. `draft` → `retired` — abandoning a working draft.
4. `candidate` → `retired` — deprecating a candidate before standardization.
5. `standard` → `retired` — sunsetting a standard after a successor has been published.

`retired` is **terminal**: a retired specification **MUST NOT** transition back to any earlier status. To revive functionality, the editor publishes a new `MAJOR.MINOR` of the slug starting at `draft` (see [§5.1](#51-scheme)).

#### Behavior at each status

* **Producers MAY** emit documents whose `type` resolves to a `draft`, `candidate`, or `standard` specification. Producers **SHOULD NOT** emit documents against a `retired` specification.
* **Consumers MUST** treat `draft`, `candidate`, `standard`, and `retired` specifications identically for the purpose of schema validation (the framework rules in [§7.2](#72-consumer-requirements) apply uniformly). Consumers **SHOULD** surface a deprecation signal — in logs, audit records, or downstream interfaces — when a received document's `type` resolves to a `retired` specification, so operators can plan migration.
* A `retired` specification **SHOULD** declare its successor via the optional `supersededBy` front-matter field (see [§7.3](#73-specification-requirements)).

A specification's current status is recorded in its front matter and reflected in the registry at <https://trusttasks.org/>. The same lifecycle applies to this framework specification itself.

The process by which a slug is assigned, by which a specification enters the registry, and by which its status is updated is governed by the registry policy maintained alongside the registry at <https://trusttasks.org/>. That policy is out of scope for this framework specification.

### 5.4 Migrating between versions

*This section is non-normative.*

When a *Trust Task specification* makes a breaking change — including adopting a new version of a *shared schema component* ([§6.6](#66-shared-schema-components)) — implementers are encouraged to migrate using an expand-then-contract sequence that keeps both versions interoperable throughout, so that no single deployment step requires producers and consumers to change in lockstep:

1. **Author the new version.** Publish the new specification version — `M.(N+1)` for a backwards-compatible change, or `(M+1).0` for a breaking change at non-`draft` status (a breaking change at `draft` MAY use a `MINOR` increment per [§5.2](#52-compatibility-rules)). If the change is driven by a shared schema, publish the new shared schema component version first and re-pin the specification's `$ref` to it (see the coupling rule below). The previous specification version remains published and unchanged.

2. **Update receivers first.** Deploy *consumer* support for the new version alongside the old, so a *consumer* accepts documents of both the old and the new version. Because no *producer* is emitting the new version yet, this step is safe to roll out on its own. For a `MINOR` increment, a forward-minor-compatible consumer ([§5.2](#52-compatibility-rules)) may already accept the new version with no code change; for a `MAJOR` increment the consumer **must** add explicit support before any producer emits it.

3. **Update senders.** Once receivers across the deployment accept the new version, deploy *producer* support so producers begin emitting the new version. Traffic shifts to the new version while consumers continue to accept any stragglers still on the old one.

4. **Retire the old version.** After traffic has fully migrated and any applicable stability window has elapsed, transition the old version to `retired` ([§5.3](#53-maturity-levels)) and declare `supersededBy` pointing at the new version. Producers stop emitting the old version; consumers may drop support for it on their own schedule.

**Coupling of schema and specification versions.** A *Trust Task specification* and its payload JSON Schema are a single versioned artifact — the schema's `$id` is the specification's *Type URI* ([§6.3](#63-schema-scope)) — so any change to the payload schema is, by definition, a new specification version. A *shared schema component* ([§6.6](#66-shared-schema-components)) versions independently, but a specification **cannot adopt a new shared schema component version without issuing a new version of itself**: re-pinning a `$ref` changes the specification's effective wire contract. A specification **MAY** instead remain pinned to the older component version and not bump.

## 6. Namespace

The framework defines a single resolvable namespace per versioned *Trust Task specification*. One canonical URL serves human-readable prose, machine-readable schemas, and (where defined) JSON-LD contexts, differentiated by HTTP content negotiation.

### 6.1 Type URI

Every versioned *Trust Task specification* **MUST** be addressable by a *Type URI* — a URI in the sense of [[RFC3986]] — of the form:

```
https://trusttasks.org/spec/<slug>/<MAJOR.MINOR>
```

The form above is the canonical, public-registry form. *Trust Task specifications* intended only for private or internal use — and not published through the public registry — **MAY** use a different authority under the same URI shape; the requirements that apply to those are given in [§6.5](#65-private-and-unpublished-trust-task-specifications).

For both forms, the path components below carry identical meaning:

* The URI scheme **MUST** be `https`. Other schemes (including `http`) are non-conformant: every representation served at a *Type URI* depends on transport-layer authentication and integrity, and permitting `http` would normalize a transport-downgrade path for any *consumer* that dereferences the URI.
* `<slug>` is a lowercase, hyphen-separated short name assigned to the specification, optionally organized into one or more path segments (e.g. `kyc-handoff`, or `acl/grant`). The slug **MUST** match the regular expression `^[a-z][a-z0-9]*(-[a-z0-9]+)*(/[a-z][a-z0-9]*(-[a-z0-9]+)*)*$`. Each `/`-delimited segment **MUST** individually satisfy the single-segment grammar (`^[a-z][a-z0-9]*(-[a-z0-9]+)*$`); consecutive hyphens are not permitted within a segment, and consecutive slashes are not permitted between segments. Segments group related specifications under a shared namespace and are reflected in the *Type URI* path verbatim — `https://trusttasks.org/spec/acl/grant/0.1` is the *Type URI* of a specification whose slug is `acl/grant`.
* `<MAJOR.MINOR>` is the specification version as defined in [§5.1](#51-scheme). When resolving a *Type URI*, a *consumer* identifies the version as the final path segment (which always matches the version grammar) and the slug as the segments between `/spec/` and the version.

A *Type URI* used as the value of a *Trust Task document*'s `type` member **MAY** additionally carry the fragment `#request` or `#response`, with the meanings defined in [§4.4.1](#441-request-and-response-variants). The fragments `#request` and `#response` are **RESERVED**; no other fragment values are defined by this framework, and individual *Trust Task specifications* **MUST NOT** define their own.

The following slugs are **RESERVED** for framework-defined specifications and **MUST NOT** be used by any individual *Trust Task specification*:

* The exact slug `trust-task`, reserved for this framework specification itself.
* Any slug whose first segment is `trust-task` or `trust-ceremony`, or begins with the prefix `trust-task-` or `trust-ceremony-`, reserved for framework-defined specifications. Equivalently, the slug **MUST NOT** match the pattern `^trust-(task|ceremony)($|-|/)`. The `trust-ceremony` half of the reservation is unused at this version and exists so that the ceremony layer of [§4.11](#411-the-ceremony-member) has a namespace to publish into that no other party can claim first. The slugs currently published by the framework under this reservation are:

  | Slug                     | Purpose                                                                 |
  |--------------------------|-------------------------------------------------------------------------|
  | `trust-task-error`       | Error-response payload — see [§8.1](#81-the-trust-task-error-specification). |
  | `trust-task-ok`          | Success-response with metadata — reserved, see [§8.6](#86-reserved-response-type-slugs). |
  | `trust-task-next-step`   | Recipient-suggested continuation — see [§8.6](#86-reserved-response-type-slugs). |
  | `trust-task-discovery`   | Discovery and capability negotiation — see [§11](#11-discovery-and-capability-negotiation). |
  | `trust-ceremony-receipt` | Evidence that one *enactment* of a *Trust Ceremony* completed — see [§4.11](#411-the-ceremony-member). |

The *Type URI* is the single canonical, resolvable reference to a versioned *Trust Task specification*. It serves both humans (rendered prose) and machines (validation schema, optional JSON-LD context) under content negotiation as defined in [§6.2](#62-content-negotiation).

The framework also reserves a parallel `/binding/` subtree under the same authority for *transport binding* identifiers and binding-internal resources (envelope `type` values, binding schema URIs, status mappings). The `/binding/` subtree is **structurally disjoint** from `/spec/`: no URI under `/binding/` is a *Type URI*, and a *Trust Task document* whose `type` is rooted at `/binding/...` is malformed. The grammar and rules for the `/binding/` subtree are defined in [§9.3](#93-binding-namespace).

A third subtree, `/ceremony/`, is reserved for *ceremony definitions* on the same terms. It is likewise **structurally disjoint** from both: no URI under `/ceremony/` is a *Type URI*, and a *Trust Task document* whose `type` is rooted at `/ceremony/...` is malformed. The grammar and rules for the `/ceremony/` subtree are defined in [§6.7](#67-ceremony-namespace).

A *Type URI* with the `<MAJOR.MINOR>` segment omitted (i.e. `https://trusttasks.org/spec/<slug>`) **SHOULD** redirect to the latest `standard` version of the specification, or — if no `standard` version exists — to the latest `candidate`, or — failing that — to the latest `draft`. `retired` versions **MUST NOT** be selected by the bare-URL redirect, since `retired` signals "no longer recommended for new use"; if every version of a slug is `retired`, the bare URL **SHOULD** return `410 Gone` with a body that links to the latest retired version and its declared `supersededBy` successor, if any.

### 6.2 Content negotiation

A server hosting a *Type URI* **MUST** support HTTP content negotiation [[RFC9110]] and **MUST** be capable of returning the representations listed below. The server **MUST** return the representation matching the highest-priority acceptable media type in the request's `Accept` header. If the `Accept` header is absent or names no representation listed below, the server **MUST** return the `text/html` representation.

| Media type | Representation |
|---|---|
| `text/html` | The rendered specification document for human readers. |
| `application/schema+json` | The normative JSON Schema [[JSON-SCHEMA-2020-12]] for this specification (see [§6.3](#63-schema-scope) for what the schema covers). |
| `application/ld+json` | The JSON-LD context for this specification, when one is defined. If no context is published for this *Type URI*, the server **MUST** respond with HTTP `406 Not Acceptable`. |

Every representation returned **MUST** describe the same version of the specification as is encoded in the requested *Type URI*.

### 6.3 Schema scope

The JSON Schema served at the *Type URI* of an individual *Trust Task specification* describes **only** the contents of that specification's `payload` member.

The outer document structure (`id`, `threadId`, `parentThreadId`, `type`, `issuer`, `recipient`, `issuedAt`, `expiresAt`, `payload`, `@context`, `proof`) is described by the JSON Schema served at the framework's own *Type URI* — `https://trusttasks.org/spec/trust-task/<MAJOR.MINOR>` — under content negotiation for `application/schema+json`. A complete document validation therefore composes the framework schema (outer structure) with the task-specific payload schema.

The JSON Schema served at any *Type URI* **MUST** declare:

* `$id` equal to that *Type URI*.
* `$schema` set to `https://json-schema.org/draft/2020-12/schema`.

It **MUST** specify `additionalProperties` either explicitly as `false` or with an accompanying prose statement of how unrecognized members are to be treated.

### 6.4 Stability

For any value of `<slug>` and any value of `<MAJOR.MINOR>`, the representations served at the corresponding *Type URI* **MUST NOT** change in a way that alters their normative content once the specification has reached the `candidate`, `standard`, or `retired` status. Once a specification is `retired`, the schema and prose are frozen at the moment of retirement; the only permitted change is correcting the `status` value itself (which is itself terminal — see [§5.3](#53-maturity-levels)) or adding the `supersededBy` declaration.

This commitment is made by the public registry for *Trust Task specifications* it hosts; private specifications published under their own authority (see [§6.5](#65-private-and-unpublished-trust-task-specifications)) **SHOULD** offer their consumers an equivalent commitment, scoped to their own trust boundary.

### 6.5 Private and unpublished Trust Task specifications

Not every *Trust Task specification* is intended for the public registry. A *producer* and *consumer* operating within a single organization, deployment, or trust boundary **MAY** define their own *Trust Task specifications* solely for internal use — never publishing them under `https://trusttasks.org/` — and still conform fully to this framework.

The following rules apply to *Trust Task specifications* that are not published through the public registry:

1. **Authority.** A private specification's *Type URI* **MUST NOT** be served from, or claim to identify a resource at, the `https://trusttasks.org/` domain. That domain is reserved for *Trust Task specifications* published through the public registry process. A private specification **SHOULD** use an HTTPS authority the publisher controls — typically a project or organization domain — so the URI uniquely identifies the specification within the publisher's trust boundary. Examples:
   ```
   https://example.com/trust-tasks/<slug>/<MAJOR.MINOR>
   https://internal.example/spec/<slug>/<MAJOR.MINOR>
   ```
   The slug grammar, version grammar, fragment conventions, and path-component meanings defined in [§6.1](#61-type-uri) apply unchanged.

2. **Reservation rule.** The slug reservation rule in [§6.1](#61-type-uri) — that the slug **MUST NOT** be `trust-task` or have a first segment matching `^trust-task(-|/)?` — applies regardless of authority. A private specification **MUST NOT** use those reserved slugs even on its own domain, so that documents flowing between trust boundaries cannot be confused with framework-defined response types.

3. **Framework conformance is unchanged.** All other framework requirements — the document structure ([§4](#4-trust-task-documents)), versioning rules ([§5](#5-versioning)), conformance behaviour ([§7](#7-minimum-requirements)), and error response shape ([§8](#8-error-responses)) — apply identically to private *Trust Task specifications*. Implementations consuming both private and registry-published specifications **SHOULD** use the same validation and signing pipeline for both.

4. **Resolvability.** A private *Type URI* **SHOULD** resolve to the specification's representations under content negotiation ([§6.2](#62-content-negotiation)) for parties within the publisher's trust boundary, but **MAY** be unresolvable from the public internet. A *consumer* unable to dereference a private *Type URI* relies on out-of-band distribution of the specification document and schema.

5. **Promotion to the registry (informative).** A private *Trust Task specification* **MAY** later be submitted for inclusion in the public registry. The submission process is governed by the registry policy referenced in [§5.3](#53-maturity-levels); a re-host typically involves a slug check, transfer of the JSON Schema document, and publication under `https://trusttasks.org/spec/<slug>/<MAJOR.MINOR>`. The original private *Type URI* and the new public *Type URI* identify distinct specifications unless and until the registry policy explicitly aliases them.

Private *Trust Task specifications* are full *Trust Task specifications* for the purposes of conformance: a producer or consumer that satisfies §7 against a private spec is a *conforming producer* or *conforming consumer* of that spec, exactly as it would be for a registry-published one.

### 6.6 Shared schema components

A *Trust Task specification*'s payload JSON Schema **MAY** reference reusable schema fragments — *shared schema components* — that several specifications have in common (for example, an access-control entry, a device binding, a sealed-envelope shape, or a synchronization event). Shared schema components are an authoring convenience and a consistency mechanism. They are **not** independently published *Type URIs*: a *Trust Task document*'s `type` **MUST NOT** resolve to a shared schema component, and a shared schema component is never the unit a document conforms to — only a specification is.

A shared schema component is nonetheless a *versioned artifact* in its own right, governed by the following rules.

1. **Versioning.** A shared schema component carries a `MAJOR.MINOR` version and follows the same compatibility rules as a specification ([§5.2](#52-compatibility-rules)), including the `draft` caveat. A breaking change to a shared schema component — re-casing an enumerated value, removing or renaming a `$def`, narrowing a constraint — **MUST** be published as a new version of that component. The prior version remains in place for the specifications still pinned to it.

2. **Pinning.** A consuming specification **MUST** reference a shared schema component at a specific version. Resolving a reference to "the latest" version of a component is non-conformant, because a later change to the component would otherwise alter an already-published specification's wire contract silently.

3. **Coupling to specification versions.** Because a consuming specification pins a specific component version, adopting a new component version requires the consuming specification to publish a new version of itself (see [§5.4](#54-migrating-between-versions)). A component version bump therefore never changes an already-published specification underfoot; the new component values become observable only through a new specification version that re-pins to them.

4. **Lifecycle and discovery.** A shared schema component **SHOULD** declare its own `status` ([§5.3](#53-maturity-levels)) and **MAY** declare `supersededBy` when retired. The registry **SHOULD** surface shared schema components and their versions alongside specifications, so that implementers can see which specification versions depend on which component versions.

### 6.7 Ceremony namespace

A *ceremony definition* — the published description of a flow composed of several *Trust Tasks*, referenced by the `ceremony.definition` member of [§4.11](#411-the-ceremony-member) — is identified by a URI in the `/ceremony/` subtree of the framework's authority:

```
https://trusttasks.org/ceremony/<slug>/<MAJOR.MINOR>
```

`<slug>` follows the same lowercase, hyphenated grammar as a Trust Task slug ([§6.1](#61-type-uri)) and is subject to the same `^trust-(task|ceremony)($|-|/)` reservation; `<MAJOR.MINOR>` follows the version grammar of [§5.1](#51-scheme).

The `/ceremony/` subtree is **structurally disjoint** from `/spec/` and `/binding/`. A *ceremony definition* is not a *Trust Task specification*: no document's `type` resolves to one, and a *consumer* that receives a *Trust Task document* whose `type` is rooted at `/ceremony/...` **MUST** reject it with `malformedRequest` ([§8.3](#83-standard-error-codes)). The Type URI grammar of [§6.1](#61-type-uri) already excludes the path; the rule is stated explicitly so the namespace boundary is visible at a glance and so such documents have a defined disposition rather than relying on grammar mismatch.

A *ceremony definition* is referenced by content as well as by name: a step carrying `ceremony.definition` **MUST** also carry `ceremony.definitionDigest` ([§4.11](#411-the-ceremony-member)). A URI alone would leave the flow's rules mutable by whoever controls the URI, retroactively and for every enactment already performed.

This version of the framework defines the namespace, the reservation, and the reference mechanism. The **content** of a ceremony definition — its role, step, ordering and completion vocabulary — is out of scope for this revision and is expected to be specified in a future one. A *consumer* encountering a `ceremony.definition` it cannot resolve or does not understand **MAY** process the document as though the member were absent; by [§4.11.4](#4114-membership-is-a-claim-not-a-permission) it forgoes no authority in doing so.

The reservation rule of [§6.5](#65-private-and-unpublished-trust-task-specifications) applies equivalently: a private ceremony definition **MUST** use an authority its publisher controls and **MUST NOT** claim to identify a resource at `https://trusttasks.org/ceremony/...`.

## 7. Minimum requirements

### 7.1 Producer requirements

A *conforming producer* **MUST**:

1. Emit a *Trust Task document* whose top-level structure satisfies [§4.2](#42-top-level-members).
2. Set the `type` member to the *Type URI* of the *Trust Task specification* being implemented, including its `<MAJOR.MINOR>` segment.
3. Place all task-specific data in `payload`, and emit a `payload` value that validates against the JSON Schema obtained by content-negotiating the *Type URI* for `application/schema+json` (see [§6.2](#62-content-negotiation)).
4. Populate `id` with a value satisfying [§4.3](#43-the-id-member).

A *conforming producer* **SHOULD** populate `issuedAt` to support freshness checks downstream, **SHOULD** populate `issuer` and `recipient` when the transport in use does not provide authenticated party identity end-to-end between *producer* and *consumer*, **SHOULD** set `threadId` when emitting a *Trust Task document* in response to another (see [§4.9](#49-the-threadid-member)), **SHOULD** set `parentThreadId` when the exchange is conducted inside another and carry it onto every document of the inner exchange (see [§4.9.2](#492-the-parentthreadid-member)), **SHOULD** set `ceremony` when the document is a step of a *Trust Ceremony* and carry the same `enactment` onto every document of that step including any *error response* (see [§4.11](#411-the-ceremony-member)), and **SHOULD** preserve any unrecognized members received from upstream parties when forwarding a *Trust Task document*.

A *conforming producer* that emits an `ext` member (see [§4.5.1](#451-the-ext-extension-member)) **MUST** namespace every immediate child key of `ext` under a reverse-DNS prefix the producer controls; bare or un-namespaced child keys are non-conforming.

### 7.2 Consumer requirements

A *conforming consumer* **MUST**:

1. Validate the outer document structure against the framework JSON Schema. The applicable framework version is the *target framework version* declared by the *Trust Task specification* identified by the document's `type` member (see [§7.3](#73-specification-requirements)). The framework schema for that version is obtained by content-negotiating `https://trusttasks.org/spec/trust-task/<MAJOR.MINOR>` for `application/schema+json`, where `<MAJOR.MINOR>` is the declared target framework version — **not** the `<MAJOR.MINOR>` of the document's `type` member, which identifies the task specification version and may differ.
2. Validate the document's `payload` member against the JSON Schema obtained by content-negotiating the document's `type` member for `application/schema+json`.
3. Reject any document whose `type` it does not recognize, unless the consumer's policy explicitly permits forward-compatible processing under [§5.2](#52-compatibility-rules).
4. Honor the document's expiry where present: if `expiresAt` is set and `now ≥ expiresAt` relative to the *consumer*'s clock (with the optional skew tolerance permitted in [§4.2](#42-top-level-members)), treat the document as expired and not act upon it. This is an **acceptance** bound: it governs whether the *consumer* may begin, and does not by itself require it to abandon execution already under way (see item 12).
5. Reject any document whose `recipient` member is set and does not identify the *consumer*'s own party. Where the *Trust Task specification* declares `recipient` as **REQUIRED** (see [§7.3](#73-specification-requirements) item 5), reject any document lacking an in-band `recipient` with `malformedRequest`.
6. Reject any document for which an in-band `issuer` or `recipient` member is inconsistent with an authenticated identity derived from the transport for the same party.
7. If the document carries a `proof` member, verify it per [§4.7](#47-proof) against the in-band `issuer` and reject the document with `proofInvalid` on verification failure. Independently, if the *Trust Task specification* identified by `type` declares `proof` as **REQUIRED** (see [§7.3](#73-specification-requirements) item 8) and no `proof` is present, reject the document with `proofRequired`.
8. If the document carries a `proof` member and no in-band `recipient`, and the *Trust Task specification* identified by `type` is **not** a *bearer specification* ([§4.8.3](#483-bearer-specifications)), reject the document with `malformedRequest`. This enforces the audience-binding rule of [§4.8.2](#482-audience-binding).
9. Not grant any authority on the basis of a `ceremony` member. Membership of an *enactment* is an assertion by the document's *issuer*, not a verified fact, and every authorization decision **MUST** be reached under item 10 below exactly as for a document carrying no such member. See [§4.11.4](#4114-membership-is-a-claim-not-a-permission). A *consumer* that does not implement ceremonies applies the unrecognized-member rule below and forgoes nothing by doing so.
10. Not treat identity or document-proof validation as authorization. Successful validation of a *VID*, `issuer`, `recipient`, transport-derived identity, or `proof` establishes **who** made the assertion and that the document reached the *consumer* unaltered. It **MUST NOT**, by itself, be treated as establishing that the *producer* is authorized to request the outcome the *Trust Task* describes, or that the *consumer* is authorized to perform it. Before executing a *Trust Task*, a *consumer* **MUST** evaluate whatever authorization requirements apply under the *Trust Task specification* identified by the document's `type`, the *consumer*'s own policy, and the trust or governance framework it operates under.
11. Not execute a *consequential Trust Task* ([§2](#2-terminology)) twice on account of the same *Trust Task document*. Once a *consumer* has accepted a document with a given `id` for execution, receipt of that same document again **MUST NOT** cause the consequential effect to occur a second time, unless the *Trust Task specification* identified by the document's `type` explicitly declares repeated execution safe and intended. A *consumer* receiving a document whose `id` matches one it has already accepted but whose content differs **MUST** reject the later document with `idConflict` ([§8.3](#83-standard-error-codes)) and **MUST NOT** treat it as a retry of the original. Transport request identifiers, transport message identifiers, and execution handles **MUST NOT** substitute for the *Trust Task document*'s `id` as the key for this rule.
12. Re-evaluate, immediately before each irreversible or externally visible effect of a *consequential Trust Task* ([§2](#2-terminology)), every condition that the *Trust Task specification* and the *consumer*'s own policy require for that effect. Successful validation establishes that a document was eligible for processing **when it was validated**; it does not establish that the work remains executable indefinitely, and for execution that is delayed, long-running, or resumed the two instants can be far apart. Where a required condition — an authorization, delegation, mandate, capability, membership, standing, credential or key status, subject relationship, or a deadline the *Trust Task specification* defines for itself — is no longer satisfied at that point, the *consumer* **MUST NOT** perform the subsequent effect.

For each of the rules in this section that references the `issuer` or `recipient` party, the in-band member value is authoritative when present and the transport-derived identity is a cross-check; when the in-band member is absent the *consumer* **MAY** derive the value from the transport. This precedence is defined normatively in [§4.8.1](#481-precedence-of-in-band-over-transport-derived-identity).

The evaluation required by item 10 **MAY** consider delegation, mandate, capability, membership, standing, credential status, subject relationship, purpose limitation, or any other evidence the *consumer* requires; this framework does not prescribe an authorization model and does not constrain which of these a *consumer* consults. A verified assertion **MAY** itself constitute authorization evidence, but only where the *Trust Task specification* explicitly defines that role for it ([§7.3](#73-specification-requirements) item 15) **and** the *consumer*'s policy accepts it for that purpose — a signed decision whose proof *is* the authorization is a design a specification may adopt deliberately, not a default a *consumer* may infer. Where authorization fails after the document has otherwise validated, the *consumer* **SHOULD** return an *error response* of `permissionDenied` ([§8.3](#83-standard-error-codes)) where one can be returned safely, subject to the message-content rule of [§10.4](#104-error-response-identity-leakage).

**Keying and comparison for item 11.** The duplicate-execution key is the *Trust Task document*'s `id` alone, which *producers* are required to mint globally unique and never to reuse ([§4.3](#43-the-id-member)). Two documents bearing the same `id` are *the same document* for the purposes of item 11 when their serializations are identical under [[RFC8785]] canonicalization — the same identity [§8.4](#84-retry-semantics) defines for a retry. Any other difference, including a changed `payload`, a changed `recipient`, or a re-signed `proof` over identical content, makes them different documents sharing an `id`, which is the `idConflict` case. A *consumer* implementing item 11 therefore retains a digest of what it accepted, not merely the `id`: an `id` alone cannot distinguish the retry it must absorb from the conflict it must reject.

**Bounding the record.** A *consumer* cannot apply item 11 to a document it can no longer recognize, so it **MUST** retain the record for at least as long as it remains willing to execute that document. The two bounds are the same bound. Where `expiresAt` is present it fixes both: after it, the document is refused under item 4 and the record may be dropped. Where `expiresAt` is absent, the *consumer*'s own acceptance window — typically a freshness bound over `issuedAt` — fixes both, and a *consumer* **MUST NOT** accept for execution a document older than the window over which it retains records. Retention beyond that point is not required: a document the *consumer* would now refuse as stale cannot be replayed into a second execution. A *consumer* that can establish neither an `expiresAt` nor an age for a document has no window in which to place it, and **MUST NOT** execute a *consequential Trust Task* on it.

**What item 12 does and does not re-check.** The rule is about **authority**, not about the clock. `expiresAt` is deliberately absent from its list: that member bounds acceptance ([§4.2](#42-top-level-members)), and re-checking it mid-execution would convert a statement about a request's staleness into an execution timeout the *producer* never set and could not have calculated — it does not know how long the *consumer*'s work takes. A *consumer* **MUST NOT** abandon execution solely because `expiresAt` has passed since it began.

A *Trust Task* that genuinely has a completion deadline — an offer that lapses, a quote that cannot be honored after a stated instant — expresses it in its own `payload`, where the *Trust Task specification* can define what lapsing *means* for that task. Such a deadline is then one of the conditions item 12 re-evaluates, on the same footing as a revoked delegation, with no framework member required. `task-consent/request/0.1` is the worked example: its `payload.expiresAt` states that the pending request lapses and no decision is accepted for it — a meaning the envelope member could not carry, because the envelope does not know what a decision is.

**Stopping is not always safer than finishing.** Item 12 is placed *before* each irreversible effect for a reason: once such an effect has occurred it cannot be undone by declining the next one, and for many *consequential Trust Tasks* abandoning a partially applied change leaves the *recipient party* in a state neither party asked for. A *consumer* **MUST NOT** treat abandonment as inherently the safe option; where a *Trust Task specification* defines a sequence whose partial application is unsafe, the specification says so and the *consumer* follows it.

Where execution has already produced partial or irreversible effects and the *consumer* stops, it **SHOULD** return a response or status that distinguishes **partial execution** from a task that was never begun. The two are different facts about the world, and a *producer* that cannot tell them apart cannot decide whether to reissue. Where the *Trust Task specification* defines no success-response document, this disposition is reported as an *error response* ([§8](#8-error-responses)) rather than left silent — the silence of item 11 signifies a duplicate absorbed, and must not be reused to signify work half-done.

**Disposition of a duplicate.** Where the original execution is still in progress, the *consumer* **SHOULD** return or expose the existing execution state rather than begin another. Where execution has completed and the *Trust Task specification* defines a success-response document ([§7.3](#73-specification-requirements) item 7.6), the *consumer* **SHOULD** return the previously determined result, or an equivalent receipt where that specification permits one. Where the specification defines **no** success-response document — the fire-and-forget case of that same item — there is nothing to return: the *consumer* declines to execute again and that silence is the correct disposition, not an error. In no case is a duplicate reported as `taskFailed`; the task did not fail, it already happened.

**Relationship to idempotency.** Idempotency as a property of the underlying operation remains task-specific and outside this framework. Item 11 requires only that transport retry or replay not invoke a consequential operation a second time. A *Trust Task specification* whose operation is naturally idempotent — where executing twice is indistinguishable from executing once, in every effect the *recipient party* exposes — **MAY** declare repeated execution safe and intended, which disapplies item 11 for that specification. Such a declaration is about the operation, not about the *consumer*'s convenience, and a specification **MUST NOT** make it merely to avoid implementing the rule.

*This paragraph is non-normative.* Item 10 generalizes a principle the framework already applies in two narrower places: [§4.11.4](#4114-membership-is-a-claim-not-a-permission) and item 9, where ceremony membership authorizes nothing, and [§7.3](#73-specification-requirements) items 13 and 14, where the side-effect and exposure classes describe a task without authorizing it. Those are instances of the general rule rather than exceptions to it. The inference the rule forecloses — *valid `proof` + recognized `issuer` + correct `recipient` = authorized instruction* — is the confused-deputy vector of [§4.11.4](#4114-membership-is-a-claim-not-a-permission) reached by a different route, and it is most dangerous where the *producer* is an autonomous agent: such a *producer* can typically prove its own identity perfectly while holding no authority to act for a subject, exercise a delegated capability, disclose information, or cause a *consequential* effect ([§2](#2-terminology)).

A *conforming consumer* **SHOULD** preserve, but **MUST NOT** act upon, members it does not recognize. A *consumer* that does not implement JSON-LD processing **MUST** ignore the `@context` member.

For documents that carry an `ext` member (see [§4.5.1](#451-the-ext-extension-member)), a *conforming consumer* **MUST** ignore every `ext` immediate-key namespace it does not recognize — the unrecognized-namespace rule is the same "preserve but MUST NOT act upon" rule as for unrecognized top-level members, applied at the `ext` level. A *consumer* **MAY** require one or more specific namespaces under `ext` as a matter of local policy and **MUST** reject a document missing a required namespace with `malformedRequest`; *consumers* applying such a policy **SHOULD** publish the requirement via discovery ([§11](#11-discovery-and-capability-negotiation)) so *producers* can satisfy it before the wire trip.

When a *consumer* rejects a *Trust Task document* under any rule in this section, and the transport in use supports a response from *consumer* to *producer*, the *consumer* **SHOULD** return an *error response* conforming to [§8](#8-error-responses).

### 7.3 Specification requirements

A *conforming Trust Task specification* **MUST** declare each of the following. Together these declarations make the specification self-describing to both human readers and machine consumers, and constitute the minimum needed to author or interpret a *Trust Task document*.

1. **Slug** — the lowercase slug used in the specification's *Type URI*, satisfying the grammar and reservation rules of [§6.1](#61-type-uri).
2. **Version** — the `MAJOR.MINOR` version of this specification, satisfying [§5.1](#51-scheme).
3. **Target framework version** — the `MAJOR.MINOR` version of this framework specification that the *Trust Task specification* targets. A *consumer* uses this declared value to select the framework schema under which the outer document is validated (see [§7.2](#72-consumer-requirements), item 1).
4. **Maturity level** — one of `draft`, `candidate`, `standard`, or `retired`, satisfying [§5.3](#53-maturity-levels). A specification whose status is `retired` **SHOULD** also declare a `supersededBy` value (item 11) pointing at the successor.
5. **Parties** — the role of each *party* expected in a document conforming to this specification, the *VID* schemes accepted for each, and whether each of the `issuer` and `recipient` members is **REQUIRED**, **RECOMMENDED**, or **OPTIONAL** in a document. The defaults from [§4.8](#48-the-issuer-and-recipient-members) apply if the specification is silent, but explicit declaration is **RECOMMENDED**. A **REQUIRED** declaration is enforceable: a *consumer* **MUST** reject documents lacking an in-band member declared **REQUIRED** with `malformedRequest` (see [§7.2](#72-consumer-requirements) item 5). **RECOMMENDED** and **OPTIONAL** declarations are advisory and impose no rejection obligation. A specification identifies which party fills each framework member by tagging that party `issuer` or `recipient`; a party named only in the *payload* — neither the document's `issuer` nor its `recipient` — carries no such tag. The requirement of the party tagged `recipient` governs the `recipient` member of a request document; because a response document swaps the parties ([§4.4.1](#441-request-and-response-variants)), the requirement of the party tagged `issuer` governs the `recipient` member of a response.
6. **Outcome** — a non-normative prose statement of what successful execution of the task achieves between the parties. This is the human-readable counterpart to the payload schema.
7. **Payload JSON Schema** — a normative JSON Schema for the `payload` member that:
   1. Is a valid JSON Schema document under [[JSON-SCHEMA-2020-12]].
   2. Sets `$id` to the specification's *Type URI* (without fragment).
   3. Sets `$schema` to `https://json-schema.org/draft/2020-12/schema`.
   4. States how unrecognized payload members are treated — by specifying `additionalProperties` explicitly as `false`, by specifying `unevaluatedProperties` as `false`, or with an accompanying prose statement. A schema assembled by `allOf` over a *shared schema component* ([§6.6](#66-shared-schema-components)) **MUST** use `unevaluatedProperties`: `additionalProperties` is evaluated by each subschema against the whole instance and cannot see members a sibling subschema matched, so it rejects the composing schema's own members. For the same reason a shared component intended for composition **SHOULD** leave itself open and let the consuming schema close the result.
   5. Is served at its *Type URI* under content negotiation for `application/schema+json`.
   6. Where the specification defines a success-response document (per [§4.4.1](#441-request-and-response-variants)), the schema **MUST** contain a sub-schema reachable via `$anchor: "response"` describing the response document's `payload`; the top-level schema (or the sub-schema reachable via `$anchor: "request"`) describes the request document's `payload`. A *consumer* receiving a document whose `type` carries `#response` resolves the response sub-schema by dereferencing the bare *Type URI* and following the `response` anchor. Where the specification defines no success-response document, the schema **MUST NOT** declare a `response` anchor; such tasks are fire-and-forget at the application layer (failures are still reported via `trust-task-error` per [§8](#8-error-responses)).
8. **Proof requirement** — an explicit statement of whether the `proof` member is **OPTIONAL**, **RECOMMENDED**, or **REQUIRED**, together with a brief rationale referencing the threat model addressed (for example, tampering by intermediaries, replay, repudiation by the *producer*, or reliance by third parties beyond the original *consumer*). The declared requirement **MUST NOT** be weaker than the default applicable under [§4.7.1](#471-when-to-include-a-proof).

    The statement takes one of two forms. A specification **MAY** declare a **single** requirement applying to every document variant, or it **MAY** declare **per-variant** requirements for the *request* and the *response* separately. The per-variant form exists because the two are relied upon differently: a response retained as evidence by a party outside the original exchange can require a proof where the request that triggered it does not, and the reverse is equally common — a request that destroys state needs to be attributable while the acknowledgement it returns protects nothing. A single value forces the stricter of the two onto both, overstating the requirement on whichever variant needs it less. Where a specification declares no requirement for the *response*, the *request*'s applies to it, so an omission can never weaken a variant.

    A *consumer* applies the requirement declared for the variant it is processing, identified by the document's `type` fragment ([§4.4.1](#441-request-and-response-variants)); the rejection rule is unchanged ([§7.2](#72-consumer-requirements) item 7).

    The **error** variant is deliberately **not** declarable here. An *error response*'s `type` resolves to the framework's `trust-task-error` specification ([§8.1](#81-the-trust-task-error-specification)), which is a different *Trust Task specification* from the one being declared, and §7.2 item 7 resolves the proof requirement from the specification the document's `type` names. A declaration made here could not reach it.
9. **Task-specific error codes (where used)** — for each extended `code` defined under [§8.5](#85-extension-by-individual-trust-task-specifications), the code identifier, its meaning, its default `retryable` value, and the JSON Schema fragment describing any `details` object it carries. Where no extensions are defined, the specification **SHOULD** state so explicitly.
10. **JSON-LD context (where used)** — if the specification publishes a canonical JSON-LD context, the context **MUST** be served at the specification's *Type URI* under content negotiation for `application/ld+json` (see [§4.6](#46-json-ld-compatibility) and [§6.2](#62-content-negotiation)). Where no context is published, the specification **SHOULD** state so explicitly.
11. **Successor (`supersededBy`, retired specifications only)** — a `retired` specification **SHOULD** declare its successor as a string of the form `<slug>` or `<slug>/<MAJOR.MINOR>`. The bare-slug form points to "the latest non-retired version of that slug"; the explicit form pins to a specific version. The value is used by the registry's bare-URL redirect (see [§6.1](#61-type-uri)) and by consumer-side deprecation tooling to direct implementers at the recommended replacement. Specifications whose status is not `retired` **MUST NOT** declare `supersededBy`.
12. **Bearer flag (where applicable)** — a *Trust Task specification* that opts out of the audience-binding rule of [§4.8.2](#482-audience-binding) **MUST** declare `bearer: true` in its front matter. The default is non-bearer; specifications omit the field or set `bearer: false` when audience binding applies. A *bearer specification* **MUST** also declare `recipient` as **OPTIONAL** under item 5 and **MUST** include the audience-free rationale required by [§4.8.3](#483-bearer-specifications).
13. **Side-effect class** — an explicit classification of the effect that successful execution has on the *recipient party*, declared in front matter as a `sideEffects` object carrying a `level` — one of `none`, `mutating`, or `destructive` — and a brief `rationale`. `none` denotes a read-only or idempotent task that persists no state change (a query, an enumeration, a discovery probe); `mutating` denotes creation or alteration of recoverable state; `destructive` denotes an irreversible or authority-shifting effect (deactivation, rotation of a sole controlling key, deletion, transfer of ownership). For a `destructive` classification the rationale **MUST** name the irreversible or authority-shifting effect. This classification is the structured, machine-actionable counterpart to the prose Outcome of item 6.

    The classification is **descriptive**: it states what the task *does*, not whether approval is required to do it. A *consumer* that gates execution on human approval — for example an agent executing a task delegated to it by a *producer* — MAY derive its approval policy from this class, but that policy is the *consumer*'s alone. Accordingly: a *Trust Task specification* **MUST NOT** declare, in any form, that a task does or does not require consent, human approval, or an authentication step-up; such policy **MUST NOT** be delegable to a specification or to the registry that serves it. A *consumer* that enforces an approval policy **MUST** determine the authoritative side-effect class from the handler it is about to invoke rather than from the declared value alone, and **MUST** treat an absent, unrecognized, or unresolvable declaration as no weaker than `mutating`. The declared class exists to inform and to render, not to authorize.
14. **Exposure class** — a declaration, orthogonal to the side-effect class of item 13, of what successful execution causes to *leave* the recipient or to be *exercised* on the subject's behalf, independent of any change to recipient state. Declared in front matter as an `exposure` object carrying a `discloses` value — one of `none`, `metadata`, or `secret` — and an `actsAsSubject` boolean. `discloses` states the sensitivity of data the task returns to the caller: `none` (an acknowledgement or a determination only), `metadata` (non-secret descriptive data about a subject or resource, such as an enumeration or a status read), or `secret` (confidential material the caller retains, such as released credential material or a usable session blob). `actsAsSubject` is `true` when execution exercises the subject's own authority to produce an attributable effect in the subject's name — a login performed on their behalf, a signature bearing their identity, a credential issued under their authority — even when no data is disclosed and no recipient state changes. Where `discloses` is not `none` or `actsAsSubject` is `true`, a `rationale` naming the disclosed material or the exercised authority is **REQUIRED**.

    The side-effect class (item 13) and the exposure class are **orthogonal**: the former measures the *integrity* effect on recipient state, the latter the *confidentiality and agency* effect of data egress and delegated action. A read-only task (`sideEffects.level: none`) may still disclose a secret; a signing task may change no recipient state yet act with the subject's full authority. Both are governed by the same discipline as item 13: the exposure class is **descriptive, not prescriptive** — a specification **MUST NOT** derive from it a consent requirement — and a *consumer* that gates on it **MUST** determine the authoritative values from the handler it is about to invoke, and **MUST** treat an absent or unresolvable declaration as no less exposed than `discloses: secret` with `actsAsSubject: true`.

15. **Authorization evidence (consequential tasks)** — a *Trust Task specification* defining a *consequential Trust Task* ([§2](#2-terminology)) **MUST** describe any class of authorization evidence a *consumer* needs in order to interpret the task correctly — for example the delegation, mandate, capability, membership, standing, or subject relationship the task presupposes. Where the task presupposes nothing beyond the *consumer*'s own policy, the specification **SHOULD** state so explicitly.

    This declaration is descriptive on exactly the terms of items 13 and 14. It states what authority the task **assumes**, and **MUST NOT** be read as obliging a *consumer* to authorize execution merely because the described evidence is present; the authorization decision remains the *consumer*'s alone under [§7.2](#72-consumer-requirements) item 10. The bar in item 13 applies here unchanged: a specification **MUST NOT** declare, in any form, that a task does or does not require consent, human approval, or an authentication step-up.

    A specification **MAY** additionally declare that a verified assertion carried by the task *is* authorization evidence for a stated purpose — the design [§7.2](#72-consumer-requirements) item 10 contemplates, in which a proof is relied upon as authorization rather than merely as integrity. Such a declaration **MUST** name the purpose and **MUST NOT** extend beyond it, and remains subject to the *consumer*'s policy accepting the assertion for that purpose.

    Unlike items 5, 8, 12, 13, and 14, this declaration is satisfied in the specification's prose and has no front-matter field; it is not machine-validated.

16. **Execution checkpoints (multi-stage consequential tasks)** — a specification describing multi-stage consequential execution **SHOULD** identify any additional points at which validity or authority is expected to be re-evaluated, beyond the one [§7.2](#72-consumer-requirements) item 12 requires before each irreversible effect. A specification whose stages must not be partially applied **SHOULD** say so explicitly, so that a *consumer* deciding whether to stop knows which of stopping and continuing its author considered the safer failure.

    Where the specification defines its own completion deadline — an instant after which the task's outcome is no longer meaningful — it declares that in its `payload` and states what lapsing means for the task. The framework's `expiresAt` bounds acceptance only ([§4.2](#42-top-level-members)) and **MUST NOT** be relied upon to terminate execution.

A worked example of a *Trust Task specification* satisfying these requirements appears in [Appendix A](#appendix-a--example-trust-task-specification).

## 8. Error responses

A *recipient party* that cannot or will not act upon a received *Trust Task document* **MAY** return an **error response** describing why. Error responses are themselves *Trust Task documents* of a framework-defined type, so that one validation, signing, and transport pipeline serves both successful tasks and their refusals.

The framework distinguishes the two reply forms cleanly:

* A **success response** uses the request's *Type URI* with the fragment `#response` (see [§4.4.1](#441-request-and-response-variants)). Its payload shape is defined by the originating *Trust Task specification*.
* An **error response** uses the framework's distinct `trust-task-error` *Type URI* (defined below). Its payload shape is defined by this framework, independent of the originating specification.

A *recipient party* **MUST NOT** report failure by emitting a `#response`-variant document of the originating spec, nor success by emitting a `trust-task-error` document. The two reply types are not interchangeable.

### 8.1 The trust-task-error specification

The framework reserves the slug `trust-task-error` for the error-response *Trust Task specification* at:

```
https://trusttasks.org/spec/trust-task-error/<MAJOR.MINOR>
```

An *error response* is a *Trust Task document* whose `type` is the URI above. Its `payload` carries the standard error structure defined in [§8.2](#82-error-payload). The `id` member of an *error response* identifies the error instance and **MUST NOT** be reused; correlation back to the original task being responded to is carried by the framework's `threadId` member ([§4.9](#49-the-threadid-member)).

The *error response*'s `issuer` is the *consumer* that emitted it (the *reporting consumer* in the conformance language of the `trust-task-error` specification at [§8.6](#86-reserved-response-type-slugs)). Its `recipient` is the party the *consumer* wishes to inform of the failure. For most rejections — `expired`, `unsupportedType`, `unsupportedVersion`, `proofRequired`, `proofInvalid`, `taskFailed`, and the rest of [§8.3](#83-standard-error-codes) — that party is the *original producer* as carried in the rejected document's in-band `issuer` member.

The exception is `identityMismatch` (and any rejection raised in the same evaluation step that surfaced the mismatch): under such a rejection the rejected document's in-band `issuer` is by definition the contested identity, and **MUST NOT** be used as the error response's `recipient`. A *consumer* that emits an error response under `identityMismatch` **MUST** address the response to the transport-authenticated sender of the rejected document, and **MUST NOT** address it to the in-band `issuer`. Where no transport-authenticated sender is available, the *consumer* **SHOULD NOT** emit an error response at all — sending one to the contested in-band identity would constitute an oracle, and (in any transport that signs error responses) would compel the *consumer* to emit a signed document about a party that did not in fact participate in the exchange.

The *consumer* **MUST** likewise sanitize the `payload.message` member of an `identityMismatch` error response: a free-text message that reveals the *consumer*'s expected transport-authenticated identity, or the contested in-band value, leaks identity information to a possibly hostile sender (see [§10](#10-security-and-privacy-considerations)). The standard wire form for this code is the code identifier alone, optionally accompanied by a non-identifying message (e.g. `"identityMismatch: in-band identity does not match transport-derived identity"`).

### 8.2 Error payload

The `payload` of an *error response* has the following members. The correlation back to the *Trust Task document* this error reports on is carried at the framework level by the `threadId` member ([§4.9](#49-the-threadid-member)), which a *producer* of an error response **MUST** set.

`threadId` correlates the exchange for a party that saw the originating request. It identifies nothing to anyone else: it is opaque, and the payload otherwise names neither the *Trust Task specification* the failure occurred under nor the document instance that triggered it. A party handed a retained error — a verifier evaluating it as evidence, an auditor reconstructing a sequence — sees a `code` and a `retryable` flag and cannot tell what failed. For an extended code the slug namespace ([§8.5](#85-extension-by-individual-trust-task-specifications)) hints at the family; for the standard codes of [§8.3](#83-standard-error-codes) there is no signal at all.

The `inResponseTo` member closes that. A *consumer* emitting an error response **SHOULD** populate it, and **MUST** populate it where the error is intended to be retained, replayed, or relied upon by parties beyond the original *producer* — the same condition under which [§4.7.1](#471-when-to-include-a-proof) makes a `proof` mandatory, and for the same reason: an error that cannot be attributed *and* cannot be identified is not evidence of anything. Its `typeUri` carries the reported-on document's `type` including any `#request` or `#response` fragment, which is what tells a consumer whose semantics apply to an extended `code`; its `id` carries that document's *document identifier*, which [§4.3](#43-the-id-member) makes globally unique and never reused, so it names one instance where `threadId` names an exchange.

Under `identityMismatch` a *consumer* **SHOULD** omit `inResponseTo.id`: per [§8.1](#81-the-trust-task-error-specification) the response is addressed to the transport-authenticated sender rather than the in-band `issuer`, and that party did not necessarily compose the document whose identifier would be echoed.

| Member | Required | Type | Description |
|---|---|---|---|
| `code` | **MUST** | string | A short identifier for the failure category. **MUST** be one of the codes in [§8.3](#83-standard-error-codes) or an extended code as defined in [§8.5](#85-extension-by-individual-trust-task-specifications). |
| `inResponseTo` | **SHOULD** | object | Identifies the *Trust Task document* this error reports on: `typeUri` (its `type`, including any fragment) and `id` (its *document identifier*). See below. |
| `message` | **SHOULD** | string | A human-readable description of the error. Non-normative; intended for logs and operator UI. |
| `retryable` | **MUST** | boolean | `true` if the *producer* of the original document **MAY** retry the task; `false` if retrying with the same document or credentials is not expected to succeed. |
| `retryAfter` | **MAY** | string (date-time) | An [[RFC3339]] timestamp before which the *producer* **SHOULD NOT** retry. Meaningful only when `retryable` is `true`. |
| `details` | **MAY** | object | Task-specific extension data; see [§8.5](#85-extension-by-individual-trust-task-specifications). |

> **Example 5 — An error response** *(non-normative)*
>
> ```json
> {
>   "id": "9e2a1c44-7b81-4d3e-9b51-7a3c89e3d1f2",
>   "type": "https://trusttasks.org/spec/trust-task-error/0.2",
>   "threadId": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
>   "issuer": "did:web:bank.example",
>   "recipient": "did:web:verifier.example",
>   "issuedAt": "2026-05-16T14:22:00Z",
>   "payload": {
>     "code": "expired",
>     "message": "Task expired at 2026-04-12T09:31:00Z.",
>     "retryable": false
>   },
>   "proof": {
>     "type": "DataIntegrityProof",
>     "cryptosuite": "eddsa-rdfc-2022",
>     "verificationMethod": "did:web:bank.example#key-1",
>     "created": "2026-05-16T14:22:00Z",
>     "proofPurpose": "assertionMethod",
>     "proofValue": "z58D..."
>   }
> }
> ```

### 8.3 Standard error codes

The framework defines the error codes listed below. A *conforming consumer* **MUST** recognize each of these codes and **MUST** apply the corresponding semantics.

| Code | Meaning | Default `retryable` |
|---|---|---|
| `malformedRequest` | The document did not validate against the framework schema or the task-specific payload schema. | `false` |
| `unsupportedType` | The *consumer* does not recognize the `type` URI. | `false` |
| `unsupportedVersion` | The `type` URI was recognized but its `MAJOR.MINOR` version is not supported. | `false` |
| `expired` | The document's `expiresAt` was in the past at the time of evaluation. | `false` |
| `proofRequired` | A `proof` was required (by the *Trust Task specification* or *consumer* policy) and was missing. | `false` |
| `proofInvalid` | A `proof` was present but failed verification. | `false` |
| `permissionDenied` | The requesting *party* is not authorized to invoke this task. | `false` |
| `wrongRecipient` | The document's `recipient` does not identify the receiving *consumer*. | `false` |
| `identityMismatch` | An in-band `issuer` or `recipient` value is inconsistent with the corresponding transport-authenticated identity. | `false` |
| `idConflict` | The document's `id` matches one the *consumer* has already accepted, but its content differs — see [§7.2](#72-consumer-requirements) item 11. | `false` |
| `taskFailed` | The *recipient party* attempted the task and could not complete it; further detail **SHOULD** appear in `details`. | varies |
| `unavailable` | The *recipient party* is temporarily unable to process the task. | `true` |
| `internalError` | The *recipient party* encountered an unexpected internal failure. | `true` |

The "Default `retryable`" column gives the value an emitter of an error response **SHOULD** use unless task-specific knowledge dictates otherwise. The actual `retryable` value carried in a given *error response* is authoritative.

### 8.4 Retry semantics

In this specification, "retrying" means re-sending a *Trust Task document* bit-for-bit identical to the one that elicited the *error response* — same `id`, same `payload`, same `proof`. Issuing a *new* document, even one addressing the same underlying intent, is not a retry; see below.

A *party* that receives an *error response* **MUST NOT** retry the original *Trust Task document* if `retryable` is `false`. When `retryable` is `true`, the party **SHOULD** wait at least until any `retryAfter` value before retrying, and **SHOULD** apply backoff appropriate to the transport in use.

Retrying is safe for a *consequential Trust Task* precisely because [§7.2](#72-consumer-requirements) item 11 requires the *consumer* to absorb the duplicate rather than execute it again. The two rules are one mechanism seen from each end: this section tells a *producer* that the only safe resend is the bit-for-bit identical document, and item 11 tells a *consumer* that a bit-for-bit identical document it has already accepted **MUST NOT** produce a second consequential effect. A *producer* that "retries" by re-signing, re-stamping `issuedAt`, or otherwise altering the bytes has not retried — it has issued a different document under a reused `id`, which item 11 requires the *consumer* to reject with `idConflict` ([§8.3](#83-standard-error-codes)). Where a *producer* genuinely needs a fresh attempt, it issues a new document with a fresh `id`, as the paragraph below describes.

A `false` value of `retryable` represents a hard failure for that specific document. It does not prohibit the *producer* from issuing a *new* *Trust Task document* — that is, a document with a fresh `id` (and **SHOULD** the same `threadId` to preserve correlation) — addressing the cause of the failure. For example, after receiving an *error response* of `code = proofInvalid` with `retryable = false`, the *producer* **MUST NOT** re-send the failed document, but **MAY** issue a new document carrying a valid `proof`.

### 8.5 Extension by individual Trust Task specifications

An individual *Trust Task specification* **MAY** define additional error codes specific to its task. Extended codes **MUST** be namespaced, separated from the local code by a colon, e.g. `kyc-handoff:documentRevoked`. The namespace **MUST** be one of exactly two things:

1. **The emitting specification's own `<slug>`** — that is, the slug of the *request* the *error response* refers to. This is the default and covers any code the specification defines for itself.
2. **A *family namespace*** — a proper path prefix of that slug, formed of one or more of its leading `/`-separated segments (for `did-management/did/delete`, the permitted prefixes are `did-management/did` and `did-management`). A family namespace **MUST** be used only for a code whose meaning is defined once for the whole family — in a shared convention that the family's specifications reference — and never to give a specification-specific code a broader name than it has earned.

The namespace **MUST NOT** be the slug of a *related or referenced* specification, and this remains true under rule 2: a proper prefix of a specification's own slug names that specification's own family and can never name a sibling. A *consumer* of `acl/change-role` that needs to surface a rejection borrowed conceptually from `acl/revoke` therefore emits `acl/change-role:<local>` or `acl:<local>`, never `acl/revoke:<local>`. Extended codes **MUST NOT** shadow any code listed in [§8.3](#83-standard-error-codes).

*This paragraph is non-normative.* Rule 2 exists because families do share failure modes. Every specification under `did-management` can reject a request naming a domain the *consumer* does not host, and that rejection means the same thing in each of them; stating it once as `did-management:unknownDomain` lets a *consumer* handle the family uniformly, where per-slug codes would oblige it to enumerate every member to recognize one condition. The narrowness of rule 2 is what keeps this safe: because a family namespace is always a prefix of the emitting slug, a *consumer* can verify the namespacing of a received code against the document's `type` alone, with no registry lookup.

A *consumer* (not only the spec author) **MAY** mint additional namespaced codes for invariants the specification did not enumerate, provided the namespacing rule above is honoured. The framework's fallback-to-`taskFailed` rule for unrecognized extended codes (see the third paragraph below) keeps these consumer-minted codes interoperable with clients that only implement the canonical set.

An individual *Trust Task specification* **MAY** also define the structure of `details` for its own error responses. Where it does so, the specification **MUST** state which `code` values may carry a `details` object and **MUST** provide a JSON Schema fragment describing the `details` shape for each.

A *consumer* that does not recognize an extended `code` **SHOULD** treat the error as if its code were `taskFailed` and **MUST** still honor the `retryable` and `retryAfter` members.

The `details` member defined here is distinct from the `ext` extension member defined in [§4.5.1](#451-the-ext-extension-member). `details` carries *task-specific structured data tied to a specific error `code`*, defined by the spec author; its shape is constrained by the JSON Schema fragment the specification publishes for each carrying code. `ext` carries *vendor-namespaced extension data at payload or nested-object level*, defined by the ecosystem; its namespace structure is opaque to the framework. Both members **MAY** appear on the same *error response* and serve different purposes — implementations **MUST NOT** treat them as interchangeable.

> **Example 6 — An error response with an extended code and `details`** *(non-normative)*
>
> ```json
> {
>   "id": "c4d2f713-9a8e-4d04-b29c-2f1b0b4cbe71",
>   "type": "https://trusttasks.org/spec/trust-task-error/0.2",
>   "threadId": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
>   "issuer": "did:web:bank.example",
>   "recipient": "did:web:verifier.example",
>   "issuedAt": "2026-05-16T14:22:00Z",
>   "payload": {
>     "code": "kyc-handoff:documentRevoked",
>     "message": "Passport used in verification was revoked by the issuing authority on 2026-05-10.",
>     "retryable": false,
>     "details": {
>       "documentRef": "urn:passport:NL:XYZ123456",
>       "revokedAt": "2026-05-10T08:00:00Z"
>     }
>   }
> }
> ```
>
> A *consumer* implementing the `kyc-handoff` *Trust Task specification* interprets the extended `code` per that specification's declarations (see [§7.3](#73-specification-requirements), item 9). A *consumer* that does not implement `kyc-handoff` treats the error as if `code = taskFailed`, retains `retryable = false`, and ignores the contents of `details`.

### 8.6 Reserved response-type slugs

The framework reserves the following additional response-type *Trust Task specification* slugs. These slugs **MUST NOT** be used for any individual *Trust Task specification* registered under [§6.1](#61-type-uri).

| Slug | Purpose |
|---|---|
| `trust-task-ok` | Success with metadata — acknowledging that a task was performed and conveying any resulting references, receipts, or transient state. |
| `trust-task-next-step` | A recipient-suggested continuation — indicating that the original task was understood but cannot complete in isolation, together with the next *Trust Task* the *recipient party* expects in order to proceed. |

`trust-task-next-step` is published; its registry entry at `https://trusttasks.org/spec/trust-task-next-step/0.1` defines the normative `payload` shape and conformance requirements, in the same relationship to this section that the `trust-task-discovery` entry has to [§11](#11-discovery-and-capability-negotiation). A *next step* is a **third** disposition alongside the success response and the *error response* of this section: it reports that the originating task was understood and is **blocked**, leaving the exchange open where the other two close it. A *consumer* **MUST NOT** report a blocked task as an *error response*, nor a refusal as a *next step*; the three replies are not interchangeable. A *next step* confers no authorization — the *Type URI* it names is a suggestion the receiving party evaluates under its own policy, on the same advisory footing as a discovery response ([§11.4](#114-status-of-the-response)).

The payload structure of `trust-task-ok` remains out of scope for this revision and will be specified in a future revision of this framework. Implementations encountering a *Trust Task document* of a reserved type whose specification is not yet published **MAY** ignore the document or **MAY** return an `unsupportedVersion` *error response*.

## 9. Transport bindings

The framework deliberately leaves transport unconstrained ([§1.1](#11-design-goals) Goal 1): a *Trust Task document* can be conveyed over any channel that preserves its content. To make that composability work in practice, each transport protocol used to carry *Trust Task documents* **SHOULD** be accompanied by a *transport binding* specification.

A *transport binding* defines how *Trust Task documents* are exchanged over a specific transport — for example, DIDComm, the IETF Trust Spanning Protocol (TSP), HTTPS with mutual-TLS, AMQP, or paper. It is the integration layer between the framework's transport-agnostic semantics and the realities of a particular transport.

### 9.1 What a transport binding specifies

A *transport binding* **SHOULD** specify each of the following:

* **Document carriage.** How a *Trust Task document* is placed onto and retrieved from the transport (request body, message payload, envelope field, attachment, etc.).
* **Field population from transport context.** Which framework members the binding **derives** from transport-derived information — typically `issuer` (from a transport-authenticated sender), `recipient` (from a transport-authenticated addressee), and any signature metadata that lets a *consumer* verify the framework `proof` against transport-bound keys or, per [§4.7.1](#471-when-to-include-a-proof), accept the document without an in-band `proof`. Per [§4.8.1](#481-precedence-of-in-band-over-transport-derived-identity), the binding fills these members from the transport **only when the corresponding in-band member is absent**; when the in-band member is present, the transport-derived value is used as a cross-check, not as a substitute.
* **Consistency enforcement.** The behavior when an in-band framework member and its transport-derived equivalent disagree. The framework requires they **MUST** be consistent (see [§4.8](#48-the-issuer-and-recipient-members), [§4.8.1](#481-precedence-of-in-band-over-transport-derived-identity), and [§7.2](#72-consumer-requirements)); the binding states how the comparison is performed for the transport in question (for example, how a DID carried in-band is matched against a transport-authenticated DID).
* **Thread correlation (where the transport has its own).** Several transports carry their own correlation and parent-correlation identifiers — DIDComm's `thid` and `pthid`, for example. Where a binding maps these onto the framework's `threadId` ([§4.9](#49-the-threadid-member)) and `parentThreadId` ([§4.9.2](#492-the-parentthreadid-member)), it **MUST** state that mapping, and the mapping **MUST** require the two to agree only when **both** are explicitly present. The two layers identify different things and typically default into their own identifier spaces — a transport's correlation identifier commonly falls back to that transport's own message identifier, which is not the *Trust Task document*'s `id` — so requiring agreement unconditionally would fail exchanges that are otherwise conforming. As everywhere else in [§4.8.1](#481-precedence-of-in-band-over-transport-derived-identity), the in-band member is authoritative and the transport value is a cross-check; a *producer* **SHOULD** populate the transport's identifiers from the framework members rather than the reverse. A disagreement is a structural inconsistency and is reported as `malformedRequest`, not `identityMismatch` — no party's identity is in dispute.
* **Transport security profile.** The integrity, authentication, confidentiality, and freshness guarantees the transport provides, so that *consumers* can correctly evaluate the `proof` requirement under [§4.7.1](#471-when-to-include-a-proof). A *transport binding* from which any framework security or identity requirement is derived — any binding that populates `issuer` or `recipient` from transport context, or that addresses the omission of `proof` — **MUST** specify this profile; for such a binding the item is not optional. See [§9.1.1](#911-permitting-proof-to-be-omitted).
* **Error and response delivery.** How an *error response* ([§8](#8-error-responses)) is returned to the *producer* of the original document, including the behavior when the transport is fire-and-forget.

#### 9.1.1 Permitting `proof` to be omitted

[§4.7.1](#471-when-to-include-a-proof) permits a *Trust Task document* to omit `proof` where the transport already provides end-to-end integrity and authentication between *producer* and *consumer*. Whether a given transport does so is not a property a *consumer* can read off the transport's name: it depends on where the security boundary actually terminates, and on what intermediaries can do to a document in flight.

A *transport binding* that permits a *Trust Task document* to omit `proof` under [§4.7.1](#471-when-to-include-a-proof) **MUST** specify the security properties on which that allowance depends. That specification **MUST** address each of the following, and **MUST** state explicitly where an item does not apply to the transport rather than leaving it unaddressed:

1. **The authenticated producer.** Which credential or transport principal is authenticated, and by which mechanism.
2. **The mapping to a VID.** How that principal is deterministically mapped to the *VID* used for the framework's identity comparisons ([§4.8.1](#481-precedence-of-in-band-over-transport-derived-identity)).
3. **Audience binding.** How the intended *consumer* is identified or bound, and whether that binding is authenticated or merely asserted.
4. **Integrity across intermediaries.** What protects the document's bytes across every party that handles them.
5. **Re-origination.** Whether any intermediary can modify or re-originate the document without detection by the final *consumer*.
6. **Freshness and replay.** What protection, if any, the transport provides against a document being delivered more than once or out of time.
7. **Key and credential status.** Any revocation, expiry, or key-status assumption the allowance depends on.
8. **Where the guarantee stops.** Any condition — a routing mode, a mediator, a proxy, a store-and-forward queue — under which the transport ceases to provide producer-to-consumer end-to-end protection.

A *transport binding* **MUST NOT** state that `proof` may be omitted solely because an individual transport hop is authenticated, where an intermediary can modify or re-originate the *Trust Task document* without detection by the final *consumer*. Hop authentication tells the *consumer* who handed it the bytes, which is not the same fact as who composed them.

Where a binding's guarantees differ by mode — direct versus routed, nested versus not, mediated versus point-to-point — the allowance **MUST** be stated per mode. A single allowance covering a transport that has both an end-to-end mode and a relayed one overstates the weaker case.

**Silence is not permission.** A *transport binding* that does not address the omission of `proof` **MUST NOT** be read as permitting it, and a *consumer* operating over such a binding evaluates the `proof` requirement from [§4.7.1](#471-when-to-include-a-proof) and the *Trust Task specification* alone, as it would over a transport with no binding at all. A binding whose transport does not provide producer-to-consumer end-to-end integrity and authentication **SHOULD** say so plainly; that statement is as useful to an implementer as an allowance, and it is what stops a familiar transport name from being read as a guarantee it does not give.

### 9.2 The transport handler

An implementation that exchanges *Trust Task documents* over a given transport **SHOULD** expose its transport-binding logic as a discrete *transport handler* component:

1. On the **producer** side, the handler composes an outbound *Trust Task document*, **MAY** omit `issuer` and `recipient` where the transport will provide authenticated identity for those roles end-to-end (see [§4.8.1](#481-precedence-of-in-band-over-transport-derived-identity)), and applies the transport's signing or sealing where the binding integrates it with `proof`.
2. On the **consumer** side, the handler extracts an inbound *Trust Task document* from the transport, applies the [§4.8.1](#481-precedence-of-in-band-over-transport-derived-identity) precedence — using in-band `issuer` and `recipient` values where present (cross-checking them against transport-derived identity) and deriving them from the transport only where the in-band member is absent — and surfaces any inconsistencies as validation failures per [§7.2](#72-consumer-requirements).

The handler boundary lets the framework's validation logic remain transport-agnostic while different transports plug in their own population rules. A DIDComm handler can populate `issuer` from the verified sender DID of the surrounding DIDComm envelope; a TSP handler can do the same from the TSP message authentication; a mutual-TLS HTTPS handler can populate `issuer` from the peer certificate's subject; an unauthenticated transport handler populates nothing, and the framework falls back to the in-band `proof` per [§4.7.1](#471-when-to-include-a-proof).

A *transport binding* specification **SHOULD** identify itself by a stable URI and **SHOULD** declare which version of this framework it targets. The framework does not maintain a closed registry of *transport bindings*; new bindings **MAY** be published independently.

### 9.3 Binding namespace

A *transport binding* published through the framework's registry is identified by a URI in the `/binding/` subtree of the framework's authority:

```
https://trusttasks.org/binding/<slug>/<MAJOR.MINOR>
```

`<slug>` follows the same lowercase, hyphenated grammar as a Trust Task slug ([§6.1](#61-type-uri)); `<MAJOR.MINOR>` follows the version grammar of [§5.1](#51-scheme). Additional path segments under a binding URI — for example `https://trusttasks.org/binding/didcomm/0.1/envelope` — identify resources internal to the binding's own vocabulary (envelope `type` values, schema URIs, status mappings, and similar). Those segments are defined by the *transport binding* specification, not by this framework.

The `/binding/` subtree and the `/spec/` subtree of [§6.1](#61-type-uri) are **structurally disjoint**. A *Type URI* — the value carried in a *Trust Task document*'s `type` member ([§4.4](#44-the-type-member)) — is always rooted at `/spec/<slug>/<MAJOR.MINOR>` and **MUST NOT** be rooted at `/binding/...`. A *consumer* that receives a *Trust Task document* whose `type` is a URI under `/binding/` **MUST** reject it with `malformedRequest` per [§8.3](#83-standard-error-codes). The Type URI grammar of [§6.1](#61-type-uri) already excludes the `/binding/` path; this rule is stated explicitly so implementers and reviewers can see the namespace boundary at a glance and so that documents which somehow construct a `/binding/...` `type` value have a defined disposition rather than relying on grammar mismatch alone.

A *transport binding* specification published through the registry **SHOULD** live at `bindings/<slug>/<MAJOR.MINOR>/spec.md` in the framework's source tree, paralleling the `specs/<slug>/<MAJOR.MINOR>/` layout for *Trust Task specifications*. The grammar and content requirements for *transport binding* specifications are defined in [§9.1](#91-what-a-transport-binding-specifies).

The reservation rule of [§6.5](#65-private-and-unpublished-trust-task-specifications) — that private specifications **MUST NOT** be served from the `https://trusttasks.org/` authority — applies to private transport bindings equivalently: a private transport binding **MUST** use an authority the publisher controls and **MUST NOT** claim to identify a resource at `https://trusttasks.org/binding/...`.

## 10. Security and Privacy Considerations

*This section is non-normative in the current draft. Future revisions are expected to make portions of it normative as individual Trust Task specifications surface concrete requirements.*

A *Trust Task document* carries no inherent transport security. The framework's default rules for when an integrity proof is required of a document are given in [§4.7.1](#471-when-to-include-a-proof), and each *Trust Task specification* declares its own requirement under [§7.3](#73-specification-requirements). When `proof` is included, it **MUST** conform to the W3C *Data Integrity* format defined in [[VC-DATA-INTEGRITY]] (see [§4.7](#47-proof)); implementations select an appropriate cryptographic suite from the W3C-registered set based on the trust requirements agreed by the parties.

Personal data carried in a *Trust Task document* is visible to every *party* that handles the document. Individual *Trust Task specifications* **SHOULD** minimize personal data in their schemas to that strictly necessary to achieve the task's outcome, and **SHOULD** prefer references (e.g. DID URLs) to direct attribute values where the relying party is able to dereference them.

Because *Trust Task documents* are self-contained, a captured document remains evidence of its content after it has been delivered. Producers **SHOULD** consider whether the document's contents are appropriate for indefinite retention by the consumer.

### 10.1 Cross-recipient replay

A *Trust Task document* signed without an in-band `recipient` provides no cryptographic binding between the *producer*'s assertion and the intended audience. An attacker who obtains such a document — from a *consumer*'s storage, an intermediate cache, or an exfiltration — can replay the bytes to a different *consumer*; the proof verifies against the original *producer*'s VID, and a recipient who does not know the *producer*'s out-of-band intent has no signal that the assertion was not made to them. The audience-binding rule of [§4.8.2](#482-audience-binding) is the primary defence: when `proof` is present, `recipient` is also required in-band, and consumers reject any document that violates this rule with `malformedRequest`. *Bearer specifications* ([§4.8.3](#483-bearer-specifications)) are the only specifications for which a `proof`-carrying document without an in-band `recipient` is conformant; bearer status is an intentional, normative property of the specification, not a consumer-side flag.

Replay of the same document by the *original* recipient back into the same *consumer* (within transport bounds) is also possible. For a *consequential Trust Task* this is not merely a threat to be mitigated by local caching: [§7.2](#72-consumer-requirements) item 11 makes duplicate-execution protection a normative consumer requirement, keyed on the document `id` and bounded by the *consumer*'s acceptance window. The rule deliberately does not distinguish a hostile replay from a legitimate transport retry, because at the document layer the two are indistinguishable — the same bytes arriving twice. What matters for interoperability is that the second arrival does not repeat the effect, whichever it was.

Consumers handling assertions whose effect persists between exchanges but whose task is **not** consequential are outside item 11 and **SHOULD** still maintain such a cache for the lifetime of the assertion's relevance.

### 10.2 Parser hardening

A *consumer* deserializing untrusted JSON into a *Trust Task document* is exposed to the standard hazards of unbounded JSON parsing: deeply nested structures can cause stack overflow, large strings or arrays can exhaust memory, and integer overflows can occur on size fields. A *consumer* **SHOULD** bound the body size at the transport layer and **SHOULD** configure a maximum parse depth on the JSON deserializer. The framework does not mandate specific limits because they vary by deployment, but a depth limit of 128 levels and a body-size limit appropriate to the *Trust Task specification*'s payload (typically a few hundred kilobytes) are reasonable defaults.

### 10.3 Schema-validation DoS

A *consumer* that validates `payload` values against a JSON Schema obtained dynamically (for example, via [§6.2](#62-content-negotiation) over the network) **MUST** treat the schema as trusted only after authenticating its source. A maliciously-crafted schema can carry `pattern` regular expressions that exhibit catastrophic backtracking on otherwise-innocuous strings, causing the validator to consume unbounded CPU and effectively become a DoS oracle for any *producer* able to choose payload values. Consumers that compile schemas from arbitrary authorities **SHOULD** apply per-validation timeouts.

This consideration does **not** apply when the schema is embedded with the *consumer* at build time (for example, fetched from the registry once at release time, verified against [§6.4](#64-stability) immutability, and shipped as part of the consumer's binary). It does apply to dynamic-registry scenarios and to consumers that accept private specifications ([§6.5](#65-private-and-unpublished-trust-task-specifications)) over a runtime channel.

### 10.4 Error-response identity leakage

A *consumer* emitting an *error response* under [§8](#8-error-responses) **MUST** treat the error response's `payload.message` as a wire-exposed value. Free-text messages that reveal the *consumer*'s expected transport-authenticated identity, the contested in-band value of a mismatched party, or other consumer-internal state convert each error response into an identity-probing oracle for an unauthenticated *producer*. The rule for `identityMismatch` is stated in [§8.1](#81-the-trust-task-error-specification); the same principle applies to every standard code: error messages **SHOULD** be derived from the code identifier and the *Trust Task specification*'s public vocabulary, not from consumer-side authentication context.

## 11. Discovery and capability negotiation

Two parties about to enter a *Trust Task* exchange often need to negotiate a shared task vocabulary first: a *producer* asks "which *Trust Tasks* are you prepared to act upon?" before committing to send any particular document. The framework supports this with a reserved *Trust Task specification* of its own: `trust-task-discovery`.

The slug `trust-task-discovery` is reserved by [§6.1](#61-type-uri) under the framework's `trust-task-` namespace. Its current published version lives at:

```
https://trusttasks.org/spec/trust-task-discovery/0.1
```

Its registry entry defines the full request/response payload schema and conformance requirements. This section gives the framework-level overview; for the normative definitions of `payload.patterns` semantics, response shape, and conformance, see that registry entry.

### 11.1 Request

A *discovery request* is a *Trust Task document* whose `type` is `https://trusttasks.org/spec/trust-task-discovery/0.1`. Its `payload` carries an optional list of slug-glob patterns:

```json
{
  "patterns": ["acl/*", "kyc-handoff"]
}
```

When `patterns` is absent or empty, the *responder* treats the query as `["*"]` — return every supported *Trust Task*.

### 11.2 Pattern grammar

Patterns are deliberately coarse. The grammar is:

* `"*"` — matches every slug.
* `"<prefix>/*"` — matches every slug whose value starts with the literal `<prefix>/` (e.g. `"acl/*"` matches `acl/grant`, `acl/revoke`, and `acl/grant/sub`).
* `"<slug>"` — exact match.

Wildcards in positions other than as the trailing `/*` of a `<prefix>/*` pattern are **not** interpreted; they match literally. Multiple patterns combine with **OR** semantics: a slug matches the query if it matches at least one pattern.

The grammar omits version filters, recursive globs (`**`), and regex on purpose. Versions are part of the *Type URI* the responder returns; a discoverer that needs to filter on version applies the constraint client-side.

### 11.3 Response

A *discovery response* is a *Trust Task document* whose `type` is `https://trusttasks.org/spec/trust-task-discovery/0.1#response`. Its `payload` carries the matching subset of *Type URIs* the responder supports:

```json
{
  "supportedTypes": [
    "https://trusttasks.org/spec/acl/grant/0.1",
    "https://trusttasks.org/spec/acl/revoke/0.1",
    "https://trusttasks.org/spec/kyc-handoff/1.0"
  ]
}
```

Each entry is a **bare** *Type URI* — no `#request` or `#response` fragment. A *Type URI*'s presence in `supportedTypes` means the responder handles both directions of that specification's exchange.

A response with `"supportedTypes": []` is conformant and means "I support nothing matching your query."

### 11.4 Status of the response

A *discovery response* is **advisory**. A *Type URI*'s presence is a hint that the responder will accept a *Trust Task document* of that type, not a binding commitment: the responder may have revoked support, may apply per-document permissions, or may itself receive a `proofInvalid` or `permissionDenied` at the point of acting on a subsequent request. Every subsequent exchange runs the full [§7.2](#72-consumer-requirements) pipeline; discovery only narrows what the discoverer chooses to send.

### 11.5 Privacy considerations

A discovery response leaks information about which specifications the responder implements. Responders that consider their supported task set sensitive **SHOULD** authenticate the discoverer before responding, and **MAY** return a filtered subset of their true capabilities (or no response at all) when the discoverer is unknown or unauthenticated. See the discovery spec's "Privacy considerations" section for additional discussion.

## 12. References

### 12.1 Normative references

* **[RFC2119]** Bradner, S. *Key words for use in RFCs to Indicate Requirement Levels*. RFC 2119, March 1997. <https://www.rfc-editor.org/rfc/rfc2119>
* **[RFC8174]** Leiba, B. *Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words*. RFC 8174, May 2017. <https://www.rfc-editor.org/rfc/rfc8174>
* **[RFC3339]** Klyne, G., Newman, C. *Date and Time on the Internet: Timestamps*. RFC 3339, July 2002. <https://www.rfc-editor.org/rfc/rfc3339>
* **[RFC3986]** Berners-Lee, T., Fielding, R., Masinter, L. *Uniform Resource Identifier (URI): Generic Syntax*. RFC 3986, January 2005. <https://www.rfc-editor.org/rfc/rfc3986>
* **[RFC8259]** Bray, T. (Ed.). *The JavaScript Object Notation (JSON) Data Interchange Format*. RFC 8259, December 2017. <https://www.rfc-editor.org/rfc/rfc8259>
* **[RFC9110]** Fielding, R. (Ed.), Nottingham, M. (Ed.), Reschke, J. (Ed.). *HTTP Semantics*. RFC 9110, June 2022. <https://www.rfc-editor.org/rfc/rfc9110>
* **[RFC8785]** Rundgren, A., Jordan, B., Erdtman, S. *JSON Canonicalization Scheme (JCS)*. RFC 8785, June 2020. <https://www.rfc-editor.org/rfc/rfc8785>
* **[RFC9562]** Davis, K. (Ed.), Peabody, B., Leach, P. *Universally Unique IDentifiers (UUIDs)*. RFC 9562, May 2024. <https://www.rfc-editor.org/rfc/rfc9562>
* **[VC-DATA-INTEGRITY]** Longley, D., Sporny, M. *Verifiable Credential Data Integrity 1.0*. W3C Recommendation. <https://www.w3.org/TR/vc-data-integrity/>
* **[ABNF]** Crocker, D., Overell, P. *Augmented BNF for Syntax Specifications: ABNF*. RFC 5234, January 2008. <https://www.rfc-editor.org/rfc/rfc5234>
* **[JSON-SCHEMA-2020-12]** Wright, A. et al. *JSON Schema: A Media Type for Describing JSON Documents*. Draft 2020-12. <https://json-schema.org/draft/2020-12/schema>
* **[DID-CORE]** Sporny, M., Longley, D., Sabadello, M., Reed, D., Steele, O., Allen, C. *Decentralized Identifiers (DIDs) v1.0*. W3C Recommendation. <https://www.w3.org/TR/did-core/>

### 12.2 Informative references

* **[VC-DATA-MODEL]** Sporny, M. et al. *Verifiable Credentials Data Model v2.0*. W3C Recommendation. <https://www.w3.org/TR/vc-data-model-2.0/>
* **[W3C-MANUAL-OF-STYLE]** W3C. *Manual of Style*. <https://www.w3.org/guide/manual-of-style/>

## 13. Acknowledgments

The editor thanks the members of the Trust Over IP Foundation Decentralized Trust Graph Working Group for their ongoing review and contributions to this specification.

## Appendix A — Example Trust Task specification

*This appendix is non-normative.*

This appendix shows the elements an individual *Trust Task specification* declares in order to satisfy [§7.3](#73-specification-requirements). The example below is illustrative; the slug `kyc-handoff` and its contents are used purely for demonstration and are not a reference to any actual specification registered under [§6.1](#61-type-uri).

### A.1 Front matter

| Declaration | Value |
|---|---|
| Slug | `kyc-handoff` |
| Version | `1.0` |
| *Type URI* | `https://trusttasks.org/spec/kyc-handoff/1.0` |
| Target framework version | `0.1` |
| Maturity level | `draft` |
| `issuer` party | The KYC verifier. **REQUIRED**. Accepted *VID* schemes: `did:web`, `did:key`, `x509`. |
| `recipient` party | The relying party (typically a bank). **REQUIRED**. Accepted *VID* schemes: `did:web`, `x509`. |
| Outcome | The *issuer* attests to the *recipient* the result and assurance level of a KYC verification performed against an identified subject. |
| Proof requirement | **REQUIRED**. Rationale: the recipient retains the verification result for compliance reporting and may rely upon it after delivery; a transport-bound integrity guarantee alone is insufficient (see [§4.7.1](#471-when-to-include-a-proof)). |
| JSON-LD `@context` | Not published at this version. |

### A.2 Payload JSON Schema

Served at the *Type URI* under content negotiation for `application/schema+json`:

```json
{
  "$id": "https://trusttasks.org/spec/kyc-handoff/1.0",
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "additionalProperties": false,
  "required": ["subject", "result", "level"],
  "properties": {
    "subject": {
      "type": "string",
      "description": "Verifiable Identifier of the verified subject."
    },
    "result": {
      "type": "string",
      "enum": ["passed", "failed"]
    },
    "level": {
      "type": "string",
      "enum": ["LOA1", "LOA2", "LOA3"]
    }
  }
}
```

### A.3 Task-specific error codes

| Code | Meaning | Default `retryable` | `details` shape |
|---|---|---|---|
| `kyc-handoff:documentRevoked` | A breeder document used in the verification was revoked by its issuing authority after the verification completed. | `false` | `{ "documentRef": <string>, "revokedAt": <RFC3339 date-time> }` |

The `details` JSON Schema fragment for this code is:

```json
{
  "type": "object",
  "additionalProperties": false,
  "required": ["documentRef"],
  "properties": {
    "documentRef": { "type": "string" },
    "revokedAt":   { "type": "string", "format": "date-time" }
  }
}
```

### A.4 An example conforming document

```json
{
  "id": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/kyc-handoff/1.0",
  "issuer": "did:web:verifier.example",
  "recipient": "did:web:bank.example",
  "issuedAt": "2026-04-12T09:31:00Z",
  "expiresAt": "2027-04-12T09:31:00Z",
  "payload": {
    "subject": "did:key:z6Mk...",
    "result": "passed",
    "level": "LOA2"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-rdfc-2022",
    "verificationMethod": "did:web:verifier.example#key-1",
    "created": "2026-04-12T09:31:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3kg..."
  }
}
```

This document carries a `proof` member because the specification declares `proof` as **REQUIRED** in §A.1. A *consumer*:

1. Resolves the document's `type` URI to learn the *target framework version* (`0.1`) and fetches the framework schema at `https://trusttasks.org/spec/trust-task/0.1`. The outer document structure is validated against it.
2. Fetches the payload schema at the same `type` URI under content negotiation for `application/schema+json`. The `payload` is validated against it.
3. Verifies the `proof` per [§4.7](#47-proof) against the *VID* in `issuer`.
4. Confirms `expiresAt` is in the future and `recipient` matches the consumer's own *VID*.

If any step fails, the *consumer* returns an *error response* per [§8](#8-error-responses).

## Appendix B — Changelog

*This appendix is non-normative.*

### 0.4

* **The `ceremony` member ([§4.11](#411-the-ceremony-member)).** A *Trust Task document* **MAY** now record that it is one step of a *Trust Ceremony* — a flow composed of several *Trust Tasks*. The framework has always modelled multi-party work as multiple bilateral tasks ([§2](#2-terminology)); what it lacked was a way for the collection to be named, identified, and evidenced, so every implementation held that knowledge in application code and no two could interoperate above the level of a single task. The member carries the *enactment* (globally unique and non-reusable, unlike `threadId`, because evidence about a flow needs a stable anchor), the step's name, an optional content-pinned reference to a published *ceremony definition*, and an optional set of predecessor digests.

    Three properties are deliberate. It is **carried on the document rather than in `payload`**, so no *Trust Task specification* changes and any existing task may be composed into a flow its author never anticipated. It is **covered by `proof`**, so a step cannot be lifted into a different enactment or reinterpreted under a different definition. And it **confers no authority** ([§4.11.4](#4114-membership-is-a-claim-not-a-permission), [§7.2](#72-consumer-requirements) item 9) — membership is an assertion by the *issuer*, not a verified fact, which is what makes it safe for a *consumer* to ignore the member entirely.

    Additive: the document wire format gains an optional member, and every document conforming to 0.3 still conforms.

* **The `/ceremony/` subtree and the `trust-ceremony` reservation ([§6.7](#67-ceremony-namespace), [§6.1](#61-type-uri)).** *Ceremony definitions* are identified in a third subtree under the framework's authority, structurally disjoint from `/spec/` and `/binding/` on the same terms — no URI under it is a *Type URI*, and a document whose `type` is rooted there is malformed. The slug reservation of §6.1 widens from `^trust-task($|-|/)` to `^trust-(task|ceremony)($|-|/)`; the new half is unused at this version and exists so the namespace cannot be claimed by another party before the layer that needs it is specified.

    The **content** of a ceremony definition is out of scope for this revision. This version defines where definitions live, how a step references one, and that the reference is by content as well as by name — a URI alone would leave a flow's rules mutable by whoever controls the URI, retroactively and for every enactment already performed.

* **Authorization is distinct from identity and proof ([§7.2](#72-consumer-requirements) item 10, [§7.3](#73-specification-requirements) item 15).** A *consumer* **MUST NOT** treat successful validation of a *VID*, `issuer`, `recipient`, transport-derived identity, or `proof` as establishing that anyone is authorized to request or perform the task, and **MUST** evaluate authorization separately before executing. The framework already said this twice in narrow forms — ceremony membership grants nothing ([§4.11.4](#4114-membership-is-a-claim-not-a-permission), [§7.2](#72-consumer-requirements) item 9), and the side-effect and exposure classes describe without authorizing ([§7.3](#73-specification-requirements) items 13 and 14) — but never for an ordinary task, leaving an implementer free to read *valid proof + recognized issuer + correct recipient* as an authorized instruction. That inference is the same confused-deputy vector §4.11.4 forecloses, and it is most dangerous where the *producer* is an agent that can prove its identity but holds no authority to act.

    The rule is deliberately **model-neutral**: it requires that an authorization decision be made, not how. A verified assertion may still *be* the authorization where a specification defines that role and the *consumer*'s policy accepts it — the `task-consent` design — but that is now an explicit declaration under item 15 rather than an available default.

    Additive: the wire format is unchanged and every document conforming to 0.3 still conforms. A *consumer* that already separated authorization from validation needs no change.

* **Validity during execution ([§7.2](#72-consumer-requirements) item 12, [§7.3](#73-specification-requirements) item 16, [§4.2](#42-top-level-members)).** Validating a document established that it was eligible for processing *at the instant it was validated*. For execution that is delayed, long-running, resumed, or agentic, that instant and the instant a consequential effect actually lands can be far apart — and the authority in between can evaporate. A *consumer* **MUST** now re-evaluate, immediately before each irreversible or externally visible effect, every condition its policy and the *Trust Task specification* require: delegation, mandate, capability, membership, standing, credential or key status, subject relationship. `task-consent/decision/0.1` already required this locally, re-checking policy and approver enrolment so a device revoked during the approval window cannot carry a task through; item 12 makes the general case normative.

    **The rule is about authority, not the clock.** `expiresAt` is deliberately excluded, and §4.2 now says plainly that it bounds *acceptance* and does not abort work under way. Re-checking it mid-execution would turn a statement about a request's staleness into an execution timeout the *producer* never set and could not compute, since it does not know how long the *consumer*'s work takes. A task with a genuine completion deadline declares it in its own `payload`, where the specification can define what lapsing means — as `task-consent/request/0.1` already does — and item 12 then re-evaluates it like any other condition. No new framework member was needed.

    **Stopping is not automatically safe.** Once an irreversible effect has occurred, declining the next one does not undo it, and abandoning a partly applied change can leave the *recipient party* in a state neither party asked for. Item 12 sits *before* each effect for that reason, §7.3 item 16 asks multi-stage specifications to say when partial application is unsafe, and a *consumer* that does stop **SHOULD** report partial execution distinguishably from never having begun — a *producer* that cannot tell the two apart cannot decide whether to reissue.

* **A transport binding must justify any allowance to omit `proof` ([§9.1](#91-what-a-transport-binding-specifies), [§9.1.1](#911-permitting-proof-to-be-omitted)).** §4.7.1 has always let a document omit `proof` where the transport provides end-to-end integrity and authentication between *producer* and *consumer*, but §9.1 only **SHOULD**ed the security profile that would establish whether it does. A binding could therefore rest a proof allowance on "the transport is authenticated" without saying which party is authenticated, how that principal becomes a VID, what an intermediary can do to the bytes, or where the guarantee stops — leaving *consumers* to infer a security boundary from a transport's name.

    The profile is now **MUST** for any binding from which a framework security or identity requirement is derived, and a binding permitting omission must address eight specific properties, saying explicitly where one does not apply. Two rules carry most of the weight: hop authentication is **not** producer authentication where an intermediary can re-originate undetected, and an allowance **MUST** be stated per mode where a transport has both an end-to-end mode and a relayed one.

    **Silence is not permission.** A binding that does not address omission is not to be read as permitting it. A binding whose transport genuinely cannot offer the guarantee **SHOULD** say so plainly — that statement is as useful as an allowance, and it is what stops a familiar transport name from being taken for a guarantee it does not give.

* **Duplicate-execution protection ([§7.2](#72-consumer-requirements) item 11, [§8.3](#83-standard-error-codes), [§8.4](#84-retry-semantics)).** A *consumer* **MUST NOT** let the same *Trust Task document* cause a consequential effect twice. The pieces were all present — unique `id`s (§4.3), audience binding (§4.8.2), bit-for-bit retry (§8.4), and an idempotency cache recommended in §10.1 — but the last of those was a **SHOULD** in a non-normative section, so two conforming consumers could both validate a repeated document correctly and one of them execute the transfer, the deletion, or the key rotation a second time. The framework had strong document-identifier uniqueness and no execution uniqueness to match it.

    The rule keys on the document `id` and is deliberately blind to intent: at the document layer a hostile replay and a legitimate transport retry are the same bytes arriving twice, and what matters is that the second arrival does not repeat the effect. Three questions the requirement has to answer, and does: a *consumer* retains a **digest**, not just an `id`, because an `id` alone cannot tell the retry it must absorb from the conflict it must reject; retention is bounded by the **same window** over which the *consumer* will still execute the document, so the rule never demands unbounded memory; and where a specification defines no success response, the duplicate is simply not executed and the silence is correct rather than an error.

    §8.4 gains the other half of the story — retry is safe *because* item 11 absorbs the duplicate — and records that a *producer* that re-signs or re-stamps has not retried but has issued a different document under a reused `id`.

* **`idConflict` ([§8.3](#83-standard-error-codes)).** A new standard error code for a document whose `id` matches one already accepted but whose content differs. Distinguishing this from a retry is the point: a retry is absorbed silently, a conflict is refused. Consumers at earlier framework versions will not recognize the code; `trust-task-error/0.4` carries it.

* **The term *consequential Trust Task* ([§2](#2-terminology)).** The predicate `sideEffects.level ∈ {mutating, destructive} ∨ exposure.discloses = secret ∨ exposure.actsAsSubject = true` is now named once rather than re-spelled at each use, with the fail-safe reading of an absent or unresolvable declaration folded into the definition. No new obligation attaches to the term itself.

* **`trust-task-next-step` published ([§8.6](#86-reserved-response-type-slugs)).** The continuation response reserved since 0.1 now has a registry entry defining its payload. A *next step* is a **third** disposition alongside the success response and the *error response*: the two of those close the originating task, and a next step leaves it **open**. A *consumer* **MUST NOT** report a blocked task as an error, nor a refusal as a next step. `trust-task-ok` remains reserved.

### 0.3

* **Error responses can identify what failed ([§8.2](#82-error-payload)).** The error payload gains an optional `inResponseTo` member carrying the reported-on document's `type` and `id`. Previously an *error response* was correlated only by `threadId`, which means something to a party that saw the originating request and nothing to anyone else — so an error retained as evidence named neither the task it terminated nor the instance, and for the standard codes of §8.3 carried no signal of origin at all. **SHOULD** in general, **MUST** where the error will be relied upon beyond the original *producer*. Published as `trust-task-error/0.3`; optional in this version so a 0.2 consumer's output remains valid, with a future major version expected to require it and 0.1/0.2 retired once consumers have moved.
* **Per-variant proof requirements ([§7.3](#73-specification-requirements) item 8).** A *Trust Task specification* may now declare the `proof` requirement for its *request* and *response* variants separately, rather than one value covering both. The two are relied upon differently — a response retained as evidence outside the original exchange can need a proof where the request that triggered it does not, and a request that destroys state needs attribution where its acknowledgement protects nothing — and a single value forces the stricter onto both. The single form remains valid and unchanged; where the per-variant form omits the *response*, the *request*'s value applies, so an omission cannot weaken a variant. The error variant stays undeclarable: an error response's `type` names `trust-task-error`, a different specification, so a declaration here could not reach it. Additive — every existing declaration keeps its meaning.
* **The `parentThreadId` member ([§4.9.2](#492-the-parentthreadid-member)).** A *Trust Task document* **MAY** now carry the `threadId` of the exchange that contains it, so a party holding a document from a nested exchange can find the exchange it was conducted within — something a flat `threadId` cannot express, and which specifications were otherwise forced to invent per-family payload conventions for. It takes `threadId`'s posture: optional, no normative validation semantics, consumers **MUST NOT** reject on it alone. It records one level of containment deliberately, rather than half-defining an ancestry chain. Where a transport carries its own parent-thread concept the two **MUST** agree when both are present, with the in-band member authoritative. Additive: the document wire format gains an optional member, and every document conforming to 0.2 still conforms.
* **Naming an exchange from outside the framework ([§4.9.1](#491-naming-an-exchange-from-outside-the-framework)).** Added the rule that anything referring to an exchange as evidence of an event — a credential citing the exchange that established what it attests, an audit record, a governance decision — **MUST** name the *innermost* exchange whose documents attest that event, by the `id` of the document that initiated it. A `threadId` names one exchange and expresses no containment, so where exchanges nest, more than one thread is open when an event occurs and only one attests it; naming an enclosing exchange collects evidence of the wrong event. Clarification only — no member is added and no existing behaviour changes.
* **Family namespaces for extended error codes ([§8.5](#85-extension-by-individual-trust-task-specifications)).** The namespace of an extended `code` may now be either the emitting specification's own slug (as before) or a *family namespace* — a proper path prefix of that slug — for a condition whose meaning is defined once across a family in a shared convention, such as `did-management:unknownDomain` on every `did-management/*` specification. Previously the namespace **MUST** have equalled the slug exactly, which gave a family-wide failure mode no way to be named once; specifications expressed it anyway, so the rule was already being broken to say something true. The relaxation is deliberately narrow: because a family namespace is always a prefix of the emitting slug, a *consumer* can still verify a received code's namespacing against the document's `type` alone, and a *sibling's* slug remains forbidden. Additive — every previously conforming code remains conforming. The prefix relationship is now enforced by the registry build, which never checked the original rule either.
* **Draft editorial changes stay in place ([§5.2](#52-compatibility-rules)).** An editorial or normalization change to a `draft` artifact — casing normalization per [§4.10](#410-naming-conventions), a framework or shared-schema-component `$ref` re-pin with no wire effect, prose rewording — is now made in place and **MUST NOT** mint a new version. A wire-identical version minted before this rule **MAY** declare the new optional `wireCompatibleWith` front-matter field naming its predecessor, so consumers can dual-accept by mechanical normalization.
* **Side-effect and exposure classes ([§7.3](#73-specification-requirements) items 13–14).** Every conforming specification now **MUST** declare two orthogonal, descriptive classifications of what executing the task does: a *side-effect class* (`none` / `mutating` / `destructive` — the integrity effect on recipient state) and an *exposure class* (`discloses` of `none` / `metadata` / `secret`, plus an `actsAsSubject` flag — the confidentiality and agency effect). Both are descriptive only — a specification **MUST NOT** derive a consent requirement from them — and exist so a delegated-execution consumer can decide whether to seek human approval without per-task code. This is a breaking change to the specification-authoring contract, carried by the internal `spec-meta/2.0` front-matter meta-schema; the **document wire format is unchanged from 0.2**, so `targetFrameworkVersion` and document validation are unaffected and specifications keep their existing framework-version targets.

### 0.2

* **Naming conventions ([§4.10](#410-naming-conventions)).** Added a normative section defining casing: framework-defined members and values use **lowerCamelCase**; payload member names and specification-defined enumerated values **SHOULD** use lowerCamelCase; externally-owned values (WebAuthn, JOSE, `SameSite`, W3C *Data Integrity*, …) are carried verbatim.
* **Standard error codes re-cased ([§8.3](#83-standard-error-codes)).** The standard error `code` identifiers are now lowerCamelCase: `malformedRequest`, `unsupportedType`, `unsupportedVersion`, `proofRequired`, `proofInvalid`, `permissionDenied`, `wrongRecipient`, `identityMismatch`, `taskFailed`, `internalError` (the single-word codes `expired`, `unavailable` are unchanged). This is a breaking change carried by `trust-task-error/0.2`; the snake_case `0.1` codes remain valid for documents whose `type` resolves to a `0.1` specification.
* **Shared schema components ([§6.6](#66-shared-schema-components)).** Added a section giving shared schema fragments first-class, independently-versioned status, with a mandatory version-pinning rule and the schema/specification version-coupling rule.
* **Migration guidance ([§5.4](#54-migrating-between-versions)).** Added the non-normative receiver-before-sender (expand/contract) migration sequence and the coupling of schema and specification versions.
* **Draft version caveat ([§5.2](#52-compatibility-rules)).** Clarified that a breaking change to a `draft` artifact MAY be released as a `MINOR` increment.
* Affected `0.1` specifications were re-published as `0.2` with lowerCamelCase enumerated values; `0.1` remains served unchanged for backwards compatibility and will be `retired` once consumers have migrated.

### 0.1

* Initial working draft of the Trust Tasks framework.
