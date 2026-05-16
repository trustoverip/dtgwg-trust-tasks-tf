# Trust Tasks

**Trust over IP Foundation — DTGWG Task Force**

| | |
|---|---|
| **Document version** | 0.1 |
| **Date** | 2026-05-16 |
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

This is a **Working Draft** prepared by the Trust Tasks Task Force of the Decentralized Trust Graph Working Group (DTGWG) of the [Trust over IP Foundation](https://trustoverip.org). It has been produced by the editor listed above and has not yet been reviewed or endorsed by the DTGWG as a whole. Publication as a Working Draft does not imply endorsement by the Trust over IP Foundation membership.

Comments on this document are welcome via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues). The editor expects substantive sections — in particular [§7. Minimum Requirements](#7-minimum-requirements), [§8. Error responses](#8-error-responses), [§9. Transport bindings](#9-transport-bindings), and [§10. Security and Privacy Considerations](#10-security-and-privacy-considerations) — to evolve as individual Trust Task specifications progress through [§5.3 Maturity levels](#53-maturity-levels) and surface gaps in this framework.

This document is governed by the [Trust over IP Foundation Patent and Copyright Grants](CONTRIBUTING.md).

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
11. [References](#11-references)
12. [Acknowledgments](#12-acknowledgments)

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

4. **A standard family of response types.** Many tasks need a structured way for a *recipient party* to report what happened. The framework reserves a small set of response-type *Trust Task specifications* addressing the common cases — failure ([§8](#8-error-responses)), success with metadata (`trust-task-ok`), and a recipient-suggested continuation (`trust-task-next-step`) — each itself a *Trust Task* so that one validation, signing, and transport pipeline serves both the task and its response. Only the failure case is fleshed out in this revision; the others are reserved (see [§8.6](#86-reserved-response-type-slugs)) and will be specified in a future revision.

## 2. Terminology

The key terms in this document are defined here. Where a term is *italicized* on subsequent use, the definition in this section applies.

* *Trust Task* — A self-contained, transport-agnostic, JSON-based specification for verifiable work between two or more parties.
* *Trust Task document* — A single JSON object that is an instance of a *Trust Task*. It carries its own type, version, and identifier alongside a task-specific *payload*. The structure is defined normatively in [§4](#4-trust-task-documents).
* *Party* — An entity that participates in a *Trust Task*. Each party is identified by a *Verifiable Identifier*.
* *Verifiable Identifier (VID)* — A string identifier whose controller is verifiable under a trust framework. Decentralized Identifiers (DIDs) [[DID-CORE]] are one realization of VIDs; others include X.509 subjects, OIDC subject identifiers, and key thumbprints. The framework does not constrain the VID scheme; the *consumer*'s trust framework determines which schemes are accepted and how each is resolved to verification material.
* *Producer* — A *party* that emits a *Trust Task document*.
* *Consumer* — A *party* that receives and processes a *Trust Task document*.
* *Recipient party* — A *consumer* in its role of evaluating whether to accept and act upon a *Trust Task document*. The term emphasizes the consumer's responsibility for acceptance; the two terms refer to the same entity.
* *Trust Task specification* — A document, conforming to this framework, that defines a single *Trust Task* — its parties, normative payload schema, and conformance rules.
* *Document identifier* — The string carried in the `id` member of a *Trust Task document* that uniquely identifies that instance.
* *Thread identifier* — An optional string carried in the `threadId` member that correlates a *Trust Task document* with other documents belonging to the same logical exchange. See [§4.9](#49-the-threadid-member).
* *Payload* — The task-specific portion of a *Trust Task document*, carried in the `payload` member. Its internal structure is defined by the *Trust Task specification* identified by the document's `type`.
* *Type URI* — A URI that identifies a *Trust Task specification* at a specific version and serves as the single resolvable namespace for that version. The canonical form is defined in [§6.1](#61-type-uri).
* *Proof* — An optional, integrity-providing object attached to a *Trust Task document*. The framework reserves the `proof` member name; concrete proof formats are out of scope at this revision (see [§4.7](#47-proof)).
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
| `type` | **MUST** | string (URI) | The *Type URI* identifying the *Trust Task specification* and version this document conforms to. See [§4.4](#44-the-type-member). |
| `issuer` | **MAY** | string (VID) | A *Verifiable Identifier* identifying the *party* responsible for the document's content. See [§4.8](#48-the-issuer-and-recipient-members). |
| `recipient` | **MAY** | string (VID) | A *Verifiable Identifier* identifying the *party* the *issuer* expects to act upon the document. See [§4.8](#48-the-issuer-and-recipient-members). |
| `issuedAt` | **SHOULD** | string (date-time) | An [[RFC3339]] timestamp recording when the document was produced. |
| `expiresAt` | **MAY** | string (date-time) | An [[RFC3339]] timestamp after which the document is no longer valid. Where `expiresAt` is specified, the *recipient party* **MUST** honor the expiry: a *consumer* **MUST NOT** act upon a document whose `expiresAt` lies in the past. See [§7.2](#72-consumer-requirements). |
| `payload` | **MUST** | object | The task-specific body. Its internal structure is governed by the *Trust Task specification* identified by `type`. See [§4.5](#45-the-payload-member). |
| `@context` | **MAY** | string \| array | If present, enables JSON-LD processing of the document. See [§4.6](#46-json-ld-compatibility). |
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
> The `payload` member is the only part whose internal shape is defined by the per-task specification; everything else is framework-defined.

### 4.3 The `id` member

The `id` member's value **MUST** be a string that is globally unique to this instance of the task. The framework places no further constraint on its form: UUIDv4 [[RFC9562]] is **RECOMMENDED** as a low-coordination default that requires no namespace ownership, but any string an implementation can guarantee unique is permitted — for example, a DID URL, a UUIDv7, a URN, or an opaque content-addressed identifier. Producers **MUST NOT** reuse an `id` value across documents.

The `id` is opaque to the framework. Resolvability of the `id` (the ability to dereference it back to the document) is not required. Where resolvability is needed for a particular use case, an individual *Trust Task specification* or transport binding **MAY** require a more specific form (for example, a DID URL).

### 4.4 The `type` member

The `type` member's value **MUST** be a *Type URI* in the form defined in [§6.1](#61-type-uri). The version of the *Trust Task specification* a document conforms to is conveyed by the trailing `<MAJOR.MINOR>` segment of this URI; no separate version member is carried in the document.

### 4.5 The `payload` member

The `payload` member's value **MUST** be a JSON object whose internal structure is defined by the *Trust Task specification* identified by the document's `type`. This framework places no constraint on the contents of `payload` beyond requiring that it be an object.

The framework separates document-level metadata (`id`, `threadId`, `type`, `issuer`, `recipient`, `issuedAt`, `expiresAt`, `proof`) from task-specific data (`payload`) so that a single framework-level schema validates the outer structure, with per-task schemas applied only to `payload`. Schema scope is defined in [§6.3](#63-schema-scope).

### 4.6 JSON-LD compatibility

A *Trust Task document* **MAY** include an `@context` member. If present, the document **SHALL** be processable as JSON-LD; the framework places no further constraint on the contents of `@context`. A *Trust Task specification* that wishes to declare a canonical JSON-LD context **MUST** publish it at its *Type URI* under content negotiation for `application/ld+json` (see [§6.2](#62-content-negotiation)).

A *consumer* that does not implement JSON-LD processing **MUST** ignore the `@context` member and process the document as plain JSON. JSON-LD support is therefore strictly additive — no consumer is required to implement it, and no document is required to include it.

### 4.7 Proof

A *Trust Task document* **MAY** include a `proof` member whose value is a W3C *Data Integrity Proof* object as defined in [[VC-DATA-INTEGRITY]]. When present, the `proof` binds the document's content to its `issuer`.

The choice of cryptographic suite is open: any suite registered by the W3C Verifiable Credential Working Group (for example, `eddsa-rdfc-2022` or `ecdsa-rdfc-2019`, or any future suite) **MAY** be used. The `verificationMethod` of the proof **MUST** resolve to verification material controlled by the *party* identified by the document's `issuer` member (see [§4.8](#48-the-issuer-and-recipient-members)).

When `proof` is present, it covers the document with `proof` itself excluded from the signed content, per the canonicalization rules of the chosen Data Integrity suite.

#### 4.7.1 When to include a proof

The default rules governing the presence of `proof` in a *Trust Task document* are:

* If the document is delivered over a transport that already provides end-to-end integrity and authentication between *producer* and *consumer* — for example, mutually-authenticated TLS or a signed DIDComm envelope — `proof` **MAY** be omitted.
* If the document is delivered over a transport that does not provide such guarantees, or where tampering or substitution by intermediaries is possible, `proof` **SHOULD** be included.
* If a strong, transport-independent guarantee of non-tampering and of *producer* identity is required — typically because the document is intended to be retained, replayed, or relied on by parties beyond the original *consumer* — `proof` **MUST** be included.

An individual *Trust Task specification* **MAY** strengthen these defaults (for example, mandate `proof` regardless of transport) but **MUST NOT** weaken them. The declaration each *Trust Task specification* makes about its own `proof` requirement is governed by [§7.3](#73-specification-requirements).

### 4.8 The `issuer` and `recipient` members

A *Trust Task document* **MAY** identify the parties involved by including the `issuer` and `recipient` members at the top level of the document.

* `issuer` — a *Verifiable Identifier* (see [§2](#2-terminology)) identifying the *party* responsible for the document's content. When `proof` is present, the `issuer` **MUST** identify the entity to which the proof's `verificationMethod` resolves.
* `recipient` — a *Verifiable Identifier* identifying the *party* the *issuer* expects to act upon the document.

The framework does not constrain the VID scheme used: a DID, an X.509 subject, an OIDC subject identifier, a key thumbprint, or any other identifier whose controller is verifiable under the *consumer*'s trust framework is acceptable.

Both members are **OPTIONAL**. Their purpose is to let the parties be identified in-band where the transport in use does not already convey strong, authenticated party identity — for example, an unauthenticated HTTP POST, a public message queue, or paper hand-off.

Where a secure transport already conveys authenticated party identity (such as mutually-authenticated TLS or a signed DIDComm envelope), these in-band members **MAY** be omitted. Where both an in-band identity and a transport-derived identity are present for the same party, they **MUST** be consistent; a *consumer* **MUST** treat a mismatch as a validation failure.

An individual *Trust Task specification* **MAY** require either or both members to be present — for example, to support audit, third-party replay, or forwarding — but **MUST NOT** prohibit a *consumer* from comparing them with transport-derived identity.

### 4.9 The `threadId` member

Every *Trust Task document* carries its own unique `id` ([§4.3](#43-the-id-member)); a response document **MUST NOT** reuse the `id` of the document it is responding to. Correlating documents back to one another — for example, linking a response to its originating request — is the purpose of the `threadId` member, not the `id` member.

A *Trust Task document* **MAY** include a `threadId` member that correlates it with other *Trust Task documents* belonging to the same logical exchange — for example, a request and its response, or a request, an intermediate `trust-task-next-step` response, and the final result.

A *producer* that emits a *Trust Task document* in response to another *Trust Task document* **SHOULD** set `threadId` to the value of the originating document's `threadId`. If the originating document carried no `threadId`, the *producer* **SHOULD** set `threadId` to the value of the originating document's `id`. The effect of this convention is that every document in a logical exchange carries the same `threadId`, and that value can always be traced back to the `id` of the document that started the thread.

The framework places no constraint on the form of a `threadId` beyond requiring it to be a string. Producers initiating a new exchange **MAY** omit `threadId` entirely (single-shot tasks need no thread), **MAY** mint a fresh value (e.g. a UUID), or **MAY** reuse the document's own `id`.

`threadId` carries no normative validation semantics. *Consumers* **MUST NOT** reject a document on the basis of `threadId` alone, but **MAY** use it for routing, correlation, aggregation, or audit.

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

### 5.3 Maturity levels

A *Trust Task specification* progresses through three maturity levels. The maturity level is independent of the version number.

| Level | Meaning |
|---|---|
| `draft` | Working draft. The schema and prose **MAY** change without notice. |
| `candidate` | Schema is frozen except for editorial clarifications. The specification **MUST** demonstrate two independent, interoperable implementations to enter this level. |
| `standard` | Stable. A `candidate` specification **MUST** complete a continuous 90-day stability window with no breaking changes before promotion to `standard`. |

A specification's current maturity level is recorded in the registry at <https://trusttasks.org/>.

## 6. Namespace

The framework defines a single resolvable namespace per versioned *Trust Task specification*. One canonical URL serves human-readable prose, machine-readable schemas, and (where defined) JSON-LD contexts, differentiated by HTTP content negotiation.

### 6.1 Type URI

Every versioned *Trust Task specification* **MUST** be addressable by a *Type URI* of the form:

```
https://trusttasks.org/spec/<slug>/<MAJOR.MINOR>
```

where:

* `<slug>` is a lowercase, hyphen-separated short name assigned to the specification (e.g. `kyc-handoff`). The slug **MUST** match the regular expression `^[a-z][a-z0-9-]*[a-z0-9]$`. The slug `trust-task` is reserved for this framework specification itself.
* `<MAJOR.MINOR>` is the specification version as defined in [§5.1](#51-scheme).

The *Type URI* is the single canonical, resolvable reference to a versioned *Trust Task specification*. It serves both humans (rendered prose) and machines (validation schema, optional JSON-LD context) under content negotiation as defined in [§6.2](#62-content-negotiation).

A *Type URI* with the `<MAJOR.MINOR>` segment omitted (i.e. `https://trusttasks.org/spec/<slug>`) **SHOULD** redirect to the latest `standard` version of the specification, or — if no `standard` version exists — to the latest `candidate`, or — failing that — to the latest `draft`.

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

The outer document structure (`id`, `threadId`, `type`, `issuer`, `recipient`, `issuedAt`, `expiresAt`, `payload`, `@context`, `proof`) is described by the JSON Schema served at the framework's own *Type URI* — `https://trusttasks.org/spec/trust-task/<MAJOR.MINOR>` — under content negotiation for `application/schema+json`. A complete document validation therefore composes the framework schema (outer structure) with the task-specific payload schema.

The JSON Schema served at any *Type URI* **MUST** declare:

* `$id` equal to that *Type URI*.
* `$schema` set to `https://json-schema.org/draft/2020-12/schema`.

It **MUST** specify `additionalProperties` either explicitly as `false` or with an accompanying prose statement of how unrecognized members are to be treated.

### 6.4 Stability

For any value of `<slug>` and any value of `<MAJOR.MINOR>`, the representations served at the corresponding *Type URI* **MUST NOT** change in a way that alters their normative content once the specification has reached the `candidate` maturity level.

## 7. Minimum requirements

### 7.1 Producer requirements

A *conforming producer* **MUST**:

1. Emit a *Trust Task document* whose top-level structure satisfies [§4.2](#42-top-level-members).
2. Set the `type` member to the *Type URI* of the *Trust Task specification* being implemented, including its `<MAJOR.MINOR>` segment.
3. Place all task-specific data in `payload`, and emit a `payload` value that validates against the JSON Schema obtained by content-negotiating the *Type URI* for `application/schema+json` (see [§6.2](#62-content-negotiation)).
4. Populate `id` with a value satisfying [§4.3](#43-the-id-member).

A *conforming producer* **SHOULD** populate `issuedAt` to support freshness checks downstream, **SHOULD** populate `issuer` and `recipient` when the transport in use does not provide authenticated party identity end-to-end between *producer* and *consumer*, **SHOULD** set `threadId` when emitting a *Trust Task document* in response to another (see [§4.9](#49-the-threadid-member)), and **SHOULD** preserve any unrecognized members received from upstream parties when forwarding a *Trust Task document*.

### 7.2 Consumer requirements

A *conforming consumer* **MUST**:

1. Validate the outer document structure against the framework schema obtained by content-negotiating the framework *Type URI* — `https://trusttasks.org/spec/trust-task/<MAJOR.MINOR>` — for `application/schema+json`, at the `MAJOR.MINOR` parsed from the trailing segment of the document's `type` member.
2. Validate the document's `payload` member against the JSON Schema obtained by content-negotiating the document's `type` member for `application/schema+json`.
3. Reject any document whose `type` it does not recognize, unless the consumer's policy explicitly permits forward-compatible processing under [§5.2](#52-compatibility-rules).
4. Honor the document's expiry where present: if `expiresAt` is set and its value lies in the past relative to the *consumer*'s clock, treat the document as expired and not act upon it.
5. Reject any document whose `recipient` member is set and does not identify the *consumer*'s own party.
6. Reject any document for which an in-band `issuer` or `recipient` member is inconsistent with an authenticated identity derived from the transport for the same party.

A *conforming consumer* **SHOULD** preserve, but **MUST NOT** act upon, members it does not recognize. A *consumer* that does not implement JSON-LD processing **MUST** ignore the `@context` member.

When a *consumer* rejects a *Trust Task document* under any rule in this section, and the transport in use supports a response from *consumer* to *producer*, the *consumer* **SHOULD** return an *error response* conforming to [§8](#8-error-responses).

### 7.3 Specification requirements

A *conforming Trust Task specification* **MUST** publish a normative JSON Schema for its `payload` member that:

1. Is a valid JSON Schema document under [[JSON-SCHEMA-2020-12]].
2. Sets `$id` to the specification's *Type URI*.
3. Sets `$schema` to `https://json-schema.org/draft/2020-12/schema`.
4. Specifies `additionalProperties` either explicitly as `false` or with an accompanying prose statement of how unrecognized payload members are to be treated.
5. Is served at its *Type URI* under content negotiation for `application/schema+json`.

A *conforming Trust Task specification* **MUST** also state explicitly whether the `proof` member is **OPTIONAL**, **RECOMMENDED**, or **REQUIRED** for documents implementing it, together with a brief rationale referencing the threat model addressed — for example, tampering by intermediaries, replay, repudiation by the *producer*, or reliance by third parties beyond the original *consumer*. The declared requirement **MUST NOT** be weaker than the default applicable under [§4.7.1](#471-when-to-include-a-proof).

## 8. Error responses

A *recipient party* that cannot or will not act upon a received *Trust Task document* **MAY** return an **error response** describing why. Error responses are themselves *Trust Task documents* of a framework-defined type, so that one validation, signing, and transport pipeline serves both successful tasks and their refusals.

### 8.1 The trust-task-error specification

The framework reserves the slug `trust-task-error` for the error-response *Trust Task specification* at:

```
https://trusttasks.org/spec/trust-task-error/<MAJOR.MINOR>
```

An *error response* is a *Trust Task document* whose `type` is the URI above. Its `payload` carries the standard error structure defined in [§8.2](#82-error-payload). The `id` member of an *error response* identifies the error instance and **MUST NOT** be reused; correlation back to the original task being responded to is carried by the framework's `threadId` member ([§4.9](#49-the-threadid-member)).

### 8.2 Error payload

The `payload` of an *error response* has the following members. The correlation back to the *Trust Task document* this error reports on is carried at the framework level by the `threadId` member ([§4.9](#49-the-threadid-member)), which a *producer* of an error response **MUST** set.

| Member | Required | Type | Description |
|---|---|---|---|
| `code` | **MUST** | string | A short identifier for the failure category. **MUST** be one of the codes in [§8.3](#83-standard-error-codes) or an extended code as defined in [§8.5](#85-extension-by-individual-trust-task-specifications). |
| `message` | **SHOULD** | string | A human-readable description of the error. Non-normative; intended for logs and operator UI. |
| `retryable` | **MUST** | boolean | `true` if the *producer* of the original document **MAY** retry the task; `false` if retrying with the same document or credentials is not expected to succeed. |
| `retryAfter` | **MAY** | string (date-time) | An [[RFC3339]] timestamp before which the *producer* **SHOULD NOT** retry. Meaningful only when `retryable` is `true`. |
| `details` | **MAY** | object | Task-specific extension data; see [§8.5](#85-extension-by-individual-trust-task-specifications). |

> **Example 2 — An error response** *(non-normative)*
>
> ```json
> {
>   "id": "9e2a1c44-7b81-4d3e-9b51-7a3c89e3d1f2",
>   "type": "https://trusttasks.org/spec/trust-task-error/1.0",
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
| `malformed_request` | The document did not validate against the framework schema or the task-specific payload schema. | `false` |
| `unsupported_type` | The *consumer* does not recognize the `type` URI. | `false` |
| `unsupported_version` | The `type` URI was recognized but its `MAJOR.MINOR` version is not supported. | `false` |
| `expired` | The document's `expiresAt` was in the past at the time of evaluation. | `false` |
| `proof_required` | A `proof` was required (by the *Trust Task specification* or *consumer* policy) and was missing. | `false` |
| `proof_invalid` | A `proof` was present but failed verification. | `false` |
| `permission_denied` | The requesting *party* is not authorized to invoke this task. | `false` |
| `wrong_recipient` | The document's `recipient` does not identify the receiving *consumer*. | `false` |
| `identity_mismatch` | An in-band `issuer` or `recipient` value is inconsistent with the corresponding transport-authenticated identity. | `false` |
| `task_failed` | The *recipient party* attempted the task and could not complete it; further detail **SHOULD** appear in `details`. | varies |
| `unavailable` | The *recipient party* is temporarily unable to process the task. | `true` |
| `internal_error` | The *recipient party* encountered an unexpected internal failure. | `true` |

The "Default `retryable`" column gives the value an emitter of an error response **SHOULD** use unless task-specific knowledge dictates otherwise. The actual `retryable` value carried in a given *error response* is authoritative.

### 8.4 Retry semantics

A *party* that receives an *error response* **MUST NOT** retry the original *Trust Task document* if `retryable` is `false`. When `retryable` is `true`, the party **SHOULD** wait at least until any `retryAfter` value before retrying, and **SHOULD** apply backoff appropriate to the transport in use.

A `false` value of `retryable` represents a hard failure for that specific document. It does not prohibit the *producer* from issuing a *new* *Trust Task document* (with a fresh `id`) addressing the cause of the failure — for example, re-issuing with a valid `proof` after a `proof_invalid` error.

### 8.5 Extension by individual Trust Task specifications

An individual *Trust Task specification* **MAY** define additional error codes specific to its task. Extended codes **MUST** be namespaced with the specification's `<slug>` separated from the local code by a colon, e.g. `kyc-handoff:document_revoked`. Extended codes **MUST NOT** shadow any code listed in [§8.3](#83-standard-error-codes).

An individual *Trust Task specification* **MAY** also define the structure of `details` for its own error responses. Where it does so, the specification **MUST** state which `code` values may carry a `details` object and **MUST** provide a JSON Schema fragment describing the `details` shape for each.

A *consumer* that does not recognize an extended `code` **SHOULD** treat the error as if its code were `task_failed` and **MUST** still honor the `retryable` and `retryAfter` members.

### 8.6 Reserved response-type slugs

The framework reserves the following additional response-type *Trust Task specification* slugs. These slugs **MUST NOT** be used for any individual *Trust Task specification* registered under [§6.1](#61-type-uri).

| Slug | Purpose |
|---|---|
| `trust-task-ok` | Success with metadata — acknowledging that a task was performed and conveying any resulting references, receipts, or transient state. |
| `trust-task-next-step` | A recipient-suggested continuation — indicating that the original task was understood but cannot complete in isolation, together with the next *Trust Task* the *recipient party* expects in order to proceed. |

The payload structures of these specifications are out of scope for this revision and will be specified in a future revision of this framework. Implementations encountering a *Trust Task document* of either reserved type before the corresponding specification is published **MAY** ignore the document or **MAY** return an `unsupported_version` *error response*.

## 9. Transport bindings

The framework deliberately leaves transport unconstrained ([§1.1](#11-design-goals) Goal 1): a *Trust Task document* can be conveyed over any channel that preserves its content. To make that composability work in practice, each transport protocol used to carry *Trust Task documents* **SHOULD** be accompanied by a *transport binding* specification.

A *transport binding* defines how *Trust Task documents* are exchanged over a specific transport — for example, DIDComm, the IETF Trust Spanning Protocol (TSP), HTTPS with mutual-TLS, AMQP, or paper. It is the integration layer between the framework's transport-agnostic semantics and the realities of a particular transport.

### 9.1 What a transport binding specifies

A *transport binding* **SHOULD** specify each of the following:

* **Document carriage.** How a *Trust Task document* is placed onto and retrieved from the transport (request body, message payload, envelope field, attachment, etc.).
* **Field population from transport context.** Which framework members the binding populates from transport-derived information — typically `issuer` (from a transport-authenticated sender), `recipient` (from a transport-authenticated addressee), and any signature metadata that lets a *consumer* verify the framework `proof` against transport-bound keys or, per [§4.7.1](#471-when-to-include-a-proof), accept the document without an in-band `proof`.
* **Consistency enforcement.** The behavior when an in-band framework member and its transport-derived equivalent disagree. The framework requires they **MUST** be consistent (see [§4.8](#48-the-issuer-and-recipient-members) and [§7.2](#72-consumer-requirements)); the binding states how the comparison is performed for the transport in question (for example, how a DID carried in-band is matched against a transport-authenticated DID).
* **Transport security profile.** The integrity, authentication, confidentiality, and freshness guarantees the transport provides, so that *consumers* can correctly evaluate the `proof` requirement under [§4.7.1](#471-when-to-include-a-proof).
* **Error and response delivery.** How an *error response* ([§8](#8-error-responses)) is returned to the *producer* of the original document, including the behavior when the transport is fire-and-forget.

### 9.2 The transport handler

An implementation that exchanges *Trust Task documents* over a given transport **SHOULD** expose its transport-binding logic as a discrete *transport handler* component:

1. On the **producer** side, the handler composes an outbound *Trust Task document*, populates framework members the binding can derive from transport context, and applies the transport's signing or sealing where the binding integrates it with `proof`.
2. On the **consumer** side, the handler extracts an inbound *Trust Task document* from the transport, populates or asserts framework members from transport-derived identity, integrity, and freshness metadata, and surfaces inconsistencies as validation failures per [§7.2](#72-consumer-requirements).

The handler boundary lets the framework's validation logic remain transport-agnostic while different transports plug in their own population rules. A DIDComm handler can populate `issuer` from the verified sender DID of the surrounding DIDComm envelope; a TSP handler can do the same from the TSP message authentication; a mutual-TLS HTTPS handler can populate `issuer` from the peer certificate's subject; an unauthenticated transport handler populates nothing, and the framework falls back to the in-band `proof` per [§4.7.1](#471-when-to-include-a-proof).

A *transport binding* specification **SHOULD** identify itself by a stable URI and **SHOULD** declare which version of this framework it targets. The framework does not maintain a closed registry of *transport bindings*; new bindings **MAY** be published independently.

## 10. Security and Privacy Considerations

*This section is non-normative in the current draft. Future revisions are expected to make portions of it normative as individual Trust Task specifications surface concrete requirements.*

A *Trust Task document* carries no inherent transport security. The framework's default rules for when an integrity proof is required of a document are given in [§4.7.1](#471-when-to-include-a-proof), and each *Trust Task specification* declares its own requirement under [§7.3](#73-specification-requirements). When `proof` is included, it **MUST** conform to the W3C *Data Integrity* format defined in [[VC-DATA-INTEGRITY]] (see [§4.7](#47-proof)); implementations select an appropriate cryptographic suite from the W3C-registered set based on the trust requirements agreed by the parties.

Personal data carried in a *Trust Task document* is visible to every *party* that handles the document. Individual *Trust Task specifications* **SHOULD** minimize personal data in their schemas to that strictly necessary to achieve the task's outcome, and **SHOULD** prefer references (e.g. DID URLs) to direct attribute values where the relying party is able to dereference them.

Because *Trust Task documents* are self-contained, a captured document remains evidence of its content after it has been delivered. Producers **SHOULD** consider whether the document's contents are appropriate for indefinite retention by the consumer.

## 11. References

### 11.1 Normative references

* **[RFC2119]** Bradner, S. *Key words for use in RFCs to Indicate Requirement Levels*. RFC 2119, March 1997. <https://www.rfc-editor.org/rfc/rfc2119>
* **[RFC8174]** Leiba, B. *Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words*. RFC 8174, May 2017. <https://www.rfc-editor.org/rfc/rfc8174>
* **[RFC3339]** Klyne, G., Newman, C. *Date and Time on the Internet: Timestamps*. RFC 3339, July 2002. <https://www.rfc-editor.org/rfc/rfc3339>
* **[RFC3986]** Berners-Lee, T., Fielding, R., Masinter, L. *Uniform Resource Identifier (URI): Generic Syntax*. RFC 3986, January 2005. <https://www.rfc-editor.org/rfc/rfc3986>
* **[RFC8259]** Bray, T. (Ed.). *The JavaScript Object Notation (JSON) Data Interchange Format*. RFC 8259, December 2017. <https://www.rfc-editor.org/rfc/rfc8259>
* **[RFC9110]** Fielding, R. (Ed.), Nottingham, M. (Ed.), Reschke, J. (Ed.). *HTTP Semantics*. RFC 9110, June 2022. <https://www.rfc-editor.org/rfc/rfc9110>
* **[RFC9562]** Davis, K. (Ed.), Peabody, B., Leach, P. *Universally Unique IDentifiers (UUIDs)*. RFC 9562, May 2024. <https://www.rfc-editor.org/rfc/rfc9562>
* **[VC-DATA-INTEGRITY]** Longley, D., Sporny, M. *Verifiable Credential Data Integrity 1.0*. W3C Recommendation. <https://www.w3.org/TR/vc-data-integrity/>
* **[ABNF]** Crocker, D., Overell, P. *Augmented BNF for Syntax Specifications: ABNF*. RFC 5234, January 2008. <https://www.rfc-editor.org/rfc/rfc5234>
* **[JSON-SCHEMA-2020-12]** Wright, A. et al. *JSON Schema: A Media Type for Describing JSON Documents*. Draft 2020-12. <https://json-schema.org/draft/2020-12/schema>
* **[DID-CORE]** Sporny, M., Longley, D., Sabadello, M., Reed, D., Steele, O., Allen, C. *Decentralized Identifiers (DIDs) v1.0*. W3C Recommendation. <https://www.w3.org/TR/did-core/>

### 11.2 Informative references

* **[VC-DATA-MODEL]** Sporny, M. et al. *Verifiable Credentials Data Model v2.0*. W3C Recommendation. <https://www.w3.org/TR/vc-data-model-2.0/>
* **[W3C-MANUAL-OF-STYLE]** W3C. *Manual of Style*. <https://www.w3.org/guide/manual-of-style/>

## 12. Acknowledgments

The editor thanks the members of the Trust over IP Foundation Decentralized Trust Graph Working Group for their ongoing review and contributions to this specification.
