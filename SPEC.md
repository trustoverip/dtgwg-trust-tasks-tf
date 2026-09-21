# Trust Tasks

> **Generated file — do not edit.** This is the Trust Tasks framework specification, generated
> from [`f2a4b67a6298`](https://github.com/trustoverip/dtgwg-trust-tasks-spec/tree/f2a4b67a6298b6b9054a459b5de0c32a0e99f32a) (`main`) of the canonical source,
> <https://github.com/trustoverip/dtgwg-trust-tasks-spec>, by `scripts/generate-framework-spec.mjs`. Changes to the text belong
> there. Section numbers are assigned here, in the canonical order, because the registry cites
> the framework by number; they are not part of the canonical text.

_Version:_ 1.0  
_Document Status:_ Working Draft 0.6.0  
_Last Updated:_ 2026-09-21  
_GitHub:_ <https://github.com/trustoverip/dtgwg-trust-tasks-spec>
<!-- _DOI:_ To be assigned when this specification reaches ToIP Approved Deliverable status. See https://lf-toip.atlassian.net/wiki/spaces/HOME/pages/767787009/ToIP+Approved+Deliverable+Process#Persistent-DOI-Link -->

_Editors:_

- Glenn Gore, Affinidi

_Contributors:_

- The participants of the Trust Tasks Task Force of the Decentralized Trust Graph Working Group (DTGWG)

**Abstract**

This document defines the **Trust Tasks** framework: a specification for the verifiable work that occurs between two or more parties. A Trust Task is a self-contained, transport-agnostic, JSON-based description of an outcome that two parties agree to achieve. This framework specification defines the document structure, version scheme, namespace, and conformance requirements that every individual Trust Task specification — published under the registry at `https://trusttasks.org/` — is expected to satisfy. Individual Trust Task specifications (for example, the `acl/change-role` specification at `https://trusttasks.org/spec/acl/change-role/0.1`) are conforming refinements of this framework.

**Intellectual Property Rights**

This specification is provided under the [Joint Development Foundation (JDF) charter](https://cdn.platform.linuxfoundation.org/agreements/ToIP.pdf) for [Trust Over IP](https://trustoverip.org) (ToIP) and is subject to the intellectual property rights policy of the **Decentralized Trust Graph Working Group (DTGWG)**:

_Copyright:_ [Creative Commons Attribution 4.0 International (CC BY 4.0)](https://creativecommons.org/licenses/by/4.0/)  
_Patent:_ W3C Mode (based on the [W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20040205/))  
_Source Code:_ [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0)

THESE MATERIALS ARE PROVIDED “AS IS.” The parties expressly disclaim any warranties (express, implied, or otherwise), including implied warranties of merchantability, non-infringement, fitness for a particular purpose, or title, related to the materials. The entire risk as to implementing or otherwise using the materials is assumed by the implementer and user. IN NO EVENT WILL THE PARTIES BE LIABLE TO ANY OTHER PARTY FOR LOST PROFITS OR ANY FORM OF INDIRECT, SPECIAL, INCIDENTAL, OR CONSEQUENTIAL DAMAGES OF ANY CHARACTER FROM ANY CAUSES OF ACTION OF ANY KIND WITH RESPECT TO THIS DELIVERABLE OR ITS GOVERNING AGREEMENT, WHETHER BASED ON BREACH OF CONTRACT, TORT (INCLUDING NEGLIGENCE), OR OTHERWISE, AND WHETHER OR NOT THE OTHER MEMBER HAS BEEN ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

<a id="abstract"></a><a id="status-of-this-document"></a>
## 1. Introduction

*This section is informative.*

Two parties interoperate when they agree on the shape of the work they cooperate on. Today that agreement is reached ad-hoc: every onboarding flow, every consent receipt, every credential exchange is described in a vendor-specific schema, carried over a vendor-specific protocol, and validated by vendor-specific code. The result is a combinatorial explosion of pairwise integrations.

A **Trust Task** is a single, finite description of an outcome between two parties — a KYC handoff, a consent grant, a payment commitment — that is portable across implementations because the task definition is decoupled from the transport that delivers it.

Three properties make a Trust Task portable:

1. **Self-contained** — the document carries everything needed to act on it: parties, criteria, schema, identifiers. No hidden context.
2. **Transport-agnostic** — the document makes no assumption about the protocol that delivers it. DIDComm, HTTPS, message queue, paper — the task is the task.
3. **JSON-based** — the canonical serialization is a single JSON object validated against a published JSON Schema.

The body of this framework specification defines the document structure, version scheme, and namespace shared by every individual Trust Task specification published under the registry.

This is a Working Draft prepared by the Trust Tasks Task Force of the Decentralized Trust Graph Working Group (DTGWG) of the [Trust Over IP Foundation](https://trustoverip.org). Publication as a Working Draft does not imply endorsement by the ToIP membership. Comments are welcome via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-spec/issues). The editors expect the substantive sections — in particular [Minimum Requirements](#7-minimum-requirements), [Error Responses](#8-error-responses), [Transport Bindings](#9-transport-bindings), and [Security Considerations](#12-security-considerations) — to evolve as individual Trust Task specifications progress through the [Maturity Levels](#53-maturity-levels) and surface gaps in this framework.

The individual Trust Task specifications that conform to this framework, together with the transport bindings and the generated client libraries, are maintained in the [Trust Tasks registry repository](https://github.com/trustoverip/dtgwg-trust-tasks-tf) and published at <https://trusttasks.org/>.

### 1.1 Design Goals

*This section is informative.*

The framework aims to solve four related problems that arise wherever two or more parties cooperate over a network.

1. **A common task vocabulary across any transport.** In a decentralized ecosystem there is no single message bus or RPC framework: parties speak DIDComm, HTTPS, message queues, paper, and anything else. *Trust Tasks* let two parties agree on *what* they are doing without first agreeing on *how* the bits move between them. The same task specification works regardless of the transport carrying it.

2. **Security, privacy, and identity that scale to the transport.** A *Trust Task document* can rely on the integrity, authentication, and party-identity guarantees already provided by the transport in use — for example, mutually-authenticated TLS or a signed DIDComm envelope — and where those guarantees are absent, the document's own `proof`, `issuer`, and `recipient` members ([Proof](#47-proof), [The `issuer` and `recipient` Members](#48-the-issuer-and-recipient-members)) supply them in-band. Implementers can match cryptographic work to the threat model in front of them rather than always paying the worst-case cost.

3. **Payload freedom, declared at the boundaries.** The framework defines the outer document shape and deliberately leaves the `payload` unconstrained. Each *Trust Task specification* chooses its own payload structure, JSON Schema, and — where useful — JSON-LD context. The framework only requires that each choice be declared explicitly ([Specification Requirements](#73-specification-requirements)) and be machine-validatable.

4. **A standard family of response types.** Many tasks need a structured way for a *recipient party* to report what happened. The framework reserves a small set of response-type *Trust Task specifications* addressing the common cases — failure ([Error Responses](#8-error-responses)), success (the `#response` variant of the task's own *Type URI*, which since framework version 0.5.0 also carries the courtesy acknowledgement of a fire-and-forget task and supersedes `trust-task-ok`), and a recipient-suggested continuation (`trust-task-next-step`) — each itself a *Trust Task* so that one validation, signing, and transport pipeline serves both the task and its response. All three are specified as of this revision (see [Error Responses](#8-error-responses) and [Reserved Response-Type Slugs](#86-reserved-response-type-slugs)).

## 2. Requirements Language

*This section is normative.*

The key words “MUST”, “MUST NOT”, “REQUIRED”, “SHALL”, “SHALL NOT”, “SHOULD”, “SHOULD NOT”, “RECOMMENDED”, “MAY”, and “OPTIONAL” in this document are to be interpreted as described in BCP 14 [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119) [RFC 8174](https://www.rfc-editor.org/rfc/rfc8174) when, and only when, they appear in all capitals, as shown here.

<a id="2-terminology"></a>
## 3. Terminology

*This section is normative.*

The key terms used throughout this specification are defined below. Where one of these terms is *italicized* on subsequent use, the definition given here applies.

Any hyperlinked term not included in this section is referenced from one of the following glossaries:

- [ToIP Main Glossary](https://glossary.trustoverip.org)
- [ToIP General IT Glossary](https://trustoverip.github.io/ctwg-general-glossary)

* *Ceremony definition* — The published, versioned description of a *Trust Ceremony*, identified by a URI in the `/ceremony/` subtree ([Ceremony Namespace](#67-ceremony-namespace)). It is **not** a *Trust Task specification* and no document's `type` resolves to one. Its content is out of scope for this version.
* *Consequential Trust Task* — A *Trust Task* whose successful execution has an effect the *consumer* cannot silently undo: it alters *recipient party* state, releases confidential material, or exercises the subject's own authority. Precisely, a *Trust Task* is *consequential* where any of the following holds — `sideEffects.level` is `mutating` or `destructive`, `exposure.discloses` is `secret`, or `exposure.actsAsSubject` is `true` (see [Specification Requirements](#73-specification-requirements) items 13 and 14). Where a *consumer* determines the authoritative values from the handler it is about to invoke rather than from the values the *Trust Task specification* declares, the authoritative values govern. An absent, unrecognized, or unresolvable declaration is *consequential*, consistent with the fail-safe defaults of [Specification Requirements](#73-specification-requirements) items 13 and 14. The term names a class of task for which several rules in this specification are stricter than the general case; it carries no authorization meaning of its own (see [Consumer Requirements](#72-consumer-requirements) item 10).
* *Consumer* — A *party* that receives and processes a *Trust Task document*. Synonym: *recipient party* when emphasizing the consumer's acceptance role (for example, in error-response prose). The two terms refer to the same entity and are used interchangeably throughout this specification.
* *Data Integrity Proof* — An optional integrity-providing object attached to a *Trust Task document*, in the form of a W3C *Data Integrity Proof* (see [Proof](#47-proof)).
* *Document identifier* — The string carried in the `id` member of a *Trust Task document* that uniquely identifies that instance.
* *Enactment* — One run of a *Trust Ceremony*, identified by a globally unique, non-reusable string carried in `ceremony.enactment`. An *enactment* is to a *ceremony definition* what a *Trust Task document* is to a *Trust Task specification*.
* *Error response* — A *Trust Task document* whose `type` resolves to the framework's reserved `trust-task-error` specification and that reports a failure with respect to a previously received *Trust Task document*. See [Error Responses](#8-error-responses).
* *Outcome evidence* — The initiating document of a cited exchange together with the terminal success document showing that the exchange completed, paired under [Evidence That a Cited Exchange Completed](#494-evidence-that-a-cited-exchange-completed). A citation names and binds an exchange; outcome evidence is what shows the cited task reached `responded`.
* *Parent thread identifier* — An optional string carried in the `parentThreadId` member that names the exchange containing this one, where a *Trust Task* is conducted inside a broader exchange. See [The `parentThreadId` Member](#492-the-parentthreadid-member).
* *Party* — An entity that participates in a *Trust Task*. Each party is identified by a *Verifiable Identifier*.
* *Payload* — The task-specific portion of a *Trust Task document*, carried in the `payload` member. Its internal structure is defined by the *Trust Task specification* identified by the document's `type`.
* *Producer* — A *party* that emits a *Trust Task document*. Synonym: *issuer* when referring to the value carried in the document's `issuer` member.
* *Thread identifier* — An optional string carried in the `threadId` member that correlates a *Trust Task document* with other documents belonging to the same logical exchange. See [The `threadId` Member](#49-the-threadid-member).
* *Transport binding* — A specification that defines how *Trust Task documents* are exchanged over a specific transport protocol, including how transport-derived identity, integrity, and freshness are mapped into framework members. See [Transport Bindings](#9-transport-bindings).
* *Trust Ceremony* — A flow composed of several *Trust Tasks*, optionally described by a published *ceremony definition*. Named here because a *Trust Task document* may record its membership of one; see [The `ceremony` Member](#411-the-ceremony-member).
* *Trust Task* — A unit of verifiable work between two parties, formally defined by a *Trust Task specification* and exchanged as *Trust Task documents*. Each instance of work — a KYC handoff, a consent grant, a payment commitment — is a *Trust Task*. The document model defined by this framework is bilateral: each *Trust Task document* names at most one *issuer* and one *recipient*. Exchanges involving more than two parties are modeled as multiple bilateral *Trust Tasks* linked by the framework's `threadId` member (see [The `threadId` Member](#49-the-threadid-member)).
* *Trust Task document* — A single JSON object that is an instance of a *Trust Task*. It carries its own type, identifier, and integrity metadata alongside a task-specific *payload*. The structure is defined normatively in [Trust Task Documents](#4-trust-task-documents).
* *Trust Task specification* — A document, conforming to this framework, that defines a single *Trust Task* — its slug, version, target framework version, parties, normative payload schema, proof requirement, and any task-specific error extensions. See [Specification Requirements](#73-specification-requirements) for the full set of declarations a specification **MUST** publish.
* *Type URI* — A URI that identifies a *Trust Task specification* at a specific version and serves as the single resolvable namespace for that version. The canonical form is defined in [Type URI](#61-type-uri).
* *VID* — A string identifier whose controller is verifiable under a trust framework. Decentralized Identifiers (DIDs) [DID Core](https://www.w3.org/TR/did-core/) are one realization of VIDs; others include X.509 subjects, OIDC subject identifiers, and key thumbprints. The framework does not constrain the VID scheme; the *consumer*'s trust framework determines which schemes are accepted and how each is resolved to verification material.

## 4. Trust Task Documents

*This section is normative.*

A *Trust Task document* is a single JSON object. The framework deliberately does **not** define a separate envelope: type, version, identifier, and integrity metadata are members of the document itself. This simplifies validation — one object, one schema composition — and removes the ambiguity of "is this field on the wrapper or the body?"

### 4.1 Encoding

A *Trust Task document* **MUST** be a JSON object as defined in [RFC 8259](https://www.rfc-editor.org/rfc/rfc8259). The document **MUST** be encoded as UTF-8 without a byte-order mark.

### 4.2 Top-Level Members

A *Trust Task document* has the following top-level members.

| Member | Required | Type | Description |
|---|---|---|---|
| `id` | **MUST** | string | The *Document identifier* — a globally unique string for this instance of the task. UUIDv4 is **RECOMMENDED**; any uniquely-assignable string is permitted. See [The `id` Member](#43-the-id-member). |
| `threadId` | **MAY** | string | The *Thread identifier* — correlates this document with others in the same logical exchange (e.g. a response back to its originating request). See [The `threadId` Member](#49-the-threadid-member). |
| `parentThreadId` | **MAY** | string | The *Parent thread identifier* — the `threadId` of the exchange that contains this one, where this exchange is conducted inside another. See [The `parentThreadId` Member](#492-the-parentthreadid-member). |
| `ceremony` | **MAY** | object | Records that this document is a step of a *Trust Ceremony* — a flow composed of several *Trust Tasks*. See [The `ceremony` Member](#411-the-ceremony-member). |
| `type` | **MUST** | string (URI) | The *Type URI* identifying the *Trust Task specification* and version this document conforms to. See [The `type` Member](#44-the-type-member). |
| `issuer` | **MAY** | string (VID) | A *Verifiable Identifier* identifying the *party* responsible for the document's content. See [The `issuer` and `recipient` Members](#48-the-issuer-and-recipient-members). |
| `recipient` | **MAY** | string (VID) | A *Verifiable Identifier* identifying the *party* the *issuer* expects to act upon the document. See [The `issuer` and `recipient` Members](#48-the-issuer-and-recipient-members). |
| `issuedAt` | **SHOULD** | string (date-time) | An [RFC 3339](https://www.rfc-editor.org/rfc/rfc3339) timestamp recording when the document was produced. It is the value a *consumer* places the document in its acceptance window by; see [Consumer Requirements](#72-consumer-requirements) item 13 for the freshness bounds applied to it, and [Specification Requirements](#73-specification-requirements) item 17 for when a *Trust Task specification* **MUST** require it. |
| `expiresAt` | **MAY** | string (date-time) | An [RFC 3339](https://www.rfc-editor.org/rfc/rfc3339) timestamp after which the document is no longer valid **to accept**. Where `expiresAt` is specified, the *recipient party* **MUST** honor the expiry: a *consumer* **MUST NOT** act upon a document for which `now ≥ expiresAt` (inclusive bound; the instant `expiresAt` is itself treated as expired). A *consumer* **MAY** apply a small clock-skew tolerance, typically ≤ 60 seconds, when evaluating this comparison. `expiresAt` bounds **acceptance**, not execution: it does not abort work already under way (see [Consumer Requirements](#72-consumer-requirements) item 12). See [Consumer Requirements](#72-consumer-requirements). |
| `payload` | **MUST** | object | The task-specific body. Its internal structure is governed by the *Trust Task specification* identified by `type`. See [The `payload` Member](#45-the-payload-member). |
| `@context` | **MAY** | string \| array \| object | If present, enables JSON-LD processing of the document. See [JSON-LD Compatibility](#46-json-ld-compatibility). |
| `proof` | **MAY** | object | An optional integrity proof. See [Proof](#47-proof). |

A *Trust Task document* **MAY** contain additional top-level members beyond those listed above. Member names beginning with `x-` are **RESERVED** for experimental extensions and **MUST NOT** be used in a published *Trust Task specification*.

> **Example 1 — A complete Trust Task document** *(non-normative)*
>
> ```json
> {
>   "id": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
>   "type": "https://trusttasks.org/spec/acl/change-role/0.1",
>   "issuer": "did:web:org.example",
>   "recipient": "did:web:maintainer.example",
>   "issuedAt": "2026-06-10T14:00:00Z",
>   "expiresAt": "2026-06-11T14:00:00Z",
>   "payload": {
>     "subject": "did:web:bob.example",
>     "fromRole": "member",
>     "toRole": "moderator"
>   },
>   "proof": {
>     "type": "DataIntegrityProof",
>     "cryptosuite": "eddsa-rdfc-2022",
>     "verificationMethod": "did:web:org.example#key-1",
>     "created": "2026-06-10T14:00:00Z",
>     "proofPurpose": "assertionMethod",
>     "proofValue": "z5xy..."
>   }
> }
> ```
>
> The `payload` member is the only part whose internal shape is defined by the per-task specification; everything else is framework-defined. This example carries a `proof` member because the *Trust Task specification* it names declares `proof` **REQUIRED** — a role change is retained as the evidentiary record of how privilege was acquired, and is relied upon after delivery. The framework itself leaves `proof` **OPTIONAL**: a document of a specification that does not require one is conforming without it, provided it is delivered over a transport that supplies end-to-end integrity and authentication between *producer* and *consumer* (see [When to Include a Proof](#471-when-to-include-a-proof)).

### 4.3 The `id` Member

The `id` member's value **MUST** be a string that is globally unique to this instance of the task. The framework places no further constraint on its form: UUIDv4 [RFC 9562](https://www.rfc-editor.org/rfc/rfc9562) is **RECOMMENDED** as a low-coordination default that requires no namespace ownership, but any string an implementation can guarantee unique is permitted — for example, a DID URL, a UUIDv7, a URN, or an opaque content-addressed identifier. Producers **MUST NOT** reuse an `id` value across documents.

The `id` is opaque to the framework. Resolvability of the `id` (the ability to dereference it back to the document) is not required. Where resolvability is needed for a particular use case, an individual *Trust Task specification* or transport binding **MAY** require a more specific form (for example, a DID URL).

### 4.4 The `type` Member

The `type` member's value **MUST** be a *Type URI* in the form defined in [Type URI](#61-type-uri). The version of the *Trust Task specification* a document conforms to is conveyed by the trailing `<MAJOR.MINOR>` segment of this URI; no separate version member is carried in the document.

A `type` URI **MAY** carry a fragment identifier. The framework reserves the fragments `#request` and `#response` to disambiguate the two directions of a request/response exchange that share a single *Trust Task specification*; see [Request and Response Variants](#441-request-and-response-variants).

#### 4.4.1 Request and Response Variants

A single *Trust Task specification* often describes both a *request* document (the document a *producer* sends to initiate the task) and a *response* document (the document the *consumer* returns when the task completes successfully). The framework distinguishes the two via the fragment of the `type` URI:

| `type` form | Meaning |
|---|---|
| `https://trusttasks.org/spec/<slug>/<MAJOR.MINOR>` | Request document. Implicitly equivalent to the explicit form below. |
| `https://trusttasks.org/spec/<slug>/<MAJOR.MINOR>#request` | Request document, explicit form. |
| `https://trusttasks.org/spec/<slug>/<MAJOR.MINOR>#response` | Success-response document for the same specification. |

A reply to a *Trust Task document* takes one of exactly three dispositions, distinguished by the reply's own `type` and by nothing else:

| Disposition | `type` of the reply | Closes the exchange? | Defined in |
|---|---|---|---|
| **Success** | the originating *Type URI* with the fragment `#response` | Yes | This section |
| **Failure** | `https://trusttasks.org/spec/trust-task-error/<MAJOR.MINOR>` | Yes | [Error Responses](#8-error-responses) |
| **Continuation** | `https://trusttasks.org/spec/trust-task-next-step/<MAJOR.MINOR>` | **No** — the originating document remains `accepted` and blocked ([Document Lifecycle](#412-document-lifecycle)) | [Reserved Response-Type Slugs](#86-reserved-response-type-slugs) |

The three are exhaustive and are not interchangeable: a *consumer* **MUST NOT** report a failure as a `#response`, a success as a `trust-task-error`, or a blocked task as either. A party receiving a reply **MUST** determine its disposition from the reply's `type` alone — the fragment for the first, the slug for the other two — and **MUST NOT** infer it from the presence or absence of any `payload` member.

The table is stated here because "is this document a response?" is otherwise answerable in three places — the fragment rule below, the distinct-*Type URI* rule of [Error Responses](#8-error-responses), and the third-disposition rule of [Reserved Response-Type Slugs](#86-reserved-response-type-slugs) — and an implementation that finds one of them hard-codes a list of the slugs it happens to know, which the next reserved slug silently invalidates.

The rules:

1. A *Trust Task document* whose `type` URI carries **no fragment** or the fragment `#request` is a *request*. The two forms are semantically equivalent; producers **MAY** emit either, consumers **MUST** accept both.
2. A *Trust Task document* whose `type` URI carries the fragment `#response` is the *success response* of a request whose `type` is the same URI with the fragment stripped. The request and response are correlated by `threadId` per [The `threadId` Member](#49-the-threadid-member).
3. The fragments `#request` and `#response` are **RESERVED** for this purpose. An individual *Trust Task specification* **MUST NOT** assign other fragment meanings to its `type` URI.
4. A *failure* response is **not** a `#response`-variant document of the request's *Type URI*. Failures are reported via the framework's distinct `trust-task-error` *Type URI* per [Error Responses](#8-error-responses).
5. Consumers **MUST** preserve the fragment when comparing `type` URIs, when routing documents internally, and when keying hash maps on `type`. A consumer that strips the fragment before keying will conflate request and response documents.
6. The payload JSON Schema for a request/response pair is published as a single schema document at the bare *Type URI* (no fragment). Within that schema, the request payload shape is the top-level schema (or the schema reachable via `$anchor: "request"`); the response payload shape is reachable via `$anchor: "response"`. See [Specification Requirements](#73-specification-requirements) for the publishing requirements.

#### 4.4.2 Acknowledging a Fire-and-Forget Task

A specification that defines a fire-and-forget task — one with no expected success response document — declares no response sub-schema ([Specification Requirements](#73-specification-requirements) item 7.6). Its *consumer* **MAY** nonetheless return a document whose `type` is the originating *Type URI* with the fragment `#response` and whose `payload` is the empty object `{}`, as a **courtesy acknowledgement** that the document was received and performed.

The rules for that acknowledgement:

1. The `payload` **MUST** be exactly `{}`. The specification declares no response shape, so there is no shape for a member to validate against; a *consumer* **MUST NOT** place any member in it, and a party receiving one **MUST NOT** attribute meaning to anything it nonetheless contains.
2. A *consumer* receiving such an acknowledgement validates it against the framework schema alone and **MUST NOT** attempt to resolve a `response` anchor, which item 7.6 forbids the specification to declare.
3. A *producer* **MUST NOT** rely on receiving one. Its absence carries no information — the *consumer* may not implement acknowledgement, may implement it and stay silent, or the document may be lost — and a *producer* that reads absence as failure and reissues a *consequential Trust Task* causes exactly the duplicate effect [Consumer Requirements](#72-consumer-requirements) item 11 exists to prevent. This is the general rule of [Document Lifecycle](#412-document-lifecycle), not an exception to it.
4. A *consumer* **MUST NOT** send one for a specification that *does* define a success response. Two success dispositions for one task leave a *producer* unable to tell which is authoritative.

This form supersedes the reserved `trust-task-ok` slug, which exists for precisely this purpose and cannot serve it. `trust-task-ok` is a *different Trust Task specification* from the task it acknowledges, so a *producer* awaiting a reply must recognize a `type` unrelated to the one it sent; and it is defined so that it cannot carry meaning — a *consumer* **MUST NOT** require any member of it and a *producer* **MUST NOT** rely on receiving one — which leaves a registry entry, a schema, and a generated type in every language whose entire content is "something arrived". The same fact now travels on the reply a *producer* was already prepared to receive. `trust-task-ok` is **deprecated** accordingly; see [Reserved Response-Type Slugs](#86-reserved-response-type-slugs).

An acknowledgement that genuinely matters remains outside both forms. A task whose acknowledgement will be relied upon, audited, or disputed declares its own success response with its own payload and its own proof requirement, or a dedicated receipt task — the empty `{}` attests nothing beyond arrival, and is signed, where it is signed at all, only to that effect.

### 4.5 The `payload` Member

The `payload` member's value **MUST** be a JSON object whose internal structure is defined by the *Trust Task specification* identified by the document's `type`. This framework places no constraint on the contents of `payload` beyond requiring that it be an object.

The framework separates document-level metadata (`id`, `threadId`, `type`, `issuer`, `recipient`, `issuedAt`, `expiresAt`, `proof`) from task-specific data (`payload`) so that a single framework-level schema validates the outer structure, with per-task schemas applied only to `payload`. Schema scope is defined in [Schema Scope](#63-schema-scope).

#### 4.5.1 The `ext` Extension Member

A *Trust Task specification* **MAY** allow an `ext` member at the top level of `payload`, at any nested object whose contents the specification controls, or both. The `ext` member is the framework's sanctioned extension point for ecosystem-defined data that the base specification does not enumerate.

The framework reserves the following normative rules for any `ext` member, in any specification:

1. `ext` **MUST** be a JSON object when present.
2. Each *immediate* key of `ext` **MUST** match the reverse-DNS grammar `^[a-z][a-z0-9-]*(\.[a-z0-9-]+)+$` — lowercase, at least one dot. Examples: `vnd.affinidi.webvh`, `org.example.acl`. Bare keys without a namespace are non-conforming.
3. The structure under each namespace is opaque to the framework. Producers MAY place any JSON value the namespace's controller chooses to define.
4. A *producer* **MUST NOT** rely on any framework-level meaning for the contents of any `ext.*` namespace.
5. A *consumer* **MUST** ignore namespaces it does not recognize, consistent with the unrecognized-member rule of [Consumer Requirements](#72-consumer-requirements). A *consumer* **MAY** require its own namespace as a matter of local policy and reject documents lacking that namespace with `malformedRequest`.
6. The framework reserves **no** `ext.*` namespace today. *Trust Task specifications* **MUST NOT** define cross-specification semantics for any `ext` key; ecosystem semantics belong to the namespace controller.

A *Trust Task specification* opts into `ext` at a given object level by including a property named `ext` (typically a `$ref` to the framework's published `Ext` `$def`) and adjusting that level's `additionalProperties` declaration accordingly. Specifications that do not include `ext` at a given level reject the member at that level under their existing `additionalProperties: false`.

The signed envelope covers `ext` in the same way it covers any other member of `payload`, so `ext` inherits the integrity guarantees of [Proof](#47-proof) when a `proof` is present.

`ext` is distinct from the task-specific `details` member of a `trust-task-error` response ([Extension by Individual Trust Task Specifications](#85-extension-by-individual-trust-task-specifications)). `details` carries structured data tied to a specific error `code` defined by the spec author; `ext` carries vendor-namespaced extension data defined by the ecosystem. Both members **MAY** appear on the same document and are not interchangeable.

### 4.6 JSON-LD Compatibility

A *Trust Task document* **MAY** include an `@context` member. If present, the document **MUST** be processable as JSON-LD; the framework places no further constraint on the contents of `@context` beyond requiring it to be a string, an array of strings or objects, or an object, in line with the JSON-LD specification. A *Trust Task specification* that wishes to declare a canonical JSON-LD context **MUST** publish it at its *Type URI* under content negotiation for `application/ld+json` (see [Content Negotiation](#62-content-negotiation)).

A *consumer* that does not implement JSON-LD processing **MUST** ignore the `@context` member and process the document as plain JSON. JSON-LD support is therefore strictly additive — no consumer is required to implement it, and no document is required to include it.

> **Example 2 — A Trust Task document with a JSON-LD context** *(non-normative)*
>
> ```json
> {
>   "@context": [
>     "https://www.w3.org/ns/credentials/v2",
>     "https://trusttasks.org/spec/acl/change-role/0.1"
>   ],
>   "id": "urn:uuid:7d8b1e3a-9a72-4f86-9d04-2a4b6c2c5e10",
>   "type": "https://trusttasks.org/spec/acl/change-role/0.1",
>   "issuer": "did:web:org.example",
>   "recipient": "did:web:maintainer.example",
>   "issuedAt": "2026-06-10T14:00:00Z",
>   "payload": {
>     "subject": "did:web:bob.example",
>     "fromRole": "member",
>     "toRole": "moderator"
>   }
> }
> ```
>
> A *consumer* that implements JSON-LD processes the document accordingly; a *consumer* that does not implement JSON-LD ignores `@context` and processes the same document as plain JSON. The two interpretations validate against the same payload schema. The second `@context` entry shows where a specification's own context appears; the registered `acl/change-role` 0.1 does not publish one, and a *Trust Task specification* that wishes to declare a canonical context publishes it at its *Type URI* under content negotiation for `application/ld+json` as this section requires. The `proof` the specification requires is omitted throughout this example, which turns on `@context` alone.

### 4.7 Proof

A *Trust Task document* **MAY** include a `proof` member whose value is a W3C *Data Integrity Proof* object as defined in [VC Data Integrity](https://www.w3.org/TR/vc-data-integrity/). When present, the `proof` binds the document's content to its `issuer`.

The choice of cryptographic suite is open: any suite registered by the W3C Verifiable Credential Working Group (for example, `eddsa-rdfc-2022` or `ecdsa-rdfc-2019`, or any future suite) **MAY** be used. The `verificationMethod` of the proof **MUST** resolve to verification material controlled by the *party* identified by the document's `issuer` member (see [The `issuer` and `recipient` Members](#48-the-issuer-and-recipient-members)).

When `proof` is present, it covers the document with `proof` itself excluded from the signed content, per the canonicalization rules of the chosen Data Integrity suite.

#### 4.7.1 When to Include a Proof

The default rules governing the presence of `proof` in a *Trust Task document* are:

* If the document is delivered over a transport that already provides end-to-end integrity and authentication between *producer* and *consumer* — for example, mutually-authenticated TLS or a signed DIDComm envelope — `proof` **MAY** be omitted.
* If the document is delivered over a transport that does not provide such guarantees, or where tampering or substitution by intermediaries is possible, `proof` **SHOULD** be included.
* If a strong, transport-independent guarantee of non-tampering and of *producer* identity is required — typically because the document is intended to be retained, replayed, or relied on by parties beyond the original *consumer* — `proof` **MUST** be included.

Whenever `proof` is included, the [Audience Binding](#482-audience-binding) rule also applies: the *producer* commits to an in-band `recipient` so that the proof binds not only the content but also the intended audience.

An individual *Trust Task specification* **MAY** strengthen these defaults (for example, mandate `proof` regardless of transport) but **MUST NOT** weaken them. The declaration each *Trust Task specification* makes about its own `proof` requirement is governed by [Specification Requirements](#73-specification-requirements).

### 4.8 The `issuer` and `recipient` Members

A *Trust Task document* **MAY** identify the parties involved by including the `issuer` and `recipient` members at the top level of the document.

* `issuer` — a *Verifiable Identifier* (see [Terminology](#3-terminology)) identifying the *party* responsible for the document's content. When `proof` is present, the `issuer` **MUST** identify the entity to which the proof's `verificationMethod` resolves.
* `recipient` — a *Verifiable Identifier* identifying the *party* the *issuer* expects to act upon the document.

The framework does not constrain the VID scheme used: a DID, an X.509 subject, an OIDC subject identifier, a key thumbprint, or any other identifier whose controller is verifiable under the *consumer*'s trust framework is acceptable.

A *VID* is compared by exact string equality wherever this framework requires a VID-to-VID comparison (notably the in-band-vs-transport cross-check in [Precedence of In-Band over Transport-Derived Identity](#481-precedence-of-in-band-over-transport-derived-identity), the recipient-enforcement rule in [Consumer Requirements](#72-consumer-requirements) item 5, and the proof-binding rule in [Proof](#47-proof)). *Producers* **SHOULD** emit *VID*s in their canonical form for the scheme in use — no leading or trailing whitespace, no normalization of case-sensitive segments, and (for schemes that admit equivalent forms) the form that the scheme's authority designates as canonical. A *consumer* **MAY** reject a *Trust Task document* whose `issuer`, `recipient`, or any *VID*-typed `payload` member is not in canonical form with `malformedRequest`; a *consumer* that accepts non-canonical input **MUST NOT** silently normalize before applying any framework rule that compares the value — normalization changes the string, and the framework's comparisons are over the unchanged bytes.

Both members are **OPTIONAL**. Their purpose is to let the parties be identified in-band where the transport in use does not already convey strong, authenticated party identity — for example, an unauthenticated HTTP POST, a public message queue, or paper hand-off.

Where a secure transport already conveys authenticated party identity (such as mutually-authenticated TLS or a signed DIDComm envelope), these in-band members **MAY** be omitted.

#### 4.8.1 Precedence of In-Band over Transport-Derived Identity

The framework treats the in-band `issuer` and `recipient` members as **authoritative** for party identity. Specifically, for each party:

1. **If the in-band member is present**, its value is the party identity that the *consumer* **MUST** apply for every subsequent framework rule that references that party — including, but not limited to, `proof` verification (where applicable, see [Proof](#47-proof)), recipient enforcement (see [Consumer Requirements](#72-consumer-requirements), item 5), and *Trust Task specification* requirements that reference the party. The transport-derived identity is, in this case, **only a cross-check**: where both an in-band identity and a transport-derived identity are present for the same party, they **MUST** be consistent, and a *consumer* **MUST** treat a mismatch as a validation failure (see [Consumer Requirements](#72-consumer-requirements), item 6).
2. **If the in-band member is absent**, a *consumer* **MAY** derive the party identity from the transport — typically via the *transport binding* in use (see [Transport Bindings](#9-transport-bindings)) — and **MAY** treat the derived value as if it had been carried in-band for the purposes of subsequent rules. A *consumer* with no in-band value and no transport-derived value for a party that the *Trust Task specification* declares as **REQUIRED** **MUST** reject the document.

In short: the document is the source of truth for who the parties are. The transport, when it provides authenticated identity, is used either to fill in what the document omits, or to verify what the document asserts — never to override it.

An individual *Trust Task specification* **MAY** require either or both members to be present — for example, to support audit, third-party replay, or forwarding — but **MUST NOT** prohibit a *consumer* from comparing them with transport-derived identity.

> **Example 3 — A Trust Task document using non-DID *Verifiable Identifiers*** *(non-normative)*
>
> ```json
> {
>   "id": "urn:uuid:0e9d4c2b-5f81-4d3e-9b51-7a3c89e3d1f2",
>   "type": "https://trusttasks.org/spec/acl/change-role/0.1",
>   "issuer": "x509:CN=Operations,O=Example Org,C=NL",
>   "recipient": "x509:CN=Directory,O=Example Org,C=NL",
>   "issuedAt": "2026-06-10T14:00:00Z",
>   "payload": {
>     "subject": "oidc:https://issuer.example/sub#user-94217",
>     "fromRole": "member",
>     "toRole": "moderator"
>   }
> }
> ```
>
> Here `issuer` and `recipient` are X.509 subject distinguished names and `payload.subject` is an OIDC subject identifier. The framework treats any string identifier whose controller is verifiable under the *consumer*'s trust framework as a valid *VID*; DIDs are one realization among several. The `proof` the named specification requires is omitted for brevity; it plays no part in the point this example makes.

#### 4.8.2 Audience Binding

When a *Trust Task document* carries a `proof` member, the document **MUST** also carry an in-band `recipient` member, unless the *Trust Task specification* identified by the document's `type` declares itself a *bearer specification* (see [Bearer Specifications](#483-bearer-specifications)).

This rule exists because a *Data Integrity Proof* covers the signed bytes — the *issuer*, *payload*, and other framework members — but does **not** cover any transport-derived identity. A document signed without an in-band `recipient` therefore provides no cryptographic binding between the *producer*'s assertion and the intended audience: an attacker who obtains the document — from a *consumer*'s storage, an intermediate cache, or an exfiltration — can replay the bytes to a different *consumer* without any signal that the original *producer* did not intend that audience to act upon them. A consumer receiving such a replayed document would otherwise verify the proof successfully, observe that no `recipient` constrains the assertion, and apply the producer's claim to its own context.

A *consumer* receiving a `proof`-carrying document with no in-band `recipient`, where the originating *Trust Task specification* is not a *bearer specification*, **MUST** reject the document with a `malformedRequest` *error response* (see [Error Responses](#8-error-responses)).

Specifications that declare `proof` as **REQUIRED** (see [Specification Requirements](#73-specification-requirements) item 8) implicitly require `recipient` in-band for all non-bearer cases; the audience-binding rule and the proof requirement combine to ensure the document is self-contained for both producer identity and intended audience.

#### 4.8.3 Bearer Specifications

A *Trust Task specification* whose `payload` carries an assertion meaningful to any *consumer* that can verify the *producer*'s identity — for example, a public attestation, a heartbeat, or a schema-publication announcement — **MAY** opt out of the [Audience Binding](#482-audience-binding) rule by declaring itself a *bearer specification*. The opt-out is published in the specification's front matter (see [Specification Requirements](#73-specification-requirements) item 12).

A *bearer specification* makes an explicit, normative claim that documents conforming to it are intended for unspecified consumption: any party that can verify the document's `proof` (where present) is a legitimate recipient.

A *bearer specification* **MUST**:

1. Declare `bearer: true` in its front matter.
2. Declare its `recipient` party requirement as **OPTIONAL** (the audience-binding rule no longer applies).
3. State in its prose what assertion the document conveys and why audience binding is inappropriate for it.

A *bearer specification* **SHOULD NOT** carry any field in `payload` whose interpretation depends on the receiving party's identity (for example, "balance owed *to you*"); such fields belong in audience-bound specifications.

The default for any *Trust Task specification* is **non-bearer**. Specifications **MUST NOT** declare themselves bearer unless the audience-free property is intrinsic to the assertion they publish.

### 4.9 The `threadId` Member

Every *Trust Task document* carries its own unique `id` ([The `id` Member](#43-the-id-member)); a response document **MUST NOT** reuse the `id` of the document it is responding to. Correlating documents back to one another — for example, linking a response to its originating request — is the purpose of the `threadId` member, not the `id` member.

A *Trust Task document* **MAY** include a `threadId` member that correlates it with other *Trust Task documents* belonging to the same logical exchange — for example, a request and its response, or a request, an intermediate `trust-task-next-step` response, and the final result.

A *producer* that emits a *Trust Task document* in response to another *Trust Task document* **SHOULD** set `threadId` to the value of the originating document's `threadId`. If the originating document carried no `threadId`, the *producer* **SHOULD** set `threadId` to the value of the originating document's `id`. The effect of this convention is that every document in a logical exchange carries the same `threadId`, and that value can always be traced back to the `id` of the document that started the thread.

The framework places no constraint on the form of a `threadId` beyond requiring it to be a string. Producers initiating a new exchange **MAY** omit `threadId` entirely (single-shot tasks need no thread), **MAY** mint a fresh value (e.g. a UUID), or **MAY** reuse the document's own `id`.

`threadId` carries no normative validation semantics. *Consumers* **MUST NOT** reject a document on the basis of `threadId` alone, but **MAY** use it for routing, correlation, aggregation, or audit.

#### 4.9.1 Naming an Exchange from Outside the Framework

A `threadId` names one exchange and expresses no relationship to any other. Exchanges nest in practice — a *Trust Task* conducted to complete a step of some broader interaction is still its own exchange, with its own `threadId`. The optional `parentThreadId` member ([The `parentThreadId` Member](#492-the-parentthreadid-member)) records that containment, but it is a navigation aid: it does not change which exchange attests an event, and the rule below holds whether or not it is present.

This matters whenever something outside the framework refers to an exchange as evidence that an event occurred: a credential that cites the exchange which established what it attests, an audit record, a governance decision that turns on some task having been performed. Nesting makes the reference ambiguous, because more than one thread was open when the event happened, and only one of them attests it.

The rule is that such a reference **MUST** name the *innermost* exchange whose documents attest the event being cited, and **MUST** name it by the `id` of the document that initiated that exchange — the value every document in the thread traces back to under the convention above ([The `id` Member](#43-the-id-member) makes that `id` globally unique and non-reusable, which a `threadId` is not required to be).

Naming an enclosing exchange instead collects evidence of the wrong event. Where a witnessing ceremony is conducted inside a broader relationship exchange, for example, only the ceremony's own response attests that the witnessing took place; the enclosing exchange's response attests the relationship interaction and says nothing about the witnessing. A consumer verifying the outer reference would conclude something the documents do not support.

Naming the right exchange is necessary but not sufficient: an `id` identifies a document without binding the citation to it, so a citation relied upon outside the exchange also carries a digest over the document it names. See [Binding a Citation to the Document It Names](#493-binding-a-citation-to-the-document-it-names).

#### 4.9.2 The `parentThreadId` Member

A *Trust Task document* **MAY** include a `parentThreadId` member whose value is the `threadId` of the exchange that contains this one. Its purpose is navigation: it lets a party holding a document from the inner exchange find the exchange it was conducted within, which a flat `threadId` cannot express.

The member takes the same posture as `threadId` ([The `threadId` Member](#49-the-threadid-member)):

* A *producer* that emits a *Trust Task document* within an enclosing exchange **SHOULD** set `parentThreadId` to that exchange's `threadId`, and **SHOULD** carry the same value onto every document of the inner exchange — including its *response* and any *error response* — since the whole exchange shares one parent.
* A *producer* **MUST NOT** set `parentThreadId` equal to the document's own `threadId`. An exchange cannot contain itself.
* `parentThreadId` carries no normative validation semantics. *Consumers* **MUST NOT** reject a document on the basis of `parentThreadId` alone, but **MAY** use it for routing, correlation, aggregation, or audit.

The member records **one** level of containment. Reconstructing a deeper ancestry requires the intervening documents, and the framework defines no representation for a full chain; a specification needing one is better served by an explicit payload structure than by inferring it from thread metadata.

Where the transport carries its own parent-thread concept, the two **MUST** agree when both are present, and the in-band member remains authoritative for framework-level processing; see [What a Transport Binding Specifies](#91-what-a-transport-binding-specifies). A transport binding that maps the two states the rule for its own protocol.

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
> A credential citing the witnessing as evidence anchors to this inner exchange, per [Naming an Exchange from Outside the Framework](#491-naming-an-exchange-from-outside-the-framework) — the enclosing exchange attests the relationship interaction, not the witnessing. The `parentThreadId` is what lets a holder of this document find that enclosing exchange; it is not what the citation names.

> **Example 4 — Request and response correlated by `threadId`** *(non-normative)*
>
> A *producer* issues an initiating *Trust Task document*:
>
> ```json
> {
>   "id": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
>   "type": "https://trusttasks.org/spec/acl/change-role/0.1",
>   "issuer": "did:web:org.example",
>   "recipient": "did:web:maintainer.example",
>   "issuedAt": "2026-06-10T14:00:00Z",
>   "payload": { "subject": "did:web:bob.example", "fromRole": "member", "toRole": "moderator" }
> }
> ```
>
> It carries no `proof`, and the specification it names declares one **REQUIRED**, so the *consumer* refuses it. The original document carried no `threadId`, so the responding *party* sets `threadId` to the originating document's `id`:
>
> ```json
> {
>   "id": "8a91c7b3-2e62-4a91-a3a4-9d61b75e2f01",
>   "type": "https://trusttasks.org/spec/trust-task-error/0.2",
>   "threadId": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
>   "issuer": "did:web:maintainer.example",
>   "recipient": "did:web:org.example",
>   "issuedAt": "2026-06-10T14:00:02Z",
>   "payload": { "code": "proofRequired", "retryable": false }
> }
> ```
>
> Both documents now share `threadId = 4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2`; any subsequent document in this exchange — for example, a retry with a fresh `id` and a valid `proof` — would carry the same `threadId`.

#### 4.9.3 Binding a Citation to the Document It Names

[Naming an Exchange from Outside the Framework](#491-naming-an-exchange-from-outside-the-framework) requires an external citation to name an exchange by the `id` of the document that initiated it. An `id` is a *name*. [The `id` Member](#43-the-id-member) obliges a *conforming producer* to mint it globally unique and never to reuse it, but that obligation constrains conforming producers and nobody else: anyone may write a different document — different parties, different `payload` — and give it the same `id`. A consumer that pairs a citation with a document by comparing `id`s alone accepts that substitute, and then reports an event the documents do not attest.

A citation that will be relied upon by parties outside the exchange **SHOULD** therefore carry, alongside the `id`, a **task digest** over the document it names. Where a citation carries one, it **MUST** be computed as:

```
taskDigest = multibase( multihash( H( JCS( document ∖ proof ) ) ) )
```

where:

* `document ∖ proof` is the *Trust Task document* with its **top-level `proof` member removed** where present, and no other member removed or added. A `proof` appearing *within* `payload` — in an embedded presentation, credential, or other artifact — is part of that payload's content and **MUST NOT** be removed. Where the document carries no top-level `proof`, the input is the document unchanged; no placeholder is substituted.
* `JCS` is the [RFC 8785](https://www.rfc-editor.org/rfc/rfc8785) canonicalization, serialized as UTF-8, as used elsewhere in this framework.
* `H` is any hash function expressible in multihash; the multihash prefix declares which, so the algorithm is not fixed by this specification and survives an algorithm change without a format revision. SHA-256 is **RECOMMENDED**.
* The result is encoded as a multibase-encoded multihash, per the `DigestMultibase` definition of the framework's *shared schema component* ([Shared Schema Components](#66-shared-schema-components)).

The top-level `proof` is excluded because [Proof](#47-proof) already defines it as excluded from the content a `proof` covers. The digest and the document's own signature therefore commit to the same content, and a document has exactly **one** task digest whether or not anyone ever signed it. A digest whose input included `proof` would be undefined for the documents [When to Include a Proof](#471-when-to-include-a-proof) permits to carry none, and would change value the moment a document was signed — so a citing party and a verifying party computing it at different points in the document's life would disagree.

A *consumer* verifying a citation **MUST** recompute the digest from the document it holds — removing the top-level `proof` first, where present — and **MUST** compare the **decoded multihash bytes**, not the encoded strings. `DigestMultibase` admits more than one base encoding, and two conforming encodings of the same digest are different strings; a string comparison rejects a valid pairing. A *consumer* that does not implement the hash algorithm named by the multihash prefix **MUST** treat the citation as unverified: it **MUST NOT** recompute under a different algorithm, and **MUST NOT** fall back to `id` comparison alone.

**This is not the document identity of [Consumer Requirements](#72-consumer-requirements) item 11.** The two answer different questions and are deliberately computed differently. Item 11 and [Retry Semantics](#84-retry-semantics) ask *which serialization arrived*, so a re-signed `proof` over identical content makes a different document — that is the `idConflict` case, and the distinction is the whole point of the rule. A citation asks *what the document says*, so the same statement signed, unsigned, or re-signed is one document with one task digest. A specification requiring the bytes-as-received sense — [`trust-ceremony-receipt`](https://trusttasks.org/spec/trust-ceremony-receipt/0.1)'s step digest, for one — states so and computes over the document including its `proof`; it is not applying this section loosely.

The framework names the two, because both have until now been called `digestMultibase` in the places they appear and the difference between them is load-bearing:

| | **`taskDigest`** | **`stepDigest`** |
|---|---|---|
| Asks | *What does the document say?* | *Which serialization arrived?* |
| Input | The document with its **top-level `proof` removed** | The document **including** its top-level `proof`, plus any salt the defining specification requires |
| The same statement, signed and unsigned | One value | Two values |
| The same content, re-signed | One value | Two values |
| Defined by | This section | The specification that requires the bytes-as-received sense |
| Used for | Binding a citation made outside the framework to the document it names | Chain of custody: the step chain of an *enactment* (`ceremony.prev`, `trust-ceremony-receipt`), and the document identity [Consumer Requirements](#72-consumer-requirements) item 11 keys `idConflict` on |

Both are computed with JCS, a multihash-tagged hash, and a multibase encoding over a *Trust Task document*, and both are carried in a member of the `DigestMultibase` shape ([Shared Schema Components](#66-shared-schema-components)). The **only** structural difference is the treatment of the top-level `proof`, and it is the whole of the difference in meaning: excluding it makes the digest a property of the *statement*, so that a document has one task digest whether or not anyone ever signed it; including it makes the digest a property of the *artifact*, so that a re-signed document is a different document — which is exactly what a custody chain and an `idConflict` check need it to be.

These are names for prose and for implementations. Neither is a document member: the members that carry these values keep the names their own specifications give them. The naming exists because an implementation with a single function called `digestMultibase` has, in practice, picked one of these two answers and is applying it to both questions — and whichever it picked, it is wrong for one of them.

> **What a task digest establishes, and what it does not** *(this note is non-normative)*
>
> Recomputation is unconditional: content that differs cannot produce the value, and there is no string for a substitute document to copy. That is why a digest is preferred here to the cited document's own `proof` value. A `proofValue` is a string, and a string can be pasted verbatim onto a counterfeit; it binds only behind full signature verification — canonicalize, hash, resolve the signer's verification method, verify — which costs more than the digest recompute and is unavailable entirely for the documents [When to Include a Proof](#471-when-to-include-a-proof) permits to carry no `proof` at all.
>
> The digest attests **content**, not authenticity. It is load-bearing because the *citing artifact* carries it under the citer's own signature: a party cannot obtain a signature over a digest of a document the signer never saw, so a counterfeit that borrowed the `id` fails the pairing no matter who wrote it. It says nothing about whether the cited document itself was signed, and a `proof`-stripped copy of a genuine document reproduces the same value by design. Authenticity of the exchange comes from the documents' own proofs under [When to Include a Proof](#471-when-to-include-a-proof) — a specification whose citations must also establish that the cited document was attributable **MUST** require a `proof` on it, and **MUST NOT** rely on the task digest for that.

#### 4.9.4 Evidence That a Cited Exchange Completed

[Binding a Citation to the Document It Names](#493-binding-a-citation-to-the-document-it-names) fixes *which* exchange a citation names. It says nothing about whether that exchange **completed**. A credential can cite an exchange that was cancelled, that failed, or that is still open, and its citation still binds — the initiating document exists in every one of those cases. A party relying on a citation as evidence that the cited event occurred therefore needs a second artifact: a terminal document showing that the task reached `responded` ([Document Lifecycle](#412-document-lifecycle)). This section defines that pairing once, so that a *Trust Task specification* whose exchanges are cited states only what is particular to it.

*Outcome evidence* for a cited exchange is the exchange's initiating document together with a terminal document satisfying the rules below. A *consumer* relying on a citation as evidence that the cited exchange completed **MUST** confirm all four, and **MUST NOT** treat the citation as evidence of completion otherwise:

1. **The citation binds the initiating document.** The initiating document's `id` equals the citation, and the citation's task digest reproduces over that document under [Binding a Citation to the Document It Names](#493-binding-a-citation-to-the-document-it-names).
2. **The terminal document belongs to that exchange.** Its `threadId` equals the initiating document's `threadId` where the initiating document carries one, and the initiating document's `id` where it does not. Pairing runs through the initiating document because [The `threadId` Member](#49-the-threadid-member) permits an initiator to mint a `threadId` unrelated to its `id`; a rule that compares the terminal's `threadId` with the citation directly rejects every conforming exchange whose initiator did so. A matching `threadId` is necessary but not sufficient: a `threadId` is not required to be unique ([Naming an Exchange from Outside the Framework](#491-naming-an-exchange-from-outside-the-framework)), and the obligation not to carry one across exchanges ([Identifier Correlation and Linkability](#131-identifier-correlation-and-linkability) item 2) binds conforming producers and nobody else. The terminal document **MUST** therefore also satisfy the binding to the initiating document that the governing *Trust Task specification* declares under [Specification Requirements](#73-specification-requirements) item 20.5, which its own `proof` covers.
3. **The terminal document is the declared outcome evidence.** Its `type` is a success response that the *Trust Task specification* governing the initiating document declares as that exchange's outcome evidence under [Specification Requirements](#73-specification-requirements) item 20. A document of any other type in the same thread attests something else: an *error response* reports `errored`, or `cancelled` where the *consumer* stopped of its own accord; the response to a `trust-task-control` document reports that the task was **stopped** at the *producer*'s request ([Task Control](#11-task-control)), and is itself a `#response`; `trust-task-next-step` reports it **blocked** ([Reserved Response-Type Slugs](#86-reserved-response-type-slugs)). The declaration is needed because an exchange need not close with the response to its own initiating document: a session opened under one specification may close with the response to a task conducted on its thread under another.
4. **The terminal document was produced by the party the exchange was addressed to.** Its `proof` verifies, and the *party* it verifies under is the initiating document's `recipient`, which item 20 requires the initiating document to carry; where it is absent, this check fails. A proof that verifies under the document's own `issuer` shows only that someone signed it — a party holding the initiating document can sign a well-formed response itself. The responder is not in general the issuer of the citing artifact: where a vetter opens a session with an applicant and later issues the credential citing it, the terminal response is the applicant's.

A *consumer* that does not hold outcome evidence satisfying these rules **MUST** treat the citation as not evidencing completion, whether or not such evidence exists elsewhere. That does not by itself invalidate the citing artifact, whose own validity is a matter for the specification that defines it. Discovering or retrieving outcome evidence a *consumer* does not already hold is out of scope for this version of this framework: a party relying on a citation as evidence of completion **SHOULD** carry both documents of the outcome evidence with the citing artifact, and the governing specification's retention class ([Specification Requirements](#73-specification-requirements) item 18) states that they are expected to be kept for that purpose.

> **Why four checks, and not fewer** *(this note is non-normative)*
>
> Each check closes a case that a rule without it decides wrongly. Without check 2's route through the initiating document, an honest exchange whose initiator minted its `threadId` fails; without its declared binding, a genuine success response from an earlier exchange whose initiator reused the `threadId` with the same counterparty reads as completion of a later one. The binding is to the initiating document's `id` and task digest wherever it can be: a fresh value such as a challenge is also minted by the initiator. So it closes that case only where the initiator is the party whose word the citing artifact already rests on, and item 20.5 permits it only there. Without check 3, a cancelled task's control response — a `#response` in the same thread, under a valid proof — reads as completion. Without check 4, a response signed by the holder of the initiating document reads as the counterparty's. And naming the terminal type as *the initiating document's type with the `#response` fragment*, rather than as a declaration, rejects the witnessing ceremony: it opens with `witness/session` and closes with `witness/session/submit#response`.
>
> [`witness/session/submit/0.1`](https://trusttasks.org/spec/witness/session/submit/0.1) and [`vetting/session/0.1`](https://trusttasks.org/spec/vetting/session/0.1) each state their own pairing today, and each is consistent with this section for its own exchange: both require the initiator to set `threadId` equal to `id`, and both name their evidence. What neither can do is generalize — the first identifies the responder as the credential's issuer, which is true of witnessing and not of vetting — which is why the rule is stated here, once. Neither yet makes the declaration item 20 asks for — `witness/session/0.1` because that declaration belongs to the specification governing the initiating document rather than to `witness/session/submit/0.1`, `vetting/session/0.1` because it names no binding — though each already carries the binding it would name: the submit response's `vwc` carries `taskContext` and `taskDigestMultibase`, and the vetting response's `card` is presented against the session's freshly generated `challenge`.

### 4.10 Naming Conventions

JSON member names and enumerated string values in *Trust Task documents* follow the casing rules below, so that documents are consistent across specifications both for human readers and for code generators.

1. **Framework-defined members.** Every member defined by this framework — `id`, `threadId`, `parentThreadId`, `type`, `issuer`, `recipient`, `issuedAt`, `expiresAt`, `payload`, `proof`, and the members of the error payload in [Error Payload](#82-error-payload) — **MUST** be named in **lowerCamelCase**. The sole exception is `@context`, which is named as required by JSON-LD.

2. **Framework-defined values.** Enumerated string values defined by this framework — notably the standard error `code` identifiers of [Standard Error Codes](#83-standard-error-codes) — **MUST** be expressed in **lowerCamelCase**.

3. **Payload member names.** A *Trust Task specification* **SHOULD** name the members of its `payload` in lowerCamelCase. A specification **MAY** deviate only where it embeds a member whose name is fixed by an external vocabulary (for example, a field copied verbatim from a WebAuthn or JOSE structure), and it **SHOULD** confine such foreign naming to the embedded sub-object.

4. **Specification-defined values.** String values drawn from a closed set that a *Trust Task specification* itself defines — statuses, kinds, decisions, event types, extended error `code` identifiers — **SHOULD** be expressed in lowerCamelCase (for example, `cacheAndKeys`, `stepUp`, `proofInvalid`).

5. **Externally-owned values.** A value whose canonical form is fixed by an external specification **MUST** be carried verbatim and **MUST NOT** be re-cased, because the framework compares such values by exact string equality (see [The `issuer` and `recipient` Members](#48-the-issuer-and-recipient-members)). Examples include WebAuthn enumerations (`public-key`, `cross-platform`), JOSE algorithm identifiers (`EdDSA`, `ES256`), cookie `SameSite` values (`Lax`, `Strict`), and W3C *Data Integrity* type and purpose values (`DataIntegrityProof`, `assertionMethod`).

6. **Out of scope.** This section does not govern *slugs* (lowercase, hyphen-separated; [Type URI](#61-type-uri)) or `ext` namespace keys (reverse-DNS; [The `ext` Extension Member](#451-the-ext-extension-member)); each retains its own grammar.

A change to the casing of an existing member name or specification-defined value is a breaking change and follows the rules of [Versioning](#5-versioning); the re-casing introduced in framework version 0.2 is recorded in [Appendix B](#appendix-b-changelog).

### 4.11 The `ceremony` Member

Some outcomes take more than one *Trust Task*. A governance decision may need several endorsements; an onboarding may span a witness and a registry. The framework's model for these is settled in [Terminology](#3-terminology) — they are multiple bilateral *Trust Tasks* — but the collection itself has, until this version, had no name, no identifier, and no way to be evidenced.

A *Trust Ceremony* is such a collection: a flow of *Trust Tasks*, optionally described by a published *ceremony definition* ([Ceremony Namespace](#67-ceremony-namespace)), of which one run is an *enactment*. A *Trust Task document* **MAY** carry a `ceremony` member recording that it is one *step* of an enactment.

The member's value is an object with the following members. Its full schema is published with the framework envelope schema for this version.

| Member | Required | Type | Description |
|---|---|---|---|
| `enactment` | **MUST** | string | Identifies one run of a ceremony. Globally unique and never reused, on the same terms as `id` ([The `id` Member](#43-the-id-member)). |
| `step` | **MUST** | string | Names this step within the ceremony. |
| `definition` | **MAY** | string (URI) | The *ceremony definition* this step is enacted under, rooted at [Ceremony Namespace](#67-ceremony-namespace). |
| `definitionDigest` | **MUST** where `definition` is present | string | A multibase-encoded multihash over the [RFC 8785](https://www.rfc-editor.org/rfc/rfc8785) canonicalization of that definition. |
| `parentEnactment` | **MAY** | string | The enactment containing this one, where a ceremony is conducted as a step of another. |
| `round` | **MAY** | integer | Distinguishes repetitions of the same step by the same party. Absent means `1`. |
| `terminal` | **MAY** | boolean | Marks a step that ends the enactment. |
| `prev` | **MAY** | array | The steps this one follows, each an object of `id` and `digestMultibase`. |

<a id="4111-optionality"></a>
#### 4.11.1 Ceremony Membership Is Optional

The `ceremony` member is optional in every sense that matters, and this is a normative property rather than a convenience:

1. A *Trust Task specification* **MUST NOT** declare anything about ceremonies, and needs no awareness of them. The member is carried on the document, not in `payload`, so any existing specification may be used as a ceremony step with no change to its schema and no new version.
2. A *Trust Task document* without the member is fully conforming.
3. A *consumer* that does not implement ceremonies **MUST** process such a document exactly as it processes any other, under the unrecognized-member rule of [Consumer Requirements](#72-consumer-requirements).

#### 4.11.2 The Identifiers Are Orthogonal

`enactment` does not replace `threadId` and is not a form of `parentThreadId`. Within a ceremony, `threadId` scopes one step's request/response exchange exactly as it does elsewhere, and `enactment` scopes the flow across all of its steps; a *producer* sets both. The steps of an enactment are typically *siblings* — several top-level exchanges, none conducted inside another — which is containment's opposite and not what `parentThreadId` records.

The distinction that matters for evidence is that `enactment` **MUST** be globally unique and non-reusable where `threadId` need not be ([The `threadId` Member](#49-the-threadid-member)). A reference naming a flow as evidence therefore names the `enactment`, under the rule of [Naming an Exchange from Outside the Framework](#491-naming-an-exchange-from-outside-the-framework).

<a id="4113-integrity"></a>
#### 4.11.3 Ceremony Integrity

Where a `proof` is present it covers the `ceremony` member as it covers any other ([Proof](#47-proof)). This is the member's placement rationale, not an incidental consequence: a signed `enactment` cannot be lifted into a different flow, and a signed `definitionDigest` cannot be reinterpreted under a definition that gives the step's name another meaning. Carried as transport metadata or as an unsigned sidecar, the member would provide neither guarantee.

A *producer* **MUST NOT** set `parentEnactment` equal to the document's own `enactment`; an enactment cannot contain itself. A *producer* **SHOULD** carry the same `ceremony.enactment` onto every document of the step it names, including any *error response*.

#### 4.11.4 Membership Is a Claim, Not a Permission

A `ceremony` member is an assertion by the document's *issuer* that this document belongs to the named enactment. A *consumer* can check what it holds — that a step matches the definition, that a `prev` digest resolves — but cannot verify from one document that the enactment exists as described.

Accordingly:

> A *consumer* **MUST NOT** grant any authority on the basis of ceremony membership alone.

Every authorization decision continues to be reached under [Consumer Requirements](#72-consumer-requirements) item 10, exactly as for a document carrying no `ceremony` member — and note that verifying `issuer` and `proof` is not by itself such a decision. Without this rule the member would be a confused-deputy vector: "you are in the onboarding ceremony, so perform this step" is an unauthenticated assertion by whoever composed the document. The rule is also what makes [Ceremony Membership Is Optional](#4111-ceremony-membership-is-optional) item 3 safe — because membership authorizes nothing, a *consumer* that ignores the member entirely omits nothing it was entitled to do.

`ceremony` otherwise carries no normative validation semantics: a *consumer* **MUST NOT** reject a document on the basis of the member alone, and **MAY** use it for routing, correlation, aggregation, or audit.

### 4.12 Document Lifecycle

Every rule in this framework that governs what a *consumer* may do next presupposes a state the document is in. [Consumer Requirements](#72-consumer-requirements) item 4 governs *acceptance*; item 11 governs a document *already accepted*; item 12 governs execution *under way*; [Task Control](#11-task-control) suspends and cancels *accepted* work; [Error Responses](#8-error-responses) reports a *refusal*. Each of those sections describes the state it needs and none of them names the whole set, so implementations have inferred five overlapping lifecycles from one document.

The states below are that set, stated once. They are normative: the state names are the vocabulary the rest of this specification's cross-references are to be read against, and a *transport binding* maps them onto its own protocol under the rule at the end of this section.

| State | Reached when | Permitted next states | Reserved reply that carries the transition | What silence in this state means |
|---|---|---|---|---|
| `received` | The *consumer* holds the document's bytes and has not yet validated them. | `validated`, `errored`, `expired` | — | **Nothing.** A *producer* cannot distinguish a document in `received` from one that never arrived. |
| `validated` | Every applicable check of [Consumer Requirements](#72-consumer-requirements) items 1–10 has passed. | `accepted`, `errored` | — | **Nothing.** Validation is internal to the *consumer* and is never signalled on the wire. |
| `accepted` | The *consumer* has committed to execute and has written the duplicate-execution record of [Consumer Requirements](#72-consumer-requirements) item 11. | `executing`, `suspended`, `cancelled`, `errored` | `trust-task-next-step` — reported understood but **blocked**; the document remains `accepted` and the exchange stays open ([Reserved Response-Type Slugs](#86-reserved-response-type-slugs)). | **Nothing.** Acceptance is not acknowledged unless the specification defines a reply that acknowledges it. |
| `executing` | Work has begun. Expiry no longer bounds it ([Consumer Requirements](#72-consumer-requirements) item 12). | `responded`, `errored`, `cancelled`, `suspended` | — | **Nothing.** In particular, silence here is indistinguishable from `accepted` and from `responded`. |
| `suspended` | A valid, authorized **suspend** has been received and recorded ([Suspension and Resumption](#115-suspension-and-resumption)). | `executing` (on **resume**), `cancelled`, `expired` | The response to the `trust-task-control` document that suspended it. | **Nothing.** A *producer* **MUST NOT** infer suspension, lapse, or resumption from the absence of a notification ([Notifications, and the Meaning of Silence](#116-notifications-and-the-meaning-of-silence)). |
| `responded` | The outcome has been delivered. **Terminal.** | — | `<type>#response`, whose payload is `{}` where the specification defines no success response and the *consumer* acknowledges as a courtesy ([Request and Response Variants](#441-request-and-response-variants)). | **Nothing.** Where no acknowledgement is sent, silence is consistent with this state *and with every state above it*, and **MUST NOT** be read as success. |
| `errored` | The *consumer* has refused, failed, or abandoned the work. **Terminal.** | — | `trust-task-error` ([Error Responses](#8-error-responses)). | **Nothing.** A *consumer* returns an *error response* where the transport permits one; where it cannot, the failure is unobservable to the *producer*. |
| `cancelled` | A valid, authorized **cancel** has taken effect, or the *consumer* stopped on its own initiative. **Terminal** ([When a Control Operation Takes Effect](#113-when-a-control-operation-takes-effect)). | — | The response to the `trust-task-control` document, for a *producer*-requested cancellation; `trust-task-error` carrying `cancelled` where the *consumer* stopped of its own accord ([Standard Error Codes](#83-standard-error-codes)). | **Nothing.** The two directions are distinguished by which reply arrives, never by which one does not. |
| `expired` | `now ≥ expiresAt` was reached before the document was accepted, or before a suspended document was resumed. **Terminal.** | — | `trust-task-error` carrying `expired`, where one can be returned. | **Nothing.** Expiry is evaluated by the *consumer*'s clock and is not announced. |

The re-arrival of a document already in `accepted` or any later state is **not** a state transition: [Consumer Requirements](#72-consumer-requirements) item 11 requires the *consumer* to absorb it, and the *disposition of a duplicate* rule of that section governs what, if anything, it returns. A re-arrival whose content differs is a different document and is refused with `idConflict`.

**Silence signifies no state.** Four sections of this framework have had occasion to say what an absent reply means, and they have not said the same thing: a fire-and-forget task treats it as success, `trust-task-ok` says its absence "carries no information", [Consumer Requirements](#72-consumer-requirements) item 11 uses it to mean a duplicate absorbed, and [Notifications, and the Meaning of Silence](#116-notifications-and-the-meaning-of-silence) says a lost control notification means nothing at all. Only one of those readings is safe, and this table settles on it: **the absence of a reply distinguishes no two states in the table above**, and a *producer* **MUST NOT** infer any state from it. The framework already warns that the silence of item 11 "must not be reused to signify work half-done" ([Consumer Requirements](#72-consumer-requirements)); the general rule is that it may not be reused to signify anything.

The consequence for a fire-and-forget specification is that its *consumer* now has a way to say "received and done" positively rather than by not speaking — the empty `#response` of [Request and Response Variants](#441-request-and-response-variants) — and its *producer* still **MUST NOT** rely on hearing it.

**A transport binding MUST map this table.** A *transport binding* ([Transport Bindings](#9-transport-bindings)) **MUST** state, for its own protocol, which protocol event or status corresponds to each state above, or state explicitly that its protocol expresses no counterpart for a given state. A binding that leaves the mapping implicit invites the substitution this section exists to prevent — an HTTP `202` read as `accepted`, an acknowledged queue delivery read as `executing`, a closed connection read as `cancelled` ([Transport-Level Cancellation Is Not Semantic Cancellation](#117-transport-level-cancellation-is-not-semantic-cancellation)) — each of which reports a document state from a transport fact that does not establish it.

## 5. Versioning

*This section is normative.*

<a id="51-scheme"></a>
### 5.1 Version Scheme

Every *Trust Task specification* **MUST** carry a version of the form `MAJOR.MINOR`, where `MAJOR` and `MINOR` are non-negative decimal integers without leading zeros (except for the value `0` itself). Patch-level versions are not used for *Trust Task specifications*; this framework specification is itself versioned as `MAJOR.MINOR.PATCH`, per [Versioning of This Framework Specification](#511-versioning-of-this-framework-specification). The grammar, in [RFC 5234](https://www.rfc-editor.org/rfc/rfc5234), is:

```abnf
version    = major "." minor
major      = "0" / nonzero *DIGIT
minor      = "0" / nonzero *DIGIT
nonzero    = %x31-39                 ; "1".."9"
```

#### 5.1.1 Versioning of This Framework Specification

This framework specification's `_Version:_` header field states the version that a wider ratifying body confirms when this framework reaches Working Group Approved Deliverable or ToIP Approved Deliverable status; it is currently `1.0` and not yet ratified. The `MAJOR.MINOR.PATCH` number described in the remainder of this section — the one embedded in this framework's *Type URI*, in every *target framework version* declaration, and in the changelog of [Appendix B](#appendix-b-changelog) — is carried instead in the header's `_Document Status:_` field, as `Working Draft MAJOR.MINOR.PATCH`, and is what a *specification* or *consumer* actually targets and resolves against.

This framework specification is itself versioned under [Semantic Versioning 2.0.0](https://semver.org/spec/v2.0.0.html), as `MAJOR.MINOR.PATCH`. A `MAJOR` increment marks a change that is breaking for *consumers* implementing this framework or for the specification-authoring contract of [Specification Requirements](#73-specification-requirements); a `MINOR` increment marks a backwards-compatible addition, such as a new optional document member or a new reserved slug; a `PATCH` increment marks a correction — errata, clarified prose, a repaired example — that changes neither the wire format nor what a conforming *Trust Task specification* must declare. The grammar extends the production above:

```abnf
framework-version = major "." minor "." patch
patch             = "0" / nonzero *DIGIT
```

The three-part form applies to **this document alone**. Every individual *Trust Task specification* carries the two-part `MAJOR.MINOR` version defined above, and that includes the framework-defined specifications published under the reserved `trust-task-` and `trust-ceremony-` prefixes of [Type URI](#61-type-uri) — `trust-task-error/0.4` is a *Trust Task specification* and versions like one. Only the exact reserved slug `trust-task`, which addresses this document, carries a `PATCH` component.

This framework is published in the registry at its full three-part version. Its own *Type URI* is `https://trusttasks.org/spec/trust-task/<MAJOR.MINOR.PATCH>` ([Schema Scope](#63-schema-scope)), and the *target framework version* a specification declares ([Specification Requirements](#73-specification-requirements) item 3) is that same three-part value. A `PATCH` release therefore mints a new framework *Type URI* rather than editing the representation served at an existing one — which is what [Stability](#64-stability) already requires of every other artifact in the registry, and what errata applied in place to a published framework version would otherwise violate.

Publishing a `PATCH` obliges no specification to move. A specification targeting `0.4.0` remains conformant when `0.4.1` is published; re-targeting is an ordinary version bump of the specification itself, sequenced as in [Migrating Between Versions](#54-migrating-between-versions). A *consumer* **MUST** validate the outer document against the framework version its specification targets and **MUST NOT** substitute a different `PATCH`, for the reason given for *shared schema components* in [Shared Schema Components](#66-shared-schema-components) item 2: a pin that resolves to "whatever is latest" lets a later change alter an already-published wire contract silently.

Framework releases published before this rule — `0.1` through `0.4` — denote the releases now written `0.1.0` through `0.4.0`; `0.4` and `0.4.0` are the same release, and the entries in [Appendix B](#appendix-b-changelog) are unchanged by the renumbering. Specifications already published against a two-part *target framework version* remain conformant: a *consumer* **MUST** resolve a two-part value `M.N` as `M.N.0`, and the registry **MUST** continue to serve the two-part framework URIs `https://trusttasks.org/spec/trust-task/0.1` through `/0.4` as aliases of the corresponding `.0` release, so that no already-published specification stops resolving. A specification published or re-issued from this version onward **MUST** declare the three-part form.

#### 5.1.2 Ratification and the Stable Release Line

This subsection governs the transition between this framework specification's own `_Version:_` and `_Document Status:_` header fields as the document moves from Working Draft to a ratified deliverable, and back again for its next major revision. It applies to the *framework document itself*; the `draft`/`candidate`/`standard`/`retired` lifecycle of [Maturity Levels](#53-maturity-levels) governs individual *Trust Task specifications* and *shared schema components* and is unaffected by it.

Before this framework specification's first ratification, `Document Status` **MAY** remain at `MAJOR` `0` even across a breaking change — mirroring the allowance [Compatibility Rules](#52-compatibility-rules) gives an individual `draft`-status artifact — since SemVer 2.0.0 §4 already treats `0.y.z` as carrying no compatibility guarantee. `_Version:_` does not move during this phase; it states the release a wider ratifying body is being asked to confirm, not the draft's own churn.

At the moment a wider body ratifies this framework specification as a Working Group Approved Deliverable or ToIP Approved Deliverable, `Document Status`'s `MAJOR` component crosses from `0` to `1` — not as an arbitrary convention, but because that crossing is what SemVer 2.0.0 §4 defines `1.0.0` to mean: "defines the public API." The header's status label changes from `Working Draft` to the ratified label, and `_Version:_` is confirmed at the matching `MAJOR.MINOR`.

From that point on, `_Version:_` and `Document Status`'s `MAJOR.MINOR` **MUST** carry the same value; they differ only in the status label that prefixes `Document Status`. A `MINOR` increment approved by the ratifying body — a backwards-compatible addition ratified after the initial release — updates both fields together. A `PATCH` increment — errata with no effect on conformance — updates `Document Status` alone; it **MUST NOT** require `_Version:_` to change, since `_Version:_` states what a *consumer* targets and a `PATCH` by definition changes nothing a *consumer* must resolve against.

Work toward the next breaking revision resumes the two-field split: the first breaking change made against a ratified `MAJOR.MINOR` line **MUST** increment `Document Status` to the next `MAJOR.0.0` immediately, with the status label reverting to `Working Draft`, by the ordinary rule of [Compatibility Rules](#52-compatibility-rules) — this is unconditional once ratified, unlike the pre-first-ratification allowance above. `_Version:_`, however, **MUST NOT** advance to that new `MAJOR` until the new line is itself ratified; it continues to state the last-ratified release throughout the new line's drafting, for the same reason it does not move with each Working Draft revision before a first ratification.

### 5.2 Compatibility Rules

A change to a *Trust Task specification* **MUST** be classified as either backwards-compatible or breaking:

* A **backwards-compatible** change — for example, adding an optional member, relaxing a constraint, adding a permitted enumeration value to a non-discriminating field, or clarifying prose — **MUST** result in a `MINOR` increment.
* A **breaking** change — for example, adding or removing a required member, removing a permitted enumeration value, narrowing a constraint, or changing the semantics of an existing member — **MUST** result in a `MAJOR` increment, with `MINOR` reset to `0`.

Implementations of a given *Trust Task specification* at version `M.N` **MUST** accept documents conforming to any version `M.K` where `K ≤ N`.

Forward minor-version compatibility is also intended: because a `MINOR` increment is by definition backwards-compatible, an implementation at `M.N` **SHOULD** accept a document at `M.K` where `K > N`, provided it can ignore any payload members it does not recognize and the document otherwise validates against the framework schema and the `M.N` payload schema known to the implementation. A *consumer* that elects not to support forward minor-version processing **MUST** reject such documents with an `unsupportedVersion` *error response* (see [Standard Error Codes](#83-standard-error-codes)).

A `MAJOR` mismatch is never forward-compatible: a *consumer* at `M.N` **MUST** reject any document whose *Type URI* carries a `MAJOR` segment it does not implement, returning `unsupportedVersion` where the transport permits a response.

*This paragraph is non-normative.* Consumers that implement forward-minor compatibility typically route documents by matching the *Type URI*'s slug and `MAJOR` segment and selecting the highest `MINOR` they implement. A consumer that routes by exact-URI equality (slug + `MAJOR.MINOR`) is conformant — strict matching is permitted by [Compatibility Rules](#52-compatibility-rules) — but precludes the forward-minor SHOULD; downstream implementations choosing strict matching SHOULD document the trade-off.

While a *Trust Task specification* — or a *shared schema component* ([Shared Schema Components](#66-shared-schema-components)) — is at `draft` status ([Maturity Levels](#53-maturity-levels)), its schema and prose **MAY** change without notice. Accordingly, a breaking change to a `draft` artifact **MAY** be released as a `MINOR` increment rather than a `MAJOR` one. Once an artifact reaches `candidate`, `standard`, or `retired`, the classification above applies strictly: every breaking change **MUST** increment `MAJOR`.

A narrower rule applies within `draft`: an **editorial or normalization change** to a `draft` artifact — re-casing an enumerated value or member name into conformance with [Naming Conventions](#410-naming-conventions), re-pinning a `$ref` to a newer framework or *shared schema component* version where the re-pin does not change the payload's effective wire shape, or rewording descriptions and other prose — **MUST** be made in place, errata-style, within the existing version, and **MUST NOT** mint a new version. Such a change carries no semantic difference on the wire; publishing it as a new version inflates the registry, grows the generated libraries, and forces implementations to straddle wire-identical versions for no behavioural gain. At `draft` status this in-place rule takes precedence over the version-coupling rule of [Shared Schema Components](#66-shared-schema-components); from `candidate` onward the classification above applies strictly — a re-cased value, in particular, is a breaking change ([Naming Conventions](#410-naming-conventions)). A version that was nonetheless minted for a purely editorial change **MAY** declare the optional `wireCompatibleWith` front-matter field, naming the wire-identical predecessor version, so that *consumers* can dual-accept documents of the predecessor by mechanical normalization rather than a hand-written adapter.

### 5.3 Maturity Levels

A *Trust Task specification* progresses through a defined lifecycle, captured by its `status` value. The lifecycle is normative: implementations and the registry use `status` to decide whether a specification can change underfoot, whether new documents **SHOULD** be issued against it, and how the bare-URL redirect in [Type URI](#61-type-uri) resolves. The maturity level is independent of the `MAJOR.MINOR` version number.

| Status | Meaning | Schema stability ([Stability](#64-stability)) |
|---|---|---|
| `draft` | Working draft. The schema and prose **MAY** change without notice. | Not stable. |
| `candidate` | Schema is frozen except for editorial clarifications. The specification **MUST** demonstrate two independent, interoperable implementations to enter this status. | Stable. |
| `standard` | Stable in the long term. A `candidate` specification **MUST** complete a continuous 90-day stability window with no breaking changes before promotion to `standard`. | Stable. |
| `retired` | Specification is no longer recommended for new use; preserved for historical reference and to keep already-issued documents verifiable. The schema and prose are frozen at the moment of retirement. | Stable. |

<a id="permitted-transitions"></a>
#### 5.3.1 Permitted Transitions

A `status` value **MUST** change only along one of the transitions below:

1. `draft` → `candidate` — once the entry criteria for `candidate` are met.
2. `candidate` → `standard` — once the 90-day stability window has elapsed without breaking changes.
3. `draft` → `retired` — abandoning a working draft.
4. `candidate` → `retired` — deprecating a candidate before standardization.
5. `standard` → `retired` — sunsetting a standard after a successor has been published.

`retired` is **terminal**: a retired specification **MUST NOT** transition back to any earlier status. To revive functionality, the editor publishes a new `MAJOR.MINOR` of the slug starting at `draft` (see [Version Scheme](#51-version-scheme)).

<a id="behavior-at-each-status"></a>
#### 5.3.2 Behaviour at Each Status

* **Producers MAY** emit documents whose `type` resolves to a `draft`, `candidate`, or `standard` specification. Producers **SHOULD NOT** emit documents against a `retired` specification.
* **Consumers MUST** treat `draft`, `candidate`, `standard`, and `retired` specifications identically for the purpose of schema validation (the framework rules in [Consumer Requirements](#72-consumer-requirements) apply uniformly). Consumers **SHOULD** surface a deprecation signal — in logs, audit records, or downstream interfaces — when a received document's `type` resolves to a `retired` specification, so operators can plan migration.
* A `retired` specification **SHOULD** declare its successor via the optional `supersededBy` front-matter field (see [Specification Requirements](#73-specification-requirements)).

A specification's current status is recorded in its front matter and reflected in the registry at <https://trusttasks.org/>. The same lifecycle applies to this framework specification itself.

The process by which a slug is assigned, by which a specification enters the registry, and by which its status is updated is governed by the registry policy maintained alongside the registry at <https://trusttasks.org/>. That policy is out of scope for this framework specification.

### 5.4 Migrating Between Versions

*This section is informative.*

When a *Trust Task specification* makes a breaking change — including adopting a new version of a *shared schema component* ([Shared Schema Components](#66-shared-schema-components)) — implementers are encouraged to migrate using an expand-then-contract sequence that keeps both versions interoperable throughout, so that no single deployment step requires producers and consumers to change in lockstep:

1. **Author the new version.** Publish the new specification version — `M.(N+1)` for a backwards-compatible change, or `(M+1).0` for a breaking change at non-`draft` status (a breaking change at `draft` MAY use a `MINOR` increment per [Compatibility Rules](#52-compatibility-rules)). If the change is driven by a shared schema, publish the new shared schema component version first and re-pin the specification's `$ref` to it (see the coupling rule below). The previous specification version remains published and unchanged.

2. **Update receivers first.** Deploy *consumer* support for the new version alongside the old, so a *consumer* accepts documents of both the old and the new version. Because no *producer* is emitting the new version yet, this step is safe to roll out on its own. For a `MINOR` increment, a forward-minor-compatible consumer ([Compatibility Rules](#52-compatibility-rules)) may already accept the new version with no code change; for a `MAJOR` increment the consumer **must** add explicit support before any producer emits it.

3. **Update senders.** Once receivers across the deployment accept the new version, deploy *producer* support so producers begin emitting the new version. Traffic shifts to the new version while consumers continue to accept any stragglers still on the old one.

4. **Retire the old version.** After traffic has fully migrated and any applicable stability window has elapsed, transition the old version to `retired` ([Maturity Levels](#53-maturity-levels)) and declare `supersededBy` pointing at the new version. Producers stop emitting the old version; consumers may drop support for it on their own schedule.

**Coupling of schema and specification versions.** A *Trust Task specification* and its payload JSON Schema are a single versioned artifact — the schema's `$id` is the specification's *Type URI* ([Schema Scope](#63-schema-scope)) — so any change to the payload schema is, by definition, a new specification version. A *shared schema component* ([Shared Schema Components](#66-shared-schema-components)) versions independently, but a specification **cannot adopt a new shared schema component version without issuing a new version of itself**: re-pinning a `$ref` changes the specification's effective wire contract. A specification **MAY** instead remain pinned to the older component version and not bump.

## 6. Namespace

*This section is normative.*

The framework defines a single resolvable namespace per versioned *Trust Task specification*. One canonical URL serves human-readable prose, machine-readable schemas, and (where defined) JSON-LD contexts, differentiated by HTTP content negotiation.

### 6.1 Type URI

Every versioned *Trust Task specification* **MUST** be addressable by a *Type URI* — a URI in the sense of [RFC 3986](https://www.rfc-editor.org/rfc/rfc3986) — of the form:

```
https://trusttasks.org/spec/<slug>/<MAJOR.MINOR>
```

The form above is the canonical, public-registry form. *Trust Task specifications* intended only for private or internal use — and not published through the public registry — **MAY** use a different authority under the same URI shape; the requirements that apply to those are given in [Private and Unpublished Trust Task Specifications](#65-private-and-unpublished-trust-task-specifications).

For both forms, the path components below carry identical meaning:

* The URI scheme **MUST** be `https`. Other schemes (including `http`) are non-conformant: every representation served at a *Type URI* depends on transport-layer authentication and integrity, and permitting `http` would normalize a transport-downgrade path for any *consumer* that dereferences the URI.
* `<slug>` is a lowercase, hyphen-separated short name assigned to the specification, optionally organized into one or more path segments (e.g. `trust-task-discovery`, or `acl/grant`). The slug **MUST** match the regular expression `^[a-z][a-z0-9]*(-[a-z0-9]+)*(/[a-z][a-z0-9]*(-[a-z0-9]+)*)*$`. Each `/`-delimited segment **MUST** individually satisfy the single-segment grammar (`^[a-z][a-z0-9]*(-[a-z0-9]+)*$`); consecutive hyphens are not permitted within a segment, and consecutive slashes are not permitted between segments. Segments group related specifications under a shared namespace and are reflected in the *Type URI* path verbatim — `https://trusttasks.org/spec/acl/grant/0.1` is the *Type URI* of a specification whose slug is `acl/grant`.
* `<MAJOR.MINOR>` is the specification version as defined in [Version Scheme](#51-version-scheme). The sole exception is the reserved slug `trust-task`, which addresses this framework specification and carries a three-part `<MAJOR.MINOR.PATCH>` segment instead ([Versioning of This Framework Specification](#511-versioning-of-this-framework-specification)); every other slug, the framework-defined ones included, carries the two-part form. When resolving a *Type URI*, a *consumer* identifies the version as the final path segment (which always matches one of the two version grammars) and the slug as the segments between `/spec/` and the version.

A *Type URI* used as the value of a *Trust Task document*'s `type` member **MAY** additionally carry the fragment `#request` or `#response`, with the meanings defined in [Request and Response Variants](#441-request-and-response-variants). The fragments `#request` and `#response` are **RESERVED**; no other fragment values are defined by this framework, and individual *Trust Task specifications* **MUST NOT** define their own.

The following slugs are **RESERVED** for framework-defined specifications and **MUST NOT** be used by any individual *Trust Task specification*:

* The exact slug `trust-task`, reserved for this framework specification itself. It is the one slug whose version segment is three-part.
* Any slug whose first segment is `trust-task` or `trust-ceremony`, or begins with the prefix `trust-task-` or `trust-ceremony-`, reserved for framework-defined specifications. Equivalently, the slug **MUST NOT** match the pattern `^trust-(task|ceremony)($|-|/)`. The `trust-ceremony` half of the reservation is unused at this version and exists so that the ceremony layer of [The `ceremony` Member](#411-the-ceremony-member) has a namespace to publish into that no other party can claim first. The slugs currently published by the framework under this reservation are:

  | Slug                     | Purpose                                                                 |
  |--------------------------|-------------------------------------------------------------------------|
  | `trust-task-error`       | Error-response payload — see [The trust-task-error Specification](#81-the-trust-task-error-specification). |
  | `trust-task-ok`          | Courtesy acknowledgement of a task that defines no success response — **deprecated at 0.5.0**, see [Reserved Response-Type Slugs](#86-reserved-response-type-slugs). The slug remains reserved permanently. |
  | `trust-task-next-step`   | Recipient-suggested continuation — see [Reserved Response-Type Slugs](#86-reserved-response-type-slugs). |
  | `trust-task-discovery`   | Discovery and capability negotiation — see [Discovery and Capability Negotiation](#10-discovery-and-capability-negotiation). |
  | `trust-task-control`     | Cancellation, suspension, and resumption of an accepted task — see [Task Control](#11-task-control). |
  | `trust-ceremony-receipt` | Evidence that one *enactment* of a *Trust Ceremony* completed — see [The `ceremony` Member](#411-the-ceremony-member). |

The *Type URI* is the single canonical, resolvable reference to a versioned *Trust Task specification*. It serves both humans (rendered prose) and machines (validation schema, optional JSON-LD context) under content negotiation as defined in [Content Negotiation](#62-content-negotiation).

The framework also reserves a parallel `/binding/` subtree under the same authority for *transport binding* identifiers and binding-internal resources (envelope `type` values, binding schema URIs, status mappings). The `/binding/` subtree is **structurally disjoint** from `/spec/`: no URI under `/binding/` is a *Type URI*, and a *Trust Task document* whose `type` is rooted at `/binding/...` is malformed. The grammar and rules for the `/binding/` subtree are defined in [Binding Namespace](#93-binding-namespace).

A third subtree, `/ceremony/`, is reserved for *ceremony definitions* on the same terms. It is likewise **structurally disjoint** from both: no URI under `/ceremony/` is a *Type URI*, and a *Trust Task document* whose `type` is rooted at `/ceremony/...` is malformed. The grammar and rules for the `/ceremony/` subtree are defined in [Ceremony Namespace](#67-ceremony-namespace).

A *Type URI* with the version segment omitted (i.e. `https://trusttasks.org/spec/<slug>`) **SHOULD** redirect to the latest `standard` version of the specification, or — if no `standard` version exists — to the latest `candidate`, or — failing that — to the latest `draft`. `retired` versions **MUST NOT** be selected by the bare-URL redirect, since `retired` signals "no longer recommended for new use"; if every version of a slug is `retired`, the bare URL **SHOULD** return `410 Gone` with a body that links to the latest retired version and its declared `supersededBy` successor, if any.

### 6.2 Content Negotiation

A server hosting a *Type URI* **MUST** support HTTP content negotiation [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110) and **MUST** be capable of returning the representations listed below. The server **MUST** return the representation matching the highest-priority acceptable media type in the request's `Accept` header. If the `Accept` header is absent or names no representation listed below, the server **MUST** return the `text/html` representation.

| Media type | Representation |
|---|---|
| `text/html` | The rendered specification document for human readers. |
| `application/schema+json` | The normative JSON Schema [JSON Schema 2020-12](https://json-schema.org/draft/2020-12/schema) for this specification (see [Schema Scope](#63-schema-scope) for what the schema covers). |
| `application/ld+json` | The JSON-LD context for this specification, when one is defined. If no context is published for this *Type URI*, the server **MUST** respond with HTTP `406 Not Acceptable`. |

Every representation returned **MUST** describe the same version of the specification as is encoded in the requested *Type URI*.

### 6.3 Schema Scope

The JSON Schema served at the *Type URI* of an individual *Trust Task specification* describes **only** the contents of that specification's `payload` member.

The outer document structure (`id`, `threadId`, `parentThreadId`, `type`, `issuer`, `recipient`, `issuedAt`, `expiresAt`, `payload`, `@context`, `proof`) is described by the JSON Schema served at the framework's own *Type URI* — `https://trusttasks.org/spec/trust-task/<MAJOR.MINOR.PATCH>`, three-part per [Versioning of This Framework Specification](#511-versioning-of-this-framework-specification) — under content negotiation for `application/schema+json`. A complete document validation therefore composes the framework schema (outer structure) with the task-specific payload schema.

The JSON Schema served at any *Type URI* **MUST** declare:

* `$id` equal to that *Type URI*.
* `$schema` set to `https://json-schema.org/draft/2020-12/schema`.

It **MUST** specify `additionalProperties` either explicitly as `false` or with an accompanying prose statement of how unrecognized members are to be treated.

### 6.4 Stability

For any value of `<slug>` and any value of the version segment, the representations served at the corresponding *Type URI* **MUST NOT** change in a way that alters their normative content once the specification has reached the `candidate`, `standard`, or `retired` status. This applies to the framework's own *Type URI* on the same terms, which is why a framework `PATCH` release is published at a new URI rather than applied in place. Once a specification is `retired`, the schema and prose are frozen at the moment of retirement; the only permitted change is correcting the `status` value itself (which is itself terminal — see [Maturity Levels](#53-maturity-levels)) or adding the `supersededBy` declaration.

This commitment is made by the public registry for *Trust Task specifications* it hosts; private specifications published under their own authority (see [Private and Unpublished Trust Task Specifications](#65-private-and-unpublished-trust-task-specifications)) **SHOULD** offer their consumers an equivalent commitment, scoped to their own trust boundary.

### 6.5 Private and Unpublished Trust Task Specifications

Not every *Trust Task specification* is intended for the public registry. A *producer* and *consumer* operating within a single organization, deployment, or trust boundary **MAY** define their own *Trust Task specifications* solely for internal use — never publishing them under `https://trusttasks.org/` — and still conform fully to this framework.

The following rules apply to *Trust Task specifications* that are not published through the public registry:

1. **Authority.** A private specification's *Type URI* **MUST NOT** be served from, or claim to identify a resource at, the `https://trusttasks.org/` domain. That domain is reserved for *Trust Task specifications* published through the public registry process. A private specification **SHOULD** use an HTTPS authority the publisher controls — typically a project or organization domain — so the URI uniquely identifies the specification within the publisher's trust boundary. Examples:
   ```
   https://example.com/trust-tasks/<slug>/<MAJOR.MINOR>
   https://internal.example/spec/<slug>/<MAJOR.MINOR>
   ```
   The slug grammar, version grammar, fragment conventions, and path-component meanings defined in [Type URI](#61-type-uri) apply unchanged.

2. **Reservation rule.** The slug reservation rule in [Type URI](#61-type-uri) — that the slug **MUST NOT** be `trust-task` or have a first segment matching `^trust-task(-|/)?` — applies regardless of authority. A private specification **MUST NOT** use those reserved slugs even on its own domain, so that documents flowing between trust boundaries cannot be confused with framework-defined response types.

3. **Framework conformance is unchanged.** All other framework requirements — the document structure ([Trust Task Documents](#4-trust-task-documents)), versioning rules ([Versioning](#5-versioning)), conformance behaviour ([Minimum Requirements](#7-minimum-requirements)), and error response shape ([Error Responses](#8-error-responses)) — apply identically to private *Trust Task specifications*. Implementations consuming both private and registry-published specifications **SHOULD** use the same validation and signing pipeline for both.

4. **Resolvability.** A private *Type URI* **SHOULD** resolve to the specification's representations under content negotiation ([Content Negotiation](#62-content-negotiation)) for parties within the publisher's trust boundary, but **MAY** be unresolvable from the public internet. A *consumer* unable to dereference a private *Type URI* relies on out-of-band distribution of the specification document and schema.

5. **Promotion to the registry (informative).** A private *Trust Task specification* **MAY** later be submitted for inclusion in the public registry. The submission process is governed by the registry policy referenced in [Maturity Levels](#53-maturity-levels); a re-host typically involves a slug check, transfer of the JSON Schema document, and publication under `https://trusttasks.org/spec/<slug>/<MAJOR.MINOR>`. The original private *Type URI* and the new public *Type URI* identify distinct specifications unless and until the registry policy explicitly aliases them.

Private *Trust Task specifications* are full *Trust Task specifications* for the purposes of conformance: a producer or consumer that satisfies [Minimum Requirements](#7-minimum-requirements) against a private spec is a *conforming producer* or *conforming consumer* of that spec, exactly as it would be for a registry-published one.

### 6.6 Shared Schema Components

A *Trust Task specification*'s payload JSON Schema **MAY** reference reusable schema fragments — *shared schema components* — that several specifications have in common (for example, an access-control entry, a device binding, a sealed-envelope shape, or a synchronization event). Shared schema components are an authoring convenience and a consistency mechanism. They are **not** independently published *Type URIs*: a *Trust Task document*'s `type` **MUST NOT** resolve to a shared schema component, and a shared schema component is never the unit a document conforms to — only a specification is.

A shared schema component is nonetheless a *versioned artifact* in its own right, governed by the following rules.

1. **Versioning.** A shared schema component carries a `MAJOR.MINOR` version and follows the same compatibility rules as a specification ([Compatibility Rules](#52-compatibility-rules)), including the `draft` caveat. A breaking change to a shared schema component — re-casing an enumerated value, removing or renaming a `$def`, narrowing a constraint — **MUST** be published as a new version of that component. The prior version remains in place for the specifications still pinned to it.

2. **Pinning.** A consuming specification **MUST** reference a shared schema component at a specific version. Resolving a reference to "the latest" version of a component is non-conformant, because a later change to the component would otherwise alter an already-published specification's wire contract silently.

3. **Coupling to specification versions.** Because a consuming specification pins a specific component version, adopting a new component version requires the consuming specification to publish a new version of itself (see [Migrating Between Versions](#54-migrating-between-versions)). A component version bump therefore never changes an already-published specification underfoot; the new component values become observable only through a new specification version that re-pins to them.

4. **Lifecycle and discovery.** A shared schema component **SHOULD** declare its own `status` ([Maturity Levels](#53-maturity-levels)) and **MAY** declare `supersededBy` when retired. The registry **SHOULD** surface shared schema components and their versions alongside specifications, so that implementers can see which specification versions depend on which component versions.

### 6.7 Ceremony Namespace

A *ceremony definition* — the published description of a flow composed of several *Trust Tasks*, referenced by the `ceremony.definition` member of [The `ceremony` Member](#411-the-ceremony-member) — is identified by a URI in the `/ceremony/` subtree of the framework's authority:

```
https://trusttasks.org/ceremony/<slug>/<MAJOR.MINOR>
```

`<slug>` follows the same lowercase, hyphenated grammar as a Trust Task slug ([Type URI](#61-type-uri)) and is subject to the same `^trust-(task|ceremony)($|-|/)` reservation; `<MAJOR.MINOR>` follows the version grammar of [Version Scheme](#51-version-scheme).

The `/ceremony/` subtree is **structurally disjoint** from `/spec/` and `/binding/`. A *ceremony definition* is not a *Trust Task specification*: no document's `type` resolves to one, and a *consumer* that receives a *Trust Task document* whose `type` is rooted at `/ceremony/...` **MUST** reject it with `malformedRequest` ([Standard Error Codes](#83-standard-error-codes)). The Type URI grammar of [Type URI](#61-type-uri) already excludes the path; the rule is stated explicitly so the namespace boundary is visible at a glance and so such documents have a defined disposition rather than relying on grammar mismatch.

A *ceremony definition* is referenced by content as well as by name: a step carrying `ceremony.definition` **MUST** also carry `ceremony.definitionDigest` ([The `ceremony` Member](#411-the-ceremony-member)). A URI alone would leave the flow's rules mutable by whoever controls the URI, retroactively and for every enactment already performed.

This version of the framework defines the namespace, the reservation, and the reference mechanism. The **content** of a ceremony definition — its role, step, ordering and completion vocabulary — is out of scope for this revision and is expected to be specified in a future one. A *consumer* encountering a `ceremony.definition` it cannot resolve or does not understand **MAY** process the document as though the member were absent; by [Membership Is a Claim, Not a Permission](#4114-membership-is-a-claim-not-a-permission) it forgoes no authority in doing so.

The reservation rule of [Private and Unpublished Trust Task Specifications](#65-private-and-unpublished-trust-task-specifications) applies equivalently: a private ceremony definition **MUST** use an authority its publisher controls and **MUST NOT** claim to identify a resource at `https://trusttasks.org/ceremony/...`.

## 7. Minimum Requirements

*This section is normative.*

### 7.1 Producer Requirements

A *conforming producer* **MUST**:

1. Emit a *Trust Task document* whose top-level structure satisfies [Top-Level Members](#42-top-level-members).
2. Set the `type` member to the *Type URI* of the *Trust Task specification* being implemented, including its `<MAJOR.MINOR>` segment.
3. Place all task-specific data in `payload`, and emit a `payload` value that validates against the JSON Schema obtained by content-negotiating the *Type URI* for `application/schema+json` (see [Content Negotiation](#62-content-negotiation)).
4. Populate `id` with a value satisfying [The `id` Member](#43-the-id-member).

A *conforming producer* **SHOULD** populate `issuedAt` to support freshness checks downstream, **SHOULD** populate `issuer` and `recipient` when the transport in use does not provide authenticated party identity end-to-end between *producer* and *consumer*, **SHOULD** set `threadId` when emitting a *Trust Task document* in response to another (see [The `threadId` Member](#49-the-threadid-member)), **SHOULD** set `parentThreadId` when the exchange is conducted inside another and carry it onto every document of the inner exchange (see [The `parentThreadId` Member](#492-the-parentthreadid-member)), **SHOULD** set `ceremony` when the document is a step of a *Trust Ceremony* and carry the same `enactment` onto every document of that step including any *error response* (see [The `ceremony` Member](#411-the-ceremony-member)), and **SHOULD** preserve any unrecognized members received from upstream parties when forwarding a *Trust Task document*.

A *conforming producer* that emits an `ext` member (see [The `ext` Extension Member](#451-the-ext-extension-member)) **MUST** namespace every immediate child key of `ext` under a reverse-DNS prefix the producer controls; bare or un-namespaced child keys are non-conforming.

### 7.2 Consumer Requirements

A *conforming consumer* **MUST**:

1. Validate the outer document structure against the framework JSON Schema. The applicable framework version is the *target framework version* declared by the *Trust Task specification* identified by the document's `type` member (see [Specification Requirements](#73-specification-requirements)). The framework schema for that version is obtained by content-negotiating `https://trusttasks.org/spec/trust-task/<MAJOR.MINOR.PATCH>` for `application/schema+json`, where `<MAJOR.MINOR.PATCH>` is the declared target framework version — **not** the `<MAJOR.MINOR>` of the document's `type` member, which identifies the task specification version, is two-part, and may differ.
2. Validate the document's `payload` member against the JSON Schema obtained by content-negotiating the document's `type` member for `application/schema+json`.
3. Reject any document whose `type` it does not recognize, unless the consumer's policy explicitly permits forward-compatible processing under [Compatibility Rules](#52-compatibility-rules).
4. Honor the document's expiry where present: if `expiresAt` is set and `now ≥ expiresAt` relative to the *consumer*'s clock (with the optional skew tolerance permitted in [Top-Level Members](#42-top-level-members)), treat the document as expired and not act upon it. This is an **acceptance** bound: it governs whether the *consumer* may begin, and does not by itself require it to abandon execution already under way (see item 12).
5. Reject any document whose `recipient` member is set and does not identify the *consumer*'s own party. Where the *Trust Task specification* declares `recipient` as **REQUIRED** (see [Specification Requirements](#73-specification-requirements) item 5), reject any document lacking an in-band `recipient` with `malformedRequest`.
6. Reject any document for which an in-band `issuer` or `recipient` member is inconsistent with an authenticated identity derived from the transport for the same party.
7. If the document carries a `proof` member, verify it per [Proof](#47-proof) against the in-band `issuer` and reject the document with `proofInvalid` on verification failure. Independently, if the *Trust Task specification* identified by `type` declares `proof` as **REQUIRED** (see [Specification Requirements](#73-specification-requirements) item 8) and no `proof` is present, reject the document with `proofRequired`.
8. If the document carries a `proof` member and no in-band `recipient`, and the *Trust Task specification* identified by `type` is **not** a *bearer specification* ([Bearer Specifications](#483-bearer-specifications)), reject the document with `malformedRequest`. This enforces the [Audience Binding](#482-audience-binding) rule.
9. Not grant any authority on the basis of a `ceremony` member. Membership of an *enactment* is an assertion by the document's *issuer*, not a verified fact, and every authorization decision **MUST** be reached under item 10 below exactly as for a document carrying no such member. See [Membership Is a Claim, Not a Permission](#4114-membership-is-a-claim-not-a-permission). A *consumer* that does not implement ceremonies applies the unrecognized-member rule below and forgoes nothing by doing so.
10. Not treat identity or document-proof validation as authorization. Successful validation of a *VID*, `issuer`, `recipient`, transport-derived identity, or `proof` establishes **who** made the assertion and that the document reached the *consumer* unaltered. It **MUST NOT**, by itself, be treated as establishing that the *producer* is authorized to request the outcome the *Trust Task* describes, or that the *consumer* is authorized to perform it. Before executing a *Trust Task*, a *consumer* **MUST** evaluate whatever authorization requirements apply under the *Trust Task specification* identified by the document's `type`, the *consumer*'s own policy, and the trust or governance framework it operates under.
11. Not execute a *consequential Trust Task* ([Terminology](#3-terminology)) twice on account of the same *Trust Task document*. Once a *consumer* has accepted a document with a given `id` for execution, receipt of that same document again **MUST NOT** cause the consequential effect to occur a second time, unless the *Trust Task specification* identified by the document's `type` explicitly declares repeated execution safe and intended. A *consumer* receiving a document whose `id` matches one it has already accepted but whose content differs **MUST** reject the later document with `idConflict` ([Standard Error Codes](#83-standard-error-codes)) and **MUST NOT** treat it as a retry of the original. Transport request identifiers, transport message identifiers, and execution handles **MUST NOT** substitute for the *Trust Task document*'s `id` as the key for this rule.
12. Re-evaluate, immediately before each irreversible or externally visible effect of a *consequential Trust Task* ([Terminology](#3-terminology)), every condition that the *Trust Task specification* and the *consumer*'s own policy require for that effect. Successful validation establishes that a document was eligible for processing **when it was validated**; it does not establish that the work remains executable indefinitely, and for execution that is delayed, long-running, or resumed the two instants can be far apart. Where a required condition — an authorization, delegation, mandate, capability, membership, standing, credential or key status, subject relationship, or a deadline the *Trust Task specification* defines for itself — is no longer satisfied at that point, the *consumer* **MUST NOT** perform the subsequent effect. A valid, authorized control operation the *consumer* has received under [Task Control](#11-task-control) is such a condition.
13. Reject a document whose timestamps place it outside the window in which the *consumer* is willing to act. Specifically, a *consumer* **MUST** reject with `malformedRequest` ([Standard Error Codes](#83-standard-error-codes)):
    1. a document whose `issuedAt` is later than the *consumer*'s own clock by more than the clock-skew tolerance it applies under [Top-Level Members](#42-top-level-members); and
    2. a document whose `expiresAt` is at or before its `issuedAt`.

    Both are refused as **malformed** rather than as `expired`: `expired` names a document that was once acceptable and no longer is, and neither of these ever was. A future-dated document asserts a production instant the *consumer* has not reached, and a document whose validity ends at or before it began is unacceptable at every instant, including the one it was minted in — a *consumer* that returned `expired` for it would be telling the *producer* to wait, when what the *producer* must do is reissue.

For each of the rules in this section that references the `issuer` or `recipient` party, the in-band member value is authoritative when present and the transport-derived identity is a cross-check; when the in-band member is absent the *consumer* **MAY** derive the value from the transport. This precedence is defined normatively in [Precedence of In-Band over Transport-Derived Identity](#481-precedence-of-in-band-over-transport-derived-identity).

The evaluation required by item 10 **MAY** consider delegation, mandate, capability, membership, standing, credential status, subject relationship, purpose limitation, or any other evidence the *consumer* requires; this framework does not prescribe an authorization model and does not constrain which of these a *consumer* consults. A verified assertion **MAY** itself constitute authorization evidence, but only where the *Trust Task specification* explicitly defines that role for it ([Specification Requirements](#73-specification-requirements) item 15) **and** the *consumer*'s policy accepts it for that purpose — a signed decision whose proof *is* the authorization is a design a specification may adopt deliberately, not a default a *consumer* may infer. Where authorization fails after the document has otherwise validated, the *consumer* **SHOULD** return an *error response* of `permissionDenied` ([Standard Error Codes](#83-standard-error-codes)) where one can be returned safely, subject to the message-content rule of [Error-Response Identity Leakage](#124-error-response-identity-leakage).

**Keying and comparison for item 11.** The duplicate-execution key is the *Trust Task document*'s `id` alone, which *producers* are required to mint globally unique and never to reuse ([The `id` Member](#43-the-id-member)). Two documents bearing the same `id` are *the same document* for the purposes of item 11 when their serializations are identical under [RFC 8785](https://www.rfc-editor.org/rfc/rfc8785) canonicalization — the same identity [Retry Semantics](#84-retry-semantics) defines for a retry. Any other difference, including a changed `payload`, a changed `recipient`, or a re-signed `proof` over identical content, makes them different documents sharing an `id`, which is the `idConflict` case. A *consumer* implementing item 11 therefore retains a digest of what it accepted, not merely the `id`: an `id` alone cannot distinguish the retry it must absorb from the conflict it must reject.

**Bounding the record.** A *consumer* cannot apply item 11 to a document it can no longer recognize, so it **MUST** retain the record for at least as long as it remains willing to execute that document. The two bounds are the same bound. Where `expiresAt` is present it fixes both: after it, the document is refused under item 4 and the record may be dropped. Where `expiresAt` is absent, the *consumer*'s own acceptance window — typically a freshness bound over `issuedAt` — fixes both, and a *consumer* **MUST NOT** accept for execution a document older than the window over which it retains records. Retention beyond that point is not required: a document the *consumer* would now refuse as stale cannot be replayed into a second execution. A *consumer* that can establish neither an `expiresAt` nor an age for a document has no window in which to place it, and **MUST NOT** execute a *consequential Trust Task* on it.

**Why item 13 exists.** Item 11 is only implementable inside a bounded window, and the two timestamp members are what bound it. Without a freshness bound the *consumer* must retain its record of every accepted document **forever** — because a document it can no longer recognize is a document it will execute a second time, and a document with no `expiresAt` and no usable `issuedAt` never leaves the set it must recognize. Item 13 removes the two ways a document could otherwise sit outside any window while still appearing acceptable: an `issuedAt` in the *consumer*'s future, which places the document in a window that has not opened and would keep re-entering it as the clock advances; and an `expiresAt` at or before the `issuedAt`, which describes a validity interval that never contained an instant and so answers item 4 differently depending only on which member the *consumer* happened to consult. Neither refusal is a judgement about the *producer*'s intent — a skewed clock produces the first case routinely — and both are cheap: they are decided from the document alone, before any resolution, verification, or execution work.

The obligation on the *producer* side is [Specification Requirements](#73-specification-requirements) item 17: a *Trust Task specification* defining a *consequential Trust Task* **MUST** require `issuedAt`, so that a *consumer* of the tasks for which item 11 matters most always has a value to place the document by.

**What item 12 does and does not re-check.** The rule is about **authority**, not about the clock. `expiresAt` is deliberately absent from its list: that member bounds acceptance ([Top-Level Members](#42-top-level-members)), and re-checking it mid-execution would convert a statement about a request's staleness into an execution timeout the *producer* never set and could not have calculated — it does not know how long the *consumer*'s work takes. A *consumer* **MUST NOT** abandon execution solely because `expiresAt` has passed since it began.

A *Trust Task* that genuinely has a completion deadline — an offer that lapses, a quote that cannot be honored after a stated instant — expresses it in its own `payload`, where the *Trust Task specification* can define what lapsing *means* for that task. Such a deadline is then one of the conditions item 12 re-evaluates, on the same footing as a revoked delegation, with no framework member required. `task-consent/request/0.1` is the worked example: its `payload.expiresAt` states that the pending request lapses and no decision is accepted for it — a meaning the envelope member could not carry, because the envelope does not know what a decision is.

**Stopping is not always safer than finishing.** Item 12 is placed *before* each irreversible effect for a reason: once such an effect has occurred it cannot be undone by declining the next one, and for many *consequential Trust Tasks* abandoning a partially applied change leaves the *recipient party* in a state neither party asked for. A *consumer* **MUST NOT** treat abandonment as inherently the safe option; where a *Trust Task specification* defines a sequence whose partial application is unsafe, the specification says so and the *consumer* follows it.

Where execution has already produced partial or irreversible effects and the *consumer* stops, it **SHOULD** return a response or status that distinguishes **partial execution** from a task that was never begun. The two are different facts about the world, and a *producer* that cannot tell them apart cannot decide whether to reissue. Where the *Trust Task specification* defines no success-response document, this disposition is reported as an *error response* ([Error Responses](#8-error-responses)) rather than left silent. Silence would report nothing at all ([Document Lifecycle](#412-document-lifecycle)) and would in particular be indistinguishable from the duplicate absorbed under item 11 — two facts about the world that oblige a *producer* to do opposite things.

**Disposition of a duplicate.** Where the original execution is still in progress, the *consumer* **SHOULD** return or expose the existing execution state rather than begin another. Where execution has completed and the *Trust Task specification* defines a success-response document ([Specification Requirements](#73-specification-requirements) item 7.6), the *consumer* **SHOULD** return the previously determined result, or an equivalent receipt where that specification permits one. Where the specification defines **no** success-response document — the fire-and-forget case of that same item — there is nothing to return beyond the courtesy acknowledgement of [Acknowledging a Fire-and-Forget Task](#442-acknowledging-a-fire-and-forget-task), which a *consumer* **MAY** return on a duplicate exactly as on the original: the *consumer* declines to execute again, and neither the acknowledgement nor its absence is an error. In no case is a duplicate reported as `taskFailed`; the task did not fail, it already happened.

**Relationship to idempotency.** Idempotency as a property of the underlying operation remains task-specific and outside this framework. Item 11 requires only that transport retry or replay not invoke a consequential operation a second time. A *Trust Task specification* whose operation is naturally idempotent — where executing twice is indistinguishable from executing once, in every effect the *recipient party* exposes — **MAY** declare repeated execution safe and intended, which disapplies item 11 for that specification. Such a declaration is about the operation, not about the *consumer*'s convenience, and a specification **MUST NOT** make it merely to avoid implementing the rule.

*This paragraph is non-normative.* Item 10 generalizes a principle the framework already applies in two narrower places: [Membership Is a Claim, Not a Permission](#4114-membership-is-a-claim-not-a-permission) and item 9, where ceremony membership authorizes nothing, and [Specification Requirements](#73-specification-requirements) items 13 and 14, where the side-effect and exposure classes describe a task without authorizing it. Those are instances of the general rule rather than exceptions to it. The inference the rule forecloses — *valid `proof` + recognized `issuer` + correct `recipient` = authorized instruction* — is the confused-deputy vector of [Membership Is a Claim, Not a Permission](#4114-membership-is-a-claim-not-a-permission) reached by a different route, and it is most dangerous where the *producer* is an autonomous agent: such a *producer* can typically prove its own identity perfectly while holding no authority to act for a subject, exercise a delegated capability, disclose information, or cause a *consequential* effect ([Terminology](#3-terminology)).

A *conforming consumer* **SHOULD** preserve, but **MUST NOT** act upon, members it does not recognize. A *consumer* that does not implement JSON-LD processing **MUST** ignore the `@context` member.

For documents that carry an `ext` member (see [The `ext` Extension Member](#451-the-ext-extension-member)), a *conforming consumer* **MUST** ignore every `ext` immediate-key namespace it does not recognize — the unrecognized-namespace rule is the same "preserve but MUST NOT act upon" rule as for unrecognized top-level members, applied at the `ext` level. A *consumer* **MAY** require one or more specific namespaces under `ext` as a matter of local policy and **MUST** reject a document missing a required namespace with `malformedRequest`; *consumers* applying such a policy **SHOULD** publish the requirement via discovery ([Discovery and Capability Negotiation](#10-discovery-and-capability-negotiation)) so *producers* can satisfy it before the wire trip.

When a *consumer* rejects a *Trust Task document* under any rule in this section, and the transport in use supports a response from *consumer* to *producer*, the *consumer* **SHOULD** return an *error response* conforming to [Error Responses](#8-error-responses).

### 7.3 Specification Requirements

A *conforming Trust Task specification* **MUST** declare each of the following. Together these declarations make the specification self-describing to both human readers and machine consumers, and constitute the minimum needed to author or interpret a *Trust Task document*.

1. **Slug** — the lowercase slug used in the specification's *Type URI*, satisfying the grammar and reservation rules of [Type URI](#61-type-uri).
2. **Version** — the `MAJOR.MINOR` version of this specification, satisfying [Version Scheme](#51-version-scheme).
3. **Target framework version** — the `MAJOR.MINOR.PATCH` version of this framework specification that the *Trust Task specification* targets, three-part per [Versioning of This Framework Specification](#511-versioning-of-this-framework-specification). A value declared before that rule took effect is two-part and resolves as `M.N.0`. A *consumer* uses this declared value to select the framework schema under which the outer document is validated (see [Consumer Requirements](#72-consumer-requirements), item 1).
4. **Maturity level** — one of `draft`, `candidate`, `standard`, or `retired`, satisfying [Maturity Levels](#53-maturity-levels). A specification whose status is `retired` **SHOULD** also declare a `supersededBy` value (item 11) pointing at the successor.
5. **Parties** — the role of each *party* expected in a document conforming to this specification, the *VID* schemes accepted for each, and whether each of the `issuer` and `recipient` members is **REQUIRED**, **RECOMMENDED**, or **OPTIONAL** in a document. The defaults from [The `issuer` and `recipient` Members](#48-the-issuer-and-recipient-members) apply if the specification is silent, but explicit declaration is **RECOMMENDED**. A **REQUIRED** declaration is enforceable: a *consumer* **MUST** reject documents lacking an in-band member declared **REQUIRED** with `malformedRequest` (see [Consumer Requirements](#72-consumer-requirements) item 5). **RECOMMENDED** and **OPTIONAL** declarations are advisory and impose no rejection obligation. A specification identifies which party fills each framework member by tagging that party `issuer` or `recipient`; a party named only in the *payload* — neither the document's `issuer` nor its `recipient` — carries no such tag. The requirement of the party tagged `recipient` governs the `recipient` member of a request document; because a response document swaps the parties ([Request and Response Variants](#441-request-and-response-variants)), the requirement of the party tagged `issuer` governs the `recipient` member of a response.

    Each party declaration **SHOULD** additionally carry an `identifierScope` — one of `pairwise`, `public`, or `any` — stating which kind of *VID* the specification expects for that party under [Identifier Correlation and Linkability](#131-identifier-correlation-and-linkability). `pairwise` states that the identifier is expected to be scoped to the relationship in which the document is exchanged; `public` states that a cross-relationship identifier is intrinsic to what the task asserts, and a `public` declaration **MUST** be accompanied by the prose justification that subsection requires; `any` states that the specification takes no position and the choice belongs to the parties.

    **What the declaration is, and what it is not.** `identifierScope` is a machine-readable restatement of item 1 of [Identifier Correlation and Linkability](#131-identifier-correlation-and-linkability), which is a binary rule: a party identifier is either scoped to the relationship in which the document is exchanged, or it is reused across relationships and therefore requires the justification that item accompanies it with. The three values mirror that rule and its abstention. The member is **not** a taxonomy of how widely an identifier may be correlated, and its name should not be read as claiming to be one.

    Two consequences follow, and both matter where a party identifier is also described by some other specification's vocabulary:

    - **It is declared by a *Trust Task specification*, about a party role.** It says what this specification expects of whichever identifier fills that role, in every document of the task, before any particular identifier exists. It is not a statement by a holder about an identifier it controls, and it does not become one when a document is produced. Where a holder declares something about its own identifier under another specification, that declaration and this one are different assertions about different subjects; neither overrides the other, and a divergence between them is not by itself a conformance failure. Which identifiers a *consumer* accepts remains governed by its own trust framework, as the following paragraph requires.
    - **An identifier recognisable within a bounded set is, for item 1's purposes, a cross-relationship identifier.** A *VID* that several counterparties can recognise — those of one community, or any other bounded group — is reused across relationships, and the joins item 1 forecloses are available to any two of them who compare what they hold. Such a party is therefore declared `public`, and the justification obligation applies to it in full. The framework deliberately does not grade the width of a cross-relationship identifier: the bound may be unknown to the parties, may grow, and does not change what item 1 is about. A specification whose prose needs to record that the recognisable set is bounded, and to what, states that in the justification.

    The declaration is **descriptive, not prescriptive**, on exactly the terms of items 13 and 14. It states what the specification expects and **MUST NOT** be read as obliging a *consumer* to accept a party identifier merely because it matches the declared scope, nor as licensing the rejection of one that does not; which *VID* schemes and scopes a *consumer* accepts remains a matter for its own trust framework ([The `issuer` and `recipient` Members](#48-the-issuer-and-recipient-members)). A *consumer* that renders or reasons about the value **MUST** treat an absent or unresolvable declaration as no less correlatable than `public`, since an identifier that may be joined across relationships is the more exposed reading. Its purpose is to make [Identifier Correlation and Linkability](#131-identifier-correlation-and-linkability) item 1 machine-readable: a reviewer, a registry, or a *producer* choosing an identifier to present can see which specifications expect a public one without reading every specification for the justification.
6. **Outcome** — a non-normative prose statement of what successful execution of the task achieves between the parties. This is the human-readable counterpart to the payload schema.
7. **Payload JSON Schema** — a normative JSON Schema for the `payload` member that:
   1. Is a valid JSON Schema document under [JSON Schema 2020-12](https://json-schema.org/draft/2020-12/schema).
   2. Sets `$id` to the specification's *Type URI* (without fragment).
   3. Sets `$schema` to `https://json-schema.org/draft/2020-12/schema`.
   4. States how unrecognized payload members are treated — by specifying `additionalProperties` explicitly as `false`, by specifying `unevaluatedProperties` as `false`, or with an accompanying prose statement. A schema assembled by `allOf` over a *shared schema component* ([Shared Schema Components](#66-shared-schema-components)) **MUST** use `unevaluatedProperties`: `additionalProperties` is evaluated by each subschema against the whole instance and cannot see members a sibling subschema matched, so it rejects the composing schema's own members. For the same reason a shared component intended for composition **SHOULD** leave itself open and let the consuming schema close the result.
   5. Is served at its *Type URI* under content negotiation for `application/schema+json`.
   6. Where the specification defines a success-response document (per [Request and Response Variants](#441-request-and-response-variants)), the schema **MUST** contain a sub-schema reachable via `$anchor: "response"` describing the response document's `payload`; the top-level schema (or the sub-schema reachable via `$anchor: "request"`) describes the request document's `payload`. A *consumer* receiving a document whose `type` carries `#response` resolves the response sub-schema by dereferencing the bare *Type URI* and following the `response` anchor. Where the specification defines no success-response document, the schema **MUST NOT** declare a `response` anchor; such tasks are fire-and-forget at the application layer (failures are still reported via `trust-task-error` per [Error Responses](#8-error-responses)).
8. **Proof requirement** — an explicit statement of whether the `proof` member is **OPTIONAL**, **RECOMMENDED**, or **REQUIRED**, together with a brief rationale referencing the threat model addressed (for example, tampering by intermediaries, replay, repudiation by the *producer*, or reliance by third parties beyond the original *consumer*). The declared requirement **MUST NOT** be weaker than the default applicable under [When to Include a Proof](#471-when-to-include-a-proof).

    The statement takes one of two forms. A specification **MAY** declare a **single** requirement applying to every document variant, or it **MAY** declare **per-variant** requirements for the *request* and the *response* separately. The per-variant form exists because the two are relied upon differently: a response retained as evidence by a party outside the original exchange can require a proof where the request that triggered it does not, and the reverse is equally common — a request that destroys state needs to be attributable while the acknowledgement it returns protects nothing. A single value forces the stricter of the two onto both, overstating the requirement on whichever variant needs it less. Where a specification declares no requirement for the *response*, the *request*'s applies to it, so an omission can never weaken a variant.

    A *consumer* applies the requirement declared for the variant it is processing, identified by the document's `type` fragment ([Request and Response Variants](#441-request-and-response-variants)); the rejection rule is unchanged ([Consumer Requirements](#72-consumer-requirements) item 7).

    The **error** variant is deliberately **not** declarable here. An *error response*'s `type` resolves to the framework's `trust-task-error` specification ([The trust-task-error Specification](#81-the-trust-task-error-specification)), which is a different *Trust Task specification* from the one being declared, and [Consumer Requirements](#72-consumer-requirements) item 7 resolves the proof requirement from the specification the document's `type` names. A declaration made here could not reach it.
9. **Task-specific error codes (where used)** — for each extended `code` defined under [Extension by Individual Trust Task Specifications](#85-extension-by-individual-trust-task-specifications), the code identifier, its meaning, its default `retryable` value, and the JSON Schema fragment describing any `details` object it carries. Where no extensions are defined, the specification **SHOULD** state so explicitly.
10. **JSON-LD context (where used)** — if the specification publishes a canonical JSON-LD context, the context **MUST** be served at the specification's *Type URI* under content negotiation for `application/ld+json` (see [JSON-LD Compatibility](#46-json-ld-compatibility) and [Content Negotiation](#62-content-negotiation)). Where no context is published, the specification **SHOULD** state so explicitly.
11. **Successor (`supersededBy`, retired specifications only)** — a `retired` specification **SHOULD** declare its successor as a string of the form `<slug>` or `<slug>/<MAJOR.MINOR>`. The bare-slug form points to "the latest non-retired version of that slug"; the explicit form pins to a specific version. The value is used by the registry's bare-URL redirect (see [Type URI](#61-type-uri)) and by consumer-side deprecation tooling to direct implementers at the recommended replacement. Specifications whose status is not `retired` **MUST NOT** declare `supersededBy`.
12. **Bearer flag (where applicable)** — a *Trust Task specification* that opts out of the [Audience Binding](#482-audience-binding) rule **MUST** declare `bearer: true` in its front matter. The default is non-bearer; specifications omit the field or set `bearer: false` when audience binding applies. A *bearer specification* **MUST** also declare `recipient` as **OPTIONAL** under item 5 and **MUST** include the audience-free rationale required by [Bearer Specifications](#483-bearer-specifications).
13. **Side-effect class** — an explicit classification of the effect that successful execution has on the *recipient party*, declared in front matter as a `sideEffects` object carrying a `level` — one of `none`, `mutating`, or `destructive` — and a brief `rationale`. `none` denotes a read-only or idempotent task that persists no state change (a query, an enumeration, a discovery probe); `mutating` denotes creation or alteration of recoverable state; `destructive` denotes an irreversible or authority-shifting effect (deactivation, rotation of a sole controlling key, deletion, transfer of ownership). For a `destructive` classification the rationale **MUST** name the irreversible or authority-shifting effect. This classification is the structured, machine-actionable counterpart to the prose Outcome of item 6.

    The classification is **descriptive**: it states what the task *does*, not whether approval is required to do it. A *consumer* that gates execution on human approval — for example an agent executing a task delegated to it by a *producer* — MAY derive its approval policy from this class, but that policy is the *consumer*'s alone. Accordingly: a *Trust Task specification* **MUST NOT** declare, in any form, that a task does or does not require consent, human approval, or an authentication step-up; such policy **MUST NOT** be delegable to a specification or to the registry that serves it. A *consumer* that enforces an approval policy **MUST** determine the authoritative side-effect class from the handler it is about to invoke rather than from the declared value alone, and **MUST** treat an absent, unrecognized, or unresolvable declaration as no weaker than `mutating`. The declared class exists to inform and to render, not to authorize.
14. **Exposure class** — a declaration, orthogonal to the side-effect class of item 13, of what successful execution causes to *leave* the recipient or to be *exercised* on the subject's behalf, independent of any change to recipient state. Declared in front matter as an `exposure` object carrying a `discloses` value — one of `none`, `metadata`, or `secret` — and an `actsAsSubject` boolean. `discloses` states the sensitivity of data the task returns to the caller: `none` (an acknowledgement or a determination only), `metadata` (non-secret descriptive data about a subject or resource, such as an enumeration or a status read), or `secret` (confidential material the caller retains, such as released credential material or a usable session blob). `actsAsSubject` is `true` when execution exercises the subject's own authority to produce an attributable effect in the subject's name — a login performed on their behalf, a signature bearing their identity, a credential issued under their authority — even when no data is disclosed and no recipient state changes. Where `discloses` is not `none` or `actsAsSubject` is `true`, a `rationale` naming the disclosed material or the exercised authority is **REQUIRED**.

    The side-effect class (item 13) and the exposure class are **orthogonal**: the former measures the *integrity* effect on recipient state, the latter the *confidentiality and agency* effect of data egress and delegated action. A read-only task (`sideEffects.level: none`) may still disclose a secret; a signing task may change no recipient state yet act with the subject's full authority. Both are governed by the same discipline as item 13: the exposure class is **descriptive, not prescriptive** — a specification **MUST NOT** derive from it a consent requirement — and a *consumer* that gates on it **MUST** determine the authoritative values from the handler it is about to invoke, and **MUST** treat an absent or unresolvable declaration as no less exposed than `discloses: secret` with `actsAsSubject: true`.

    The `exposure` object **SHOULD** additionally carry an `ingests` value — one of `none`, `metadata`, `personal`, or `secret` — grading the sensitivity of the data the task requires the *producer* to **send**. `discloses` is response-only by construction: it grades what execution returns to the caller. A task whose request `payload` carries a full verifiable presentation, a document image, or a free-text field a person typed therefore declares `discloses: none` entirely correctly today, while saying nothing about the most sensitive data in the exchange — and a *consumer* reading only the exposure class would conclude the task moves nothing of consequence in either direction. `none` denotes a request carrying no data about any subject (a discovery probe, an acknowledgement, a control operation); `metadata` denotes non-secret descriptive data (an identifier to be resolved, a status to be read); `personal` denotes data relating to an identified or identifiable person, free text a person authored included; `secret` denotes confidential material the *recipient party* will thereafter hold (credential material, a key, a session blob, an authenticator response). Where `ingests` is `personal` or `secret`, a `rationale` naming the ingested material is **REQUIRED**.

    The enumeration deliberately differs from `discloses`, which has no `personal` value. A disclosure is graded by what the *caller* could do with what it receives; an ingest is graded by what the *recipient* now holds about a person who is frequently not a party to the exchange at all, and for whom the relevant question is not confidentiality but whether the data should have moved. What becomes of it once it has moved is the other half of the question, and item 18 carries that half.

    The discipline of this item governs `ingests` unchanged: the value is **descriptive, not prescriptive**, a specification **MUST NOT** derive a consent requirement from it, a *consumer* that gates on it **MUST** determine the authoritative value from the handler it is about to invoke, and a *consumer* **MUST** treat an absent or unresolvable declaration as no less sensitive than `personal`. The floor is `personal` rather than `secret` because a request that ingests genuine secret material is rare enough that assuming it of every undeclared specification would make the class carry no information, whereas personal data in a request is the ordinary case — and `personal` is already the level at which the minimization guidance of [Privacy Considerations](#13-privacy-considerations) item 1 attaches.

15. **Authorization evidence (consequential tasks)** — a *Trust Task specification* defining a *consequential Trust Task* ([Terminology](#3-terminology)) **MUST** describe any class of authorization evidence a *consumer* needs in order to interpret the task correctly — for example the delegation, mandate, capability, membership, standing, or subject relationship the task presupposes. Where the task presupposes nothing beyond the *consumer*'s own policy, the specification **SHOULD** state so explicitly.

    This declaration is descriptive on exactly the terms of items 13 and 14. It states what authority the task **assumes**, and **MUST NOT** be read as obliging a *consumer* to authorize execution merely because the described evidence is present; the authorization decision remains the *consumer*'s alone under [Consumer Requirements](#72-consumer-requirements) item 10. The bar in item 13 applies here unchanged: a specification **MUST NOT** declare, in any form, that a task does or does not require consent, human approval, or an authentication step-up.

    A specification **MAY** additionally declare that a verified assertion carried by the task *is* authorization evidence for a stated purpose — the design [Consumer Requirements](#72-consumer-requirements) item 10 contemplates, in which a proof is relied upon as authorization rather than merely as integrity. Such a declaration **MUST** name the purpose and **MUST NOT** extend beyond it, and remains subject to the *consumer*'s policy accepting the assertion for that purpose.

    Unlike items 5, 8, 12, 13, and 14, this declaration is satisfied in the specification's prose and has no front-matter field; it is not machine-validated.

16. **Execution checkpoints (multi-stage consequential tasks)** — a specification describing multi-stage consequential execution **SHOULD** identify any additional points at which validity or authority is expected to be re-evaluated, beyond the one [Consumer Requirements](#72-consumer-requirements) item 12 requires before each irreversible effect. A specification whose stages must not be partially applied **SHOULD** say so explicitly, so that a *consumer* deciding whether to stop knows which of stopping and continuing its author considered the safer failure.

    Where the specification defines its own completion deadline — an instant after which the task's outcome is no longer meaningful — it declares that in its `payload` and states what lapsing means for the task. The framework's `expiresAt` bounds acceptance only ([Top-Level Members](#42-top-level-members)) and **MUST NOT** be relied upon to terminate execution.

17. **Freshness (consequential tasks)** — a *Trust Task specification* defining a *consequential Trust Task* ([Terminology](#3-terminology)) **MUST** require the `issuedAt` member, raising the framework's **SHOULD** ([Top-Level Members](#42-top-level-members)) to a **MUST** for documents conforming to it. Such a specification **SHOULD** also state the acceptance window it expects its consumers to apply, and **MAY** require `expiresAt` where the task's outcome is meaningless after a fixed interval.

    The reason is the one given under [Consumer Requirements](#72-consumer-requirements): the duplicate-execution protection of item 11 is exactly the rule that matters for a *consequential* task, and it is implementable only over a bounded window. A document carrying neither `expiresAt` nor `issuedAt` gives its *consumer* no window to place it in, so item 11 obliges that *consumer* either to retain its record of the document forever or to refuse to execute it at all — and a framework whose most dangerous tasks are the ones a *consumer* may have to refuse for want of a timestamp has the requirement in the wrong place. Requiring `issuedAt` of the specification puts it on the party that can satisfy it for nothing.

18. **Retention class** — a declaration of what the *recipient party* is expected to keep of a document conforming to this specification, and for how long. Declared in front matter as a `retention` object carrying a `class` — one of `transient`, `exchange`, or `durable` — and a `rationale`. `transient` denotes a task whose data the recipient needs only in order to produce its reply and is expected to discard when the exchange closes; `exchange` denotes data the recipient holds for the life of the exchange and for the windows this framework already obliges it to hold — the duplicate-execution record of [Consumer Requirements](#72-consumer-requirements) item 11, an acceptance window, a suspension it may be asked to resume — and no longer; `durable` denotes data the recipient is expected to keep beyond the exchange because placing it there is what the task is *for*: a compliance record, an issued credential, a registry entry, an audit trail. A `durable` classification **MUST** name in its rationale what is retained and what obliges its retention.

    The framework has carried no retention vocabulary at all, and the gap is not neutral. Every rule in this document and in the published specifications that uses the word *retain* mandates **more** retention, for good reasons: a proof is required so a document can be relied upon after delivery ([When to Include a Proof](#471-when-to-include-a-proof)), a digest is retained so a duplicate can be absorbed ([Consumer Requirements](#72-consumer-requirements) item 11), an error is identified so it can serve as evidence ([Error Payload](#82-error-payload)). Each of those is right on its own and their sum is a framework that tells a *recipient party* what it must keep and never once what it may let go — so the safe reading of every task has been to keep everything. `retention` is the missing half: a per-specification statement of which of those obligations actually reaches this task, so that a *consumer* can tell the data it is obliged to hold from the data it merely still has. It pairs with `exposure.ingests` (item 14): that value says what arrived, this one says what becomes of it.

    The classification is **descriptive, not prescriptive**, on exactly the terms of items 13 and 14. It states what the task's design assumes and **MUST NOT** be read as obliging a *recipient party* to retain anything, as authorizing it to retain anything, or as discharging any retention or deletion obligation the *consumer* is under by law, contract, or governance framework — those bind the *consumer* and are no more delegable to a specification than a consent requirement is (item 13, [Governance Considerations](#14-governance-considerations)). A *consumer* that gates on the value **MUST** determine the authoritative class from the handler it is about to invoke rather than from the declared value alone, and **MUST** treat an absent or unresolvable declaration as no weaker than `durable`.

    A `transient` declaration does **not** disapply [Consumer Requirements](#72-consumer-requirements) item 11. That record is a digest of what was accepted rather than a copy of the document's contents, and a *consumer* retains it for the acceptance window of item 13 whatever this class says — the two are not in tension, and a specification whose data must genuinely not outlive the exchange is served by the digest precisely because a digest is not the data.

19. **Free-text members** — for every member of its `payload` whose value is free text, meaning a string whose content no enumeration, pattern, or external vocabulary constrains, a *Trust Task specification* **MUST**:

    1. Declare a `maxLength` in the payload JSON Schema.
    2. State who reads the value — an operator, an approver, a log, a downstream system, nobody — and whether the *recipient party* is expected to retain it (item 18).
    3. State whether the value is trusted. Where it is authored by any party other than the one whose signature covers the document's assertions, the specification **MUST** say so explicitly and **MUST** require that any surface rendering it attribute it to its author.

    Such a member **SHOULD** be **OPTIONAL**, and a specification **SHOULD** prefer a closed enumeration the specification itself defines — which [Naming Conventions](#410-naming-conventions) item 4 already governs — accompanied where genuinely needed by one bounded, optional note, over a free-text member carrying the meaning that the enumeration should have carried.

    `task-consent/request/0.1`'s `note` is the pattern to copy: 500 characters, optional, attributed on every rendering surface to the `requester` who wrote it, declared **explicitly untrusted** as the one member of that payload whose prose the executor did not author, and forbidden from substituting for, reordering, or obscuring the executor-authored effects it appears beside. Every one of those constraints is in the specification, where the party who knows what the field is for could state it.

    Two things drive the rule. The first is **disclosure**: a free-text member is the one place in a *Trust Task document* where the schema constrains the shape and nothing constrains the content, so it is where personal data arrives in a task that declares it ingests none ([Specification Requirements](#73-specification-requirements) item 14), where a secret arrives pasted by a person who was asked for a reason, and where instructions addressed to a downstream reader arrive in a field the specification believed was a comment. A bound and a stated audience do not prevent that, but they are the minimum that lets a *consumer* reason about it, and an unbounded field cannot be reasoned about at all. The second is **wire cost**: an unbounded string is an unbounded document. [Parser Hardening](#122-parser-hardening) has a *consumer* bound the body at the transport layer, which is the right defence and the wrong place to decide the number — a transport-layer limit is one figure for every task the *consumer* implements, where only the specification knows whether this field is a reason code or a paragraph. Declaring the bound in the schema also makes it validatable, so an oversized value is refused as `malformedRequest` by the pipeline every *consumer* already runs rather than by a limit each one picks for itself.

    A member whose content *is* constrained — an enumerated status, a *VID*, a *Type URI*, a timestamp, a value carried verbatim from an external vocabulary ([Naming Conventions](#410-naming-conventions) item 5) — is not free text and this item does not reach it.

20. **Outcome evidence (cited exchanges)** — a *Trust Task specification* whose exchanges a party outside the exchange may cite as evidence that an event occurred — a DTG credential's `taskContext`, an audit record, a governance decision ([Naming an Exchange from Outside the Framework](#491-naming-an-exchange-from-outside-the-framework)) — **MUST** declare which success response constitutes that exchange's *outcome evidence* under [Evidence That a Cited Exchange Completed](#494-evidence-that-a-cited-exchange-completed), naming it by *Type URI* with its `#response` fragment. For that response the specification:

    1. **MUST** define its `payload` schema (item 7.6), so that successful termination is observable from the exchange's own documents;
    2. **MUST** declare `proof` **REQUIRED**, as a single requirement or for the response variant (item 8), since the response is relied upon by parties beyond the original *consumer*;
    3. **MUST** require the `issuedAt` member (item 17), whether or not the task is otherwise *consequential*, so that a relying party can place the evidence in time;
    4. **SHOULD** declare retention class `durable` (item 18), naming the citation as what obliges retention; and
    5. **MUST** declare how that response is bound to the initiating document, as payload members a *consumer* can compare, each named by a JSON Pointer ([RFC 6901](https://www.rfc-editor.org/rfc/rfc6901)) evaluated against the root of the *Trust Task document* that carries it (for example `/payload/vwc/taskContext`). The binding is one of:

        1. members carrying the initiating document's `id` and its task digest ([Binding a Citation to the Document It Names](#493-binding-a-citation-to-the-document-it-names)); or
        2. a member reproducing an unguessable value that the initiating document freshly supplied, such as a presentation challenge, together with the initiating-document member that supplied it. That value's freshness is guaranteed only by the initiating document's *producer*, which can reuse it together with a `threadId` and so present one exchange's response as the outcome evidence of another. A specification **MAY** declare this form only where it requires the initiating document's *producer* to be the issuer of the citing artifact, so that a relying party already depends on that producer's honesty for what the artifact states. Otherwise it **MUST** declare the first form.

        Sharing a `threadId` does not satisfy this sub-item.

    Where a cited exchange is conducted under more than one specification — a session opened under one and completed by a task conducted on its thread under another — the specification governing the **initiating** document makes the declaration, since that document's `type` is what a relying party holds when it looks for one. That specification **MUST** also require the initiating document to carry `recipient` (check 4 of [Evidence That a Cited Exchange Completed](#494-evidence-that-a-cited-exchange-completed) identifies the responder by it), so an exchange whose initiating document is governed by a *bearer specification* ([Bearer Specifications](#483-bearer-specifications)) and names no recipient cannot be cited as evidence of completion. A specification that defines no success response of its own, and whose only positive terminal signal is the empty `#response` of [Acknowledging a Fire-and-Forget Task](#442-acknowledging-a-fire-and-forget-task), **MUST NOT** be cited as evidence of completion unless it declares a success response under this item — which, under sub-items 1 and 5, gives that response a defined payload carrying its binding, and so replaces the empty acknowledgement.

A worked example of a *Trust Task specification* satisfying these requirements appears in [Appendix A](#appendix-a-example-trust-task-specification).

## 8. Error Responses

*This section is normative.*

A *recipient party* that cannot or will not act upon a received *Trust Task document* **MAY** return an **error response** describing why. Error responses are themselves *Trust Task documents* of a framework-defined type, so that one validation, signing, and transport pipeline serves both successful tasks and their refusals.

The framework distinguishes the two reply forms cleanly:

* A **success response** uses the request's *Type URI* with the fragment `#response` (see [Request and Response Variants](#441-request-and-response-variants)). Its payload shape is defined by the originating *Trust Task specification*.
* An **error response** uses the framework's distinct `trust-task-error` *Type URI* (defined below). Its payload shape is defined by this framework, independent of the originating specification.

A *recipient party* **MUST NOT** report failure by emitting a `#response`-variant document of the originating spec, nor success by emitting a `trust-task-error` document. The two reply types are not interchangeable.

### 8.1 The trust-task-error Specification

The framework reserves the slug `trust-task-error` for the error-response *Trust Task specification* at:

```
https://trusttasks.org/spec/trust-task-error/<MAJOR.MINOR>
```

An *error response* is a *Trust Task document* whose `type` is the URI above. Its `payload` carries the standard error structure defined in [Error Payload](#82-error-payload). The `id` member of an *error response* identifies the error instance and **MUST NOT** be reused; correlation back to the original task being responded to is carried by the framework's `threadId` member ([The `threadId` Member](#49-the-threadid-member)).

The *error response*'s `issuer` is the *consumer* that emitted it (the *reporting consumer* in the conformance language of the `trust-task-error` specification at [Reserved Response-Type Slugs](#86-reserved-response-type-slugs)). Its `recipient` is the party the *consumer* wishes to inform of the failure. For most rejections — `expired`, `unsupportedType`, `unsupportedVersion`, `proofRequired`, `proofInvalid`, `taskFailed`, and the rest of [Standard Error Codes](#83-standard-error-codes) — that party is the *original producer* as carried in the rejected document's in-band `issuer` member.

The exception is `identityMismatch` (and any rejection raised in the same evaluation step that surfaced the mismatch): under such a rejection the rejected document's in-band `issuer` is by definition the contested identity, and **MUST NOT** be used as the error response's `recipient`. A *consumer* that emits an error response under `identityMismatch` **MUST** address the response to the transport-authenticated sender of the rejected document, and **MUST NOT** address it to the in-band `issuer`. Where no transport-authenticated sender is available, the *consumer* **SHOULD NOT** emit an error response at all — sending one to the contested in-band identity would constitute an oracle, and (in any transport that signs error responses) would compel the *consumer* to emit a signed document about a party that did not in fact participate in the exchange.

The *consumer* **MUST** likewise sanitize the `payload.message` member of an `identityMismatch` error response: a free-text message that reveals the *consumer*'s expected transport-authenticated identity, or the contested in-band value, leaks identity information to a possibly hostile sender. The standard wire form for this code is the code identifier alone, optionally accompanied by a non-identifying message (e.g. `"identityMismatch: in-band identity does not match transport-derived identity"`). The general form of this rule — which binds every code, not only this one — is [What a `message` May Not Say](#821-what-a-message-may-not-say).

### 8.2 Error Payload

The `payload` of an *error response* has the following members. The correlation back to the *Trust Task document* this error reports on is carried at the framework level by the `threadId` member ([The `threadId` Member](#49-the-threadid-member)), which a *producer* of an error response **MUST** set.

`threadId` correlates the exchange for a party that saw the originating request. It identifies nothing to anyone else: it is opaque, and the payload otherwise names neither the *Trust Task specification* the failure occurred under nor the document instance that triggered it. A party handed a retained error — a verifier evaluating it as evidence, an auditor reconstructing a sequence — sees a `code` and a `retryable` flag and cannot tell what failed. For an extended code the slug namespace ([Extension by Individual Trust Task Specifications](#85-extension-by-individual-trust-task-specifications)) hints at the family; for the standard codes of [Standard Error Codes](#83-standard-error-codes) there is no signal at all.

The `inResponseTo` member closes that. A *consumer* emitting an error response **SHOULD** populate it, and **MUST** populate it where the error is intended to be retained, replayed, or relied upon by parties beyond the original *producer* — the same condition under which [When to Include a Proof](#471-when-to-include-a-proof) makes a `proof` mandatory, and for the same reason: an error that cannot be attributed *and* cannot be identified is not evidence of anything. Its `typeUri` carries the reported-on document's `type` including any `#request` or `#response` fragment, which is what tells a consumer whose semantics apply to an extended `code`; its `id` carries that document's *document identifier*, which [The `id` Member](#43-the-id-member) makes globally unique and never reused, so it names one instance where `threadId` names an exchange.

Under `identityMismatch` a *consumer* **SHOULD** omit `inResponseTo.id`: per [The trust-task-error Specification](#81-the-trust-task-error-specification) the response is addressed to the transport-authenticated sender rather than the in-band `issuer`, and that party did not necessarily compose the document whose identifier would be echoed.

| Member | Required | Type | Description |
|---|---|---|---|
| `code` | **MUST** | string | A short identifier for the failure category. **MUST** be one of the codes in [Standard Error Codes](#83-standard-error-codes) or an extended code as defined in [Extension by Individual Trust Task Specifications](#85-extension-by-individual-trust-task-specifications). |
| `inResponseTo` | **SHOULD** | object | Identifies the *Trust Task document* this error reports on: `typeUri` (its `type`, including any fragment) and `id` (its *document identifier*). See below. |
| `message` | **SHOULD** | string | A human-readable description of the error, subject to the disclosure rule of [What a `message` May Not Say](#821-what-a-message-may-not-say). Non-normative as to the cause of the failure; intended for logs and operator UI. |
| `retryable` | **MUST** | boolean | `true` if the *producer* of the original document **MAY** retry the task; `false` if retrying with the same document or credentials is not expected to succeed. |
| `retryAfter` | **MAY** | string (date-time) | An [RFC 3339](https://www.rfc-editor.org/rfc/rfc3339) timestamp before which the *producer* **SHOULD NOT** retry. Meaningful only when `retryable` is `true`. |
| `details` | **MAY** | object | Task-specific extension data; see [Extension by Individual Trust Task Specifications](#85-extension-by-individual-trust-task-specifications). Bounded per [Bounding `details`](#822-bounding-details). The framework itself defines the shape carried under the `cancelled` code; see [Effects Reported with `cancelled`](#831-effects-reported-with-cancelled). |

> **Example 5 — An error response** *(non-normative)*
>
> ```json
> {
>   "id": "9e2a1c44-7b81-4d3e-9b51-7a3c89e3d1f2",
>   "type": "https://trusttasks.org/spec/trust-task-error/0.2",
>   "threadId": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
>   "issuer": "did:web:maintainer.example",
>   "recipient": "did:web:org.example",
>   "issuedAt": "2026-06-11T14:05:00Z",
>   "payload": {
>     "code": "expired",
>     "message": "Task expired at 2026-06-11T14:00:00Z.",
>     "retryable": false
>   },
>   "proof": {
>     "type": "DataIntegrityProof",
>     "cryptosuite": "eddsa-rdfc-2022",
>     "verificationMethod": "did:web:maintainer.example#key-1",
>     "created": "2026-06-11T14:05:00Z",
>     "proofPurpose": "assertionMethod",
>     "proofValue": "z58D..."
>   }
> }
> ```

#### 8.2.1 What a `message` May Not Say

The `message` member is a **wire-exposed value**, and it is exposed under the least favourable conditions the framework has: an *error response* is emitted to a party the *consumer* has, by construction, just failed to validate; it is frequently emitted before any authorization decision has been reached ([Consumer Requirements](#72-consumer-requirements) item 10); and it is retainable by whoever receives it.

A *consumer* emitting an *error response* **MUST NOT** place in `message`:

1. **Consumer-internal state** — an internal identifier, a queue or worker name, a policy or rule name, a host name, a file or database path, a stack frame, or a software version.
2. **The contested value of a mismatched party** — the in-band value the *consumer* rejected, or the transport-authenticated identity it expected instead. This is the rule already stated for `identityMismatch` in [The trust-task-error Specification](#81-the-trust-task-error-specification); it is restated here because it is not peculiar to that code.
3. **Resolver, verifier, or key-status internals** — the URL the *consumer* dereferenced, a resolver's own error text, the verification method it tried, the status list it consulted, or the reason a signature failed beyond the fact that it did.

A `message` **MUST** instead be derived from the `code` identifier and from the *Trust Task specification*'s public vocabulary — the same material the receiving party could have read for itself.

This applies to **every** code in [Standard Error Codes](#83-standard-error-codes) and to every extended code, not only to `identityMismatch`. The rule was written for that code first because the leak is most obvious there, not because that is where it applies. Every other rejection is emitted on the same path, to the same possibly-unauthenticated party, and any consumer-internal fact placed in the message makes the *error response* an oracle: a sender that can vary one member of a document and read the message back can enumerate which identities the *consumer* recognizes, which identifiers it can resolve, and which of its dependencies are currently reachable — an identity- and reachability-probing instrument the *consumer* pays for and operates on the sender's behalf. The *consumer*'s own logs are the correct place for everything this rule excludes; nothing here bars recording it locally, only sending it.

#### 8.2.2 Bounding `details`

`details` is the one member of the error payload whose size the framework does not otherwise constrain, and it travels in the direction no bound reaches: a *producer* that bounded its request bounded nothing about the reply. It is also emitted on a failure path — the path least exercised in testing and most readily reached by an unauthenticated party.

Accordingly, a *Trust Task specification* that defines a `details` shape for a code **MUST** declare a bound for it: a maximum serialized size, a maximum number of members, or both, alongside the JSON Schema fragment required by [Extension by Individual Trust Task Specifications](#85-extension-by-individual-trust-task-specifications). Where no bound is declared for the code being emitted, a *consumer* **MUST NOT** emit a `details` object exceeding **4096 bytes** of [RFC 8785](https://www.rfc-editor.org/rfc/rfc8785)-canonicalized UTF-8 or **16** immediate members.

A party receiving a `details` object that exceeds either bound **MUST NOT** reject the *error response* on that account — the response is already a failure report, and discarding it loses the `code` the party needs — but **MUST** ignore the contents of `details` and **MUST** still honor `code`, `retryable`, and `retryAfter`, on the same terms as the fallback for an unrecognized extended code. Any free-text member inside `details` is subject to [Specification Requirements](#73-specification-requirements) item 19 and to the disclosure rule of [What a `message` May Not Say](#821-what-a-message-may-not-say); `details` is not a route around either.

### 8.3 Standard Error Codes

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
| `idConflict` | The document's `id` matches one the *consumer* has already accepted, but its content differs — see [Consumer Requirements](#72-consumer-requirements) item 11. | `false` |
| `cancelled` | The *consumer* stopped the task on its own initiative — operator action, policy, capacity, or a compliance hold. Distinct from a *producer*-requested cancellation, which is answered by a response to the control document ([Task Control](#11-task-control)). | `false` |
| `taskFailed` | The *recipient party* attempted the task and could not complete it; further detail **SHOULD** appear in `details`. | varies |
| `unavailable` | The *recipient party* is temporarily unable to process the task. | `true` |
| `internalError` | The *recipient party* encountered an unexpected internal failure. | `true` |

The "Default `retryable`" column gives the value an emitter of an error response **SHOULD** use unless task-specific knowledge dictates otherwise. The actual `retryable` value carried in a given *error response* is authoritative.

#### 8.3.1 Effects Reported with `cancelled`

A *consumer* emitting `cancelled` has stopped work it had accepted, and [Consumer Requirements](#72-consumer-requirements) item 12 already obliges it to distinguish **partial execution** from a task that was never begun — because a *producer* that cannot tell the two apart cannot decide whether to reissue, and because [Control Does Not Roll Back](#114-control-does-not-roll-back) makes clear that nothing is undone by stopping. What that obligation lacked was a shape: the same fact, reported to the same *producer*, is machine-readable when the *producer* asked for the stop and prose when the *consumer* decided on it.

The framework therefore defines the `details` shape for the `cancelled` code as a single member, `effects`, whose value is an array of effect objects:

| Member | Required | Type | Description |
|---|---|---|---|
| `description` | **MUST** | string | Human-readable statement of the effect that occurred. |
| `ref` | **MAY** | string | An identifier for the effect where one exists — a credential identifier, a record identifier, a transaction reference — so that a compensating task can name it. |
| `reversible` | **MAY** | boolean | Whether the *consumer* believes this effect can be compensated by a further *Trust Task*. Advisory. Absent means unknown, which a *producer* **SHOULD** treat as no weaker than `false`. |

This is deliberately the **same** array-of-effects shape the `trust-task-control` specification defines for the response to a *producer*-requested cancellation. The two report the identical fact — what had already landed when the stop took hold — and differ only in which party decided, which is a distinction the framework keeps precisely so that neither party has to infer it ([Task Control](#11-task-control)). A *producer* deciding whether to invoke a compensating task should not have to parse that decision out of prose in one direction and read it from an array in the other.

The rules:

1. A *consumer* emitting `cancelled` after one or more irreversible or externally visible effects have occurred **MUST** populate `effects` with one entry per effect.
2. An `effects` value of `[]` means **nothing landed**. An **absent** `effects` member means the *consumer* did not report, and a *producer* **MUST NOT** read its absence as "nothing landed" — the general rule of [Document Lifecycle](#412-document-lifecycle) applies to an omitted member as it does to an absent reply.
3. `description` is free text and is bound by [Specification Requirements](#73-specification-requirements) item 19 and by [What a `message` May Not Say](#821-what-a-message-may-not-say). It names the effect on the *recipient party*'s own state, which is what the *producer* needs; it **MUST NOT** name the internal mechanism that produced it.
4. `effects` counts toward the bound of [Bounding `details`](#822-bounding-details). A *consumer* whose effect list would exceed it reports the effects that are consequential for the *producer* and says so in the last `description`, rather than truncating silently.

### 8.4 Retry Semantics

In this specification, "retrying" means re-sending a *Trust Task document* bit-for-bit identical to the one that elicited the *error response* — same `id`, same `payload`, same `proof`. Issuing a *new* document, even one addressing the same underlying intent, is not a retry; see below.

A *party* that receives an *error response* **MUST NOT** retry the original *Trust Task document* if `retryable` is `false`. When `retryable` is `true`, the party **SHOULD** wait at least until any `retryAfter` value before retrying, and **SHOULD** apply backoff appropriate to the transport in use.

Retrying is safe for a *consequential Trust Task* precisely because [Consumer Requirements](#72-consumer-requirements) item 11 requires the *consumer* to absorb the duplicate rather than execute it again. The two rules are one mechanism seen from each end: this section tells a *producer* that the only safe resend is the bit-for-bit identical document, and item 11 tells a *consumer* that a bit-for-bit identical document it has already accepted **MUST NOT** produce a second consequential effect. A *producer* that "retries" by re-signing, re-stamping `issuedAt`, or otherwise altering the bytes has not retried — it has issued a different document under a reused `id`, which item 11 requires the *consumer* to reject with `idConflict` ([Standard Error Codes](#83-standard-error-codes)). Where a *producer* genuinely needs a fresh attempt, it issues a new document with a fresh `id`, as the paragraph below describes.

A `false` value of `retryable` represents a hard failure for that specific document. It does not prohibit the *producer* from issuing a *new* *Trust Task document* — that is, a document with a fresh `id` (and **SHOULD** the same `threadId` to preserve correlation) — addressing the cause of the failure. For example, after receiving an *error response* of `code = proofInvalid` with `retryable = false`, the *producer* **MUST NOT** re-send the failed document, but **MAY** issue a new document carrying a valid `proof`.

### 8.5 Extension by Individual Trust Task Specifications

An individual *Trust Task specification* **MAY** define additional error codes specific to its task. Extended codes **MUST** be namespaced, separated from the local code by a colon, e.g. `acl/change-role:stateMismatch`. The namespace **MUST** be one of exactly two things:

1. **The emitting specification's own `<slug>`** — that is, the slug of the *request* the *error response* refers to. This is the default and covers any code the specification defines for itself.
2. **A *family namespace*** — a proper path prefix of that slug, formed of one or more of its leading `/`-separated segments (for `did-management/did/delete`, the permitted prefixes are `did-management/did` and `did-management`). A family namespace **MUST** be used only for a code whose meaning is defined once for the whole family — in a shared convention that the family's specifications reference — and never to give a specification-specific code a broader name than it has earned.

The namespace **MUST NOT** be the slug of a *related or referenced* specification, and this remains true under rule 2: a proper prefix of a specification's own slug names that specification's own family and can never name a sibling. A *consumer* of `acl/change-role` that needs to surface a rejection borrowed conceptually from `acl/revoke` therefore emits `acl/change-role:<local>` or `acl:<local>`, never `acl/revoke:<local>`. Extended codes **MUST NOT** shadow any code listed in [Standard Error Codes](#83-standard-error-codes).

*This paragraph is non-normative.* Rule 2 exists because families do share failure modes. Every specification under `did-management` can reject a request naming a domain the *consumer* does not host, and that rejection means the same thing in each of them; stating it once as `did-management:unknownDomain` lets a *consumer* handle the family uniformly, where per-slug codes would oblige it to enumerate every member to recognize one condition. The narrowness of rule 2 is what keeps this safe: because a family namespace is always a prefix of the emitting slug, a *consumer* can verify the namespacing of a received code against the document's `type` alone, with no registry lookup.

A *consumer* (not only the spec author) **MAY** mint additional namespaced codes for invariants the specification did not enumerate, provided the namespacing rule above is honoured. The framework's fallback-to-`taskFailed` rule for unrecognized extended codes (see the third paragraph below) keeps these consumer-minted codes interoperable with clients that only implement the canonical set.

An individual *Trust Task specification* **MAY** also define the structure of `details` for its own error responses. Where it does so, the specification **MUST** state which `code` values may carry a `details` object and **MUST** provide a JSON Schema fragment describing the `details` shape for each.

A *consumer* that does not recognize an extended `code` **SHOULD** treat the error as if its code were `taskFailed` and **MUST** still honor the `retryable` and `retryAfter` members.

The `details` member defined here is distinct from the `ext` extension member defined in [The `ext` Extension Member](#451-the-ext-extension-member). `details` carries *task-specific structured data tied to a specific error `code`*, defined by the spec author; its shape is constrained by the JSON Schema fragment the specification publishes for each carrying code. `ext` carries *vendor-namespaced extension data at payload or nested-object level*, defined by the ecosystem; its namespace structure is opaque to the framework. Both members **MAY** appear on the same *error response* and serve different purposes — implementations **MUST NOT** treat them as interchangeable.

> **Example 6 — An error response with an extended code and `details`** *(non-normative)*
>
> ```json
> {
>   "id": "c4d2f713-9a8e-4d04-b29c-2f1b0b4cbe71",
>   "type": "https://trusttasks.org/spec/trust-task-error/0.2",
>   "threadId": "7b1e4d09-3c62-4a17-9f2d-51c8ab3e7d40",
>   "issuer": "did:web:maintainer.example",
>   "recipient": "did:web:org.example",
>   "issuedAt": "2026-06-12T11:04:00Z",
>   "payload": {
>     "code": "acl/change-role:roleNotRecognized",
>     "message": "Role 'principal' is not part of this ACL's role vocabulary.",
>     "retryable": false,
>     "details": {
>       "offendingRole": "principal",
>       "knownRoles": ["member", "moderator", "admin"]
>     }
>   }
> }
> ```
>
> A *consumer* implementing the `acl/change-role` *Trust Task specification* interprets the extended `code` per that specification's declarations (see [Specification Requirements](#73-specification-requirements), item 9). A *consumer* that does not implement `acl/change-role` treats the error as if `code = taskFailed`, retains `retryable = false`, and ignores the contents of `details`.

### 8.6 Reserved Response-Type Slugs

The framework reserves the following additional response-type *Trust Task specification* slugs. These slugs **MUST NOT** be used for any individual *Trust Task specification* registered under [Type URI](#61-type-uri).

| Slug | Purpose |
|---|---|
| `trust-task-ok` | **Deprecated at framework version 0.5.0.** A courtesy acknowledgement — confirming that a task defining no success response of its own was received and performed, and optionally surfacing opaque references the *consumer* chose to share. Never relied upon. Superseded by the empty `#response` of [Acknowledging a Fire-and-Forget Task](#442-acknowledging-a-fire-and-forget-task). |
| `trust-task-next-step` | A recipient-suggested continuation — indicating that the original task was understood but cannot complete in isolation, together with the next *Trust Task* the *recipient party* expects in order to proceed. |

`trust-task-next-step` is published; its registry entry at `https://trusttasks.org/spec/trust-task-next-step/0.1` defines the normative `payload` shape and conformance requirements, in the same relationship to this section that the `trust-task-discovery` entry has to [Discovery and Capability Negotiation](#10-discovery-and-capability-negotiation). A *next step* is a **third** disposition alongside the success response and the *error response* of this section: it reports that the originating task was understood and is **blocked**, leaving the exchange open where the other two close it. A *consumer* **MUST NOT** report a blocked task as an *error response*, nor a refusal as a *next step*; the three replies are not interchangeable. A *next step* confers no authorization — the *Type URI* it names is a suggestion the receiving party evaluates under its own policy, on the same advisory footing as a discovery response ([Status of the Discovery Response](#104-status-of-the-discovery-response)).

`trust-task-ok` is published at `https://trusttasks.org/spec/trust-task-ok/0.1`, and is **deprecated as of framework version 0.5.0** in favour of the empty `#response` acknowledgement defined in [Acknowledging a Fire-and-Forget Task](#442-acknowledging-a-fire-and-forget-task). It remains a **courtesy acknowledgement** with the meaning it has always had: a *consumer* **MAY** return one to confirm that it received and performed a *Trust Task* whose specification defines no success-response document of its own, and **MUST NOT** send one in place of a response a specification does define — two success dispositions for one task leave a *producer* unable to tell which is authoritative.

Its weakness is deliberate, and is also why it is deprecated. A *producer* **MUST NOT** rely on receiving an acknowledgement, and **the absence of one carries no information**: a *consumer* may not implement the specification, may implement it and not send one, or the document may be lost. A *producer* that reads absence as failure and reissues a *consequential Trust Task* causes exactly the duplicate effect [Consumer Requirements](#72-consumer-requirements) item 11 exists to prevent. A whole *Trust Task specification* — a registry entry, a schema, a generated type in each supported language, a `type` a *producer* must learn to recognize that bears no relation to the one it sent — to carry a fact that may not be relied upon is more machinery than the fact is worth, and the reply the *producer* already awaits can carry it instead. An acknowledgement that genuinely matters is **not** either form: such a task declares its own success response, or a dedicated receipt task with its own proof requirement.

The consequences of deprecation are deliberately small, because the slug's own weakness makes them small:

* A *consumer* **SHOULD NOT** emit a `trust-task-ok` document from this version onward, and **SHOULD** emit the empty `#response` instead.
* A *producer* **MUST** continue to accept a `trust-task-ok` document, on the terms above, for as long as the specification is served. It never conveyed anything a *producer* was entitled to act on, so accepting it costs nothing.
* The slug and its *Type URI* remain **RESERVED** under [Type URI](#61-type-uri) permanently, whatever becomes of the specification published under it. Retirement of the registry entry itself follows [Maturity Levels](#53-maturity-levels) and is a matter for the registry, not for this document.

Implementations encountering a *Trust Task document* of a reserved type whose specification is not yet published **MAY** ignore the document or **MAY** return an `unsupportedVersion` *error response*.

## 9. Transport Bindings

*This section is normative.*

The framework deliberately leaves transport unconstrained ([Design Goals](#11-design-goals) Goal 1): a *Trust Task document* can be conveyed over any channel that preserves its content. To make that composability work in practice, each transport protocol used to carry *Trust Task documents* **SHOULD** be accompanied by a *transport binding* specification.

A *transport binding* defines how *Trust Task documents* are exchanged over a specific transport — for example, DIDComm, the IETF Trust Spanning Protocol (TSP), HTTPS with mutual-TLS, AMQP, or paper. It is the integration layer between the framework's transport-agnostic semantics and the realities of a particular transport.

### 9.1 What a Transport Binding Specifies

A *transport binding* **SHOULD** specify each of the following:

* **Document carriage.** How a *Trust Task document* is placed onto and retrieved from the transport (request body, message payload, envelope field, attachment, etc.).
* **Field population from transport context.** Which framework members the binding **derives** from transport-derived information — typically `issuer` (from a transport-authenticated sender), `recipient` (from a transport-authenticated addressee), and any signature metadata that lets a *consumer* verify the framework `proof` against transport-bound keys or, per [When to Include a Proof](#471-when-to-include-a-proof), accept the document without an in-band `proof`. Per [Precedence of In-Band over Transport-Derived Identity](#481-precedence-of-in-band-over-transport-derived-identity), the binding fills these members from the transport **only when the corresponding in-band member is absent**; when the in-band member is present, the transport-derived value is used as a cross-check, not as a substitute.
* **Consistency enforcement.** The behavior when an in-band framework member and its transport-derived equivalent disagree. The framework requires they **MUST** be consistent (see [The `issuer` and `recipient` Members](#48-the-issuer-and-recipient-members), [Precedence of In-Band over Transport-Derived Identity](#481-precedence-of-in-band-over-transport-derived-identity), and [Consumer Requirements](#72-consumer-requirements)); the binding states how the comparison is performed for the transport in question (for example, how a DID carried in-band is matched against a transport-authenticated DID).
* **Thread correlation (where the transport has its own).** Several transports carry their own correlation and parent-correlation identifiers — DIDComm's `thid` and `pthid`, for example. Where a binding maps these onto the framework's `threadId` ([The `threadId` Member](#49-the-threadid-member)) and `parentThreadId` ([The `parentThreadId` Member](#492-the-parentthreadid-member)), it **MUST** state that mapping, and the mapping **MUST** require the two to agree only when **both** are explicitly present. The two layers identify different things and typically default into their own identifier spaces — a transport's correlation identifier commonly falls back to that transport's own message identifier, which is not the *Trust Task document*'s `id` — so requiring agreement unconditionally would fail exchanges that are otherwise conforming. As everywhere else in [Precedence of In-Band over Transport-Derived Identity](#481-precedence-of-in-band-over-transport-derived-identity), the in-band member is authoritative and the transport value is a cross-check; a *producer* **SHOULD** populate the transport's identifiers from the framework members rather than the reverse. A disagreement is a structural inconsistency and is reported as `malformedRequest`, not `identityMismatch` — no party's identity is in dispute.
* **Transport security profile.** The integrity, authentication, confidentiality, and freshness guarantees the transport provides, so that *consumers* can correctly evaluate the `proof` requirement under [When to Include a Proof](#471-when-to-include-a-proof). A *transport binding* from which any framework security or identity requirement is derived — any binding that populates `issuer` or `recipient` from transport context, or that addresses the omission of `proof` — **MUST** specify this profile; for such a binding the item is not optional. See [Permitting `proof` to Be Omitted](#911-permitting-proof-to-be-omitted).
* **Error and response delivery.** How an *error response* ([Error Responses](#8-error-responses)) is returned to the *producer* of the original document, including the behavior when the transport is fire-and-forget.
* **Lifecycle mapping.** Which protocol event or status corresponds to each state of [Document Lifecycle](#412-document-lifecycle), or an explicit statement that the protocol expresses no counterpart for a given state. This item is **MUST** for every binding, per the rule stated in that section: a transport status left silently equated to a document state — a `202` to `accepted`, an acknowledged delivery to `executing`, a dropped connection to `cancelled` — reports a document state from a transport fact that does not establish it.

#### 9.1.1 Permitting `proof` to Be Omitted

[When to Include a Proof](#471-when-to-include-a-proof) permits a *Trust Task document* to omit `proof` where the transport already provides end-to-end integrity and authentication between *producer* and *consumer*. Whether a given transport does so is not a property a *consumer* can read off the transport's name: it depends on where the security boundary actually terminates, and on what intermediaries can do to a document in flight.

A *transport binding* that permits a *Trust Task document* to omit `proof` under [When to Include a Proof](#471-when-to-include-a-proof) **MUST** specify the security properties on which that allowance depends. That specification **MUST** address each of the following, and **MUST** state explicitly where an item does not apply to the transport rather than leaving it unaddressed:

1. **The authenticated producer.** Which credential or transport principal is authenticated, and by which mechanism.
2. **The mapping to a VID.** How that principal is deterministically mapped to the *VID* used for the framework's identity comparisons ([Precedence of In-Band over Transport-Derived Identity](#481-precedence-of-in-band-over-transport-derived-identity)).
3. **Audience binding.** How the intended *consumer* is identified or bound, and whether that binding is authenticated or merely asserted.
4. **Integrity across intermediaries.** What protects the document's bytes across every party that handles them.
5. **Re-origination.** Whether any intermediary can modify or re-originate the document without detection by the final *consumer*.
6. **Freshness and replay.** What protection, if any, the transport provides against a document being delivered more than once or out of time.
7. **Key and credential status.** Any revocation, expiry, or key-status assumption the allowance depends on.
8. **Where the guarantee stops.** Any condition — a routing mode, a mediator, a proxy, a store-and-forward queue — under which the transport ceases to provide producer-to-consumer end-to-end protection.

A *transport binding* **MUST NOT** state that `proof` may be omitted solely because an individual transport hop is authenticated, where an intermediary can modify or re-originate the *Trust Task document* without detection by the final *consumer*. Hop authentication tells the *consumer* who handed it the bytes, which is not the same fact as who composed them.

Where a binding's guarantees differ by mode — direct versus routed, nested versus not, mediated versus point-to-point — the allowance **MUST** be stated per mode. A single allowance covering a transport that has both an end-to-end mode and a relayed one overstates the weaker case.

**Silence is not permission.** A *transport binding* that does not address the omission of `proof` **MUST NOT** be read as permitting it, and a *consumer* operating over such a binding evaluates the `proof` requirement from [When to Include a Proof](#471-when-to-include-a-proof) and the *Trust Task specification* alone, as it would over a transport with no binding at all. A binding whose transport does not provide producer-to-consumer end-to-end integrity and authentication **SHOULD** say so plainly; that statement is as useful to an implementer as an allowance, and it is what stops a familiar transport name from being read as a guarantee it does not give.

### 9.2 The Transport Handler

An implementation that exchanges *Trust Task documents* over a given transport **SHOULD** expose its transport-binding logic as a discrete *transport handler* component:

1. On the **producer** side, the handler composes an outbound *Trust Task document*, **MAY** omit `issuer` and `recipient` where the transport will provide authenticated identity for those roles end-to-end (see [Precedence of In-Band over Transport-Derived Identity](#481-precedence-of-in-band-over-transport-derived-identity)), and applies the transport's signing or sealing where the binding integrates it with `proof`.
2. On the **consumer** side, the handler extracts an inbound *Trust Task document* from the transport, applies the [Precedence of In-Band over Transport-Derived Identity](#481-precedence-of-in-band-over-transport-derived-identity) precedence — using in-band `issuer` and `recipient` values where present (cross-checking them against transport-derived identity) and deriving them from the transport only where the in-band member is absent — and surfaces any inconsistencies as validation failures per [Consumer Requirements](#72-consumer-requirements).

The handler boundary lets the framework's validation logic remain transport-agnostic while different transports plug in their own population rules. A DIDComm handler can populate `issuer` from the verified sender DID of the surrounding DIDComm envelope; a TSP handler can do the same from the TSP message authentication; a mutual-TLS HTTPS handler can populate `issuer` from the peer certificate's subject; an unauthenticated transport handler populates nothing, and the framework falls back to the in-band `proof` per [When to Include a Proof](#471-when-to-include-a-proof).

A *transport binding* specification **SHOULD** identify itself by a stable URI and **SHOULD** declare which version of this framework it targets. The framework does not maintain a closed registry of *transport bindings*; new bindings **MAY** be published independently.

### 9.3 Binding Namespace

A *transport binding* published through the framework's registry is identified by a URI in the `/binding/` subtree of the framework's authority:

```
https://trusttasks.org/binding/<slug>/<MAJOR.MINOR>
```

`<slug>` follows the same lowercase, hyphenated grammar as a Trust Task slug ([Type URI](#61-type-uri)); `<MAJOR.MINOR>` follows the version grammar of [Version Scheme](#51-version-scheme). Additional path segments under a binding URI — for example `https://trusttasks.org/binding/didcomm/0.1/envelope` — identify resources internal to the binding's own vocabulary (envelope `type` values, schema URIs, status mappings, and similar). Those segments are defined by the *transport binding* specification, not by this framework.

The `/binding/` subtree and the `/spec/` subtree of [Type URI](#61-type-uri) are **structurally disjoint**. A *Type URI* — the value carried in a *Trust Task document*'s `type` member ([The `type` Member](#44-the-type-member)) — is always rooted at `/spec/<slug>/<MAJOR.MINOR>` and **MUST NOT** be rooted at `/binding/...`. A *consumer* that receives a *Trust Task document* whose `type` is a URI under `/binding/` **MUST** reject it with `malformedRequest` per [Standard Error Codes](#83-standard-error-codes). The Type URI grammar of [Type URI](#61-type-uri) already excludes the `/binding/` path; this rule is stated explicitly so implementers and reviewers can see the namespace boundary at a glance and so that documents which somehow construct a `/binding/...` `type` value have a defined disposition rather than relying on grammar mismatch alone.

A *transport binding* specification published through the registry **SHOULD** live at `bindings/<slug>/<MAJOR.MINOR>/spec.md` in the framework's source tree, paralleling the `specs/<slug>/<MAJOR.MINOR>/` layout for *Trust Task specifications*. The grammar and content requirements for *transport binding* specifications are defined in [What a Transport Binding Specifies](#91-what-a-transport-binding-specifies).

The reservation rule of [Private and Unpublished Trust Task Specifications](#65-private-and-unpublished-trust-task-specifications) — that private specifications **MUST NOT** be served from the `https://trusttasks.org/` authority — applies to private transport bindings equivalently: a private transport binding **MUST** use an authority the publisher controls and **MUST NOT** claim to identify a resource at `https://trusttasks.org/binding/...`.

<a id="11-discovery-and-capability-negotiation"></a>
## 10. Discovery and Capability Negotiation

*This section is normative.*

Two parties about to enter a *Trust Task* exchange often need to negotiate a shared task vocabulary first: a *producer* asks "which *Trust Tasks* are you prepared to act upon?" before committing to send any particular document. The framework supports this with a reserved *Trust Task specification* of its own: `trust-task-discovery`.

The slug `trust-task-discovery` is reserved by [Type URI](#61-type-uri) under the framework's `trust-task-` namespace. Its current published version lives at:

```
https://trusttasks.org/spec/trust-task-discovery/0.1
```

Its registry entry defines the full request/response payload schema and conformance requirements. This section gives the framework-level overview; for the normative definitions of `payload.patterns` semantics, response shape, and conformance, see that registry entry.

<a id="111-request"></a>
### 10.1 Discovery Request

A *discovery request* is a *Trust Task document* whose `type` is `https://trusttasks.org/spec/trust-task-discovery/0.1`. Its `payload` carries an optional list of slug-glob patterns:

```json
{
  "patterns": ["acl/*", "consent/request"]
}
```

When `patterns` is absent or empty, the *responder* treats the query as `["*"]` — return every supported *Trust Task*.

A *discovery request*'s `patterns` list **SHOULD** be bounded: the specification **SHOULD** declare a maximum number of patterns and a maximum length for each, and a *responder* **MAY** reject a request exceeding either with `malformedRequest`. A pattern list is evaluated against every slug the *responder* supports, so an unbounded list is an unbounded amount of matching work bought for one document — and unlike the free-text bound of [Specification Requirements](#73-specification-requirements) item 19, this one is paid by the party that did not choose it. A discoverer that wants everything sends `["*"]`, which costs one comparison.

<a id="112-pattern-grammar"></a>
### 10.2 Pattern Grammar

Patterns are deliberately coarse. The grammar is:

* `"*"` — matches every slug.
* `"<prefix>/*"` — matches every slug whose value starts with the literal `<prefix>/` (e.g. `"acl/*"` matches `acl/grant`, `acl/revoke`, and `acl/grant/sub`).
* `"<slug>"` — exact match.

Wildcards in positions other than as the trailing `/*` of a `<prefix>/*` pattern are **not** interpreted; they match literally. Multiple patterns combine with **OR** semantics: a slug matches the query if it matches at least one pattern.

The grammar omits version filters, recursive globs (`**`), and regex on purpose. Versions are part of the *Type URI* the responder returns; a discoverer that needs to filter on version applies the constraint client-side.

<a id="113-response"></a>
### 10.3 Discovery Response

A *discovery response* is a *Trust Task document* whose `type` is `https://trusttasks.org/spec/trust-task-discovery/0.1#response`. Its `payload` carries the matching subset of *Type URIs* the responder supports:

```json
{
  "supportedTypes": [
    "https://trusttasks.org/spec/acl/grant/0.1",
    "https://trusttasks.org/spec/acl/revoke/0.1",
    "https://trusttasks.org/spec/consent/request/1.0"
  ]
}
```

Each entry is a **bare** *Type URI* — no `#request` or `#response` fragment. A *Type URI*'s presence in `supportedTypes` means the responder handles both directions of that specification's exchange.

A response with `"supportedTypes": []` is conformant and means "I support nothing matching your query."

<a id="114-status-of-the-response"></a>
### 10.4 Status of the Discovery Response

A *discovery response* is **advisory**. A *Type URI*'s presence is a hint that the responder will accept a *Trust Task document* of that type, not a binding commitment: the responder may have revoked support, may apply per-document permissions, or may itself receive a `proofInvalid` or `permissionDenied` at the point of acting on a subsequent request. Every subsequent exchange runs the full [Consumer Requirements](#72-consumer-requirements) pipeline; discovery only narrows what the discoverer chooses to send.

<a id="116-authenticity-of-a-discovery-response"></a>
### 10.5 Authenticity of a Discovery Response

A *discovery response* is advisory as to *content*, but a discoverer acts on it: it narrows what the discoverer chooses to send, and — through the capability annotations described below — can shape what a *producer* puts in the documents it sends next. An advisory document that is acted upon still has to be attributable.

The `trust-task-discovery` specification declares its `proof` requirement **OPTIONAL**, on the rationale that a discovery exchange takes place between parties that "have already authenticated through the transport". That premise does not hold generally, and the framework has since said why it cannot be assumed from a transport's name: [Permitting `proof` to Be Omitted](#911-permitting-proof-to-be-omitted) requires a *transport binding* to establish the point explicitly, and at least one published binding — the HTTPS binding — states in its own security profile that it provides **no** producer-to-consumer end-to-end guarantee. A discovery exchange over such a binding is one between parties that have authenticated nothing.

Accordingly, from this version:

1. The `proof` requirement applicable to the `trust-task-discovery` specification is **RECOMMENDED**, not **OPTIONAL**. This is a framework default under [When to Include a Proof](#471-when-to-include-a-proof), so [Specification Requirements](#73-specification-requirements) item 8 forbids the specification's own declaration being weaker; the registry entry is expected to be re-issued to match.

2. A *discoverer* **MUST NOT** act upon a *discovery response* whose origin it can authenticate neither in-band — from a `proof` resolving to an `issuer` it recognizes — nor from the transport. This mirrors the rule the `trust-task-next-step` specification already imposes on a continuation, and for the same reason: an unauthenticated redirection is indistinguishable from an injected one. "Act upon" here means narrowing the task set the discoverer will send, satisfying an advertised requirement, or recording the response as evidence of what a party supports; a discoverer that cannot authenticate the response **MAY** still discard it, log it, or retry.

3. **A responder's advertised requirements are untrusted input.** The expanded form of a `supportedTypes` entry may carry capability annotations — the `requiredExt` namespace list of [Consumer Requirements](#72-consumer-requirements) being the one the 0.1 specification defines. Such an annotation is a statement by the responder about the responder's own policy. It **MUST NOT** cause a *producer* to attach to a subsequent *Trust Task document* any data it would not otherwise have sent: a *producer* satisfies an advertised requirement only where it independently holds the data, is willing to disclose it to that party, and would have been willing to do so had the requirement never been advertised. A *producer* that treats an advertised requirement as an instruction has handed the choice of what leaves it to whoever answered the query — which, absent rule 2, is whoever answered it *first*.

    A *producer* unwilling or unable to satisfy an advertised requirement simply does not send the task. The alternative reading turns a discovery response into a data-collection instrument that costs an attacker one unauthenticated reply.

<a id="115-privacy-considerations"></a>
### 10.6 Privacy of Discovery Responses

A discovery response leaks information about which specifications the responder implements. Responders that consider their supported task set sensitive **SHOULD** authenticate the discoverer before responding, and **MAY** return a filtered subset of their true capabilities (or no response at all) when the discoverer is unknown or unauthenticated. See the discovery spec's "Privacy considerations" section for additional discussion.

<a id="12-task-control"></a>
## 11. Task Control

*This section is normative.*

Acceptance of a *Trust Task document*, or commencement of execution upon it, **MUST NOT** by itself make the requested work semantically irrevocable. This section defines transport-independent semantics by which a *producer* can withdraw or pause work a *consumer* has already accepted.

The mechanism is a *Trust Task document* like any other, of the framework-reserved specification `trust-task-control` ([Type URI](#61-type-uri)). It is a **request**, not a response: it flows from *producer* to *consumer*, and is therefore not one of the reserved response-type slugs of [Reserved Response-Type Slugs](#86-reserved-response-type-slugs). Its payload, its response variant, and its conformance requirements are defined by its registry entry.

The framework defines three operations: **cancel**, **suspend**, and **resume**. The corresponding operation for a *consumer* that stops work on its own initiative is not a control operation at all — it is an *error response* carrying `cancelled` ([Standard Error Codes](#83-standard-error-codes)), because a *consumer* refusing or abandoning work is already the case [Error Responses](#8-error-responses) covers. The two directions are deliberately not symmetric: only a *producer* sends a control document, and only a *consumer* emits `cancelled`, so that no party and no auditor need infer from a document alone which of them decided.

<a id="121-authorization"></a>
### 11.1 Control Authorization

The *party* identified by the target document's `issuer` is authorized to cancel, suspend, or resume that task **by default**; a *consumer* **MUST NOT** require further authorization evidence from that party. Where the target document carried no in-band `issuer`, the authorized party is the identity authenticated for it under [Precedence of In-Band over Transport-Derived Identity](#481-precedence-of-in-band-over-transport-derived-identity).

Whether a *consumer* honors a control document from **any other** *party* is that *consumer*'s own decision under [Consumer Requirements](#72-consumer-requirements) item 10, evaluated exactly as for any other *Trust Task*. This is a floor, not a ceiling: a *consumer* executing work on behalf of a mandate holder, a supervising principal, or an organization whose agent initiated the task **MAY** recognize that party's authority to stop it, under its own policy and applicable governance framework. The framework does not foreclose that, and a *consumer* that recognizes only the initiator is equally conformant.

A control document **MUST** carry a `proof`, and the [Audience Binding](#482-audience-binding) rule applies to it. A control operation that cannot be attributed is worthless as evidence of withdrawal, and an unattributable one is a denial-of-service vector against another party's work. Membership of a *Trust Ceremony* confers no authority here, exactly as [Membership Is a Claim, Not a Permission](#4114-membership-is-a-claim-not-a-permission) provides generally.

<a id="122-identifying-the-target"></a>
### 11.2 Identifying the Target

A control document **MUST** identify the specific *Trust Task document* to which it applies, by that document's `id`. It **SHOULD** also carry the target's `type`, so that a *consumer* can detect a control document aimed at an `id` it holds under a different *Trust Task specification*, and **SHOULD** carry the same `threadId` as the target so the two correlate within one exchange.

`threadId`, `parentThreadId`, and `ceremony` membership **MUST NOT**, by themselves, identify the controlled task. More than one *Trust Task document* can occur within a single exchange or *enactment*, and an operation naming only the exchange is ambiguous exactly when it matters most — in a flow busy enough to have several tasks in flight.

<a id="123-when-a-control-operation-takes-effect"></a>
### 11.3 When a Control Operation Takes Effect

**A valid, authorized control operation that a *consumer* has received is one of the conditions that [Consumer Requirements](#72-consumer-requirements) item 12 requires it to re-evaluate before each irreversible or externally visible effect.** This is the normative connection between the two mechanisms, and it is stated here explicitly rather than left to be inferred from item 12's general wording.

A *consumer* therefore does not need a separate race protocol. Having received and authorized a cancellation or suspension, it records it; at the next item 12 checkpoint the condition fails, and item 12 already requires that the subsequent effect **MUST NOT** be performed and that partial execution be reported distinguishably from a task that was never begun.

Where an irreversible or externally visible effect has **already** occurred, a *consumer* **MUST NOT** report the task as cleanly cancelled. It reports what occurred, so that the *producer* can determine whether a compensating action is required (see [Control Does Not Roll Back](#114-control-does-not-roll-back)).

**Cancellation is terminal.** A cancelled task **MUST NOT** be resumed, retried, or cancelled again; a *producer* that still wants the work issues a **new** *Trust Task document* with a fresh `id`. This is the same line [Retry Semantics](#84-retry-semantics) draws for error responses, and for the same reason: a document with two contradictory lifecycle states cannot be reasoned about by any party that retains it.

A control document **MAY** arrive before the *Trust Task document* it names — ordinary on asynchronous and store-and-forward transports. A *consumer* **SHOULD** record it against the target `id` and refuse the later-arriving document rather than execute it. The record required by [Consumer Requirements](#72-consumer-requirements) item 11 serves this purpose and is bounded by the same acceptance window; a control document naming an `id` whose window has lapsed has nothing to match, and is reported as such. The item 11 record **MUST** survive cancellation for the remainder of that window, so that a re-delivery of the original document after cancellation is absorbed rather than executed.

<a id="124-control-does-not-roll-back"></a>
### 11.4 Control Does Not Roll Back

Cancellation prevents future effects. It **MUST NOT** be understood to undo effects that have already occurred, and this framework does not require a *consumer* to retain state for the purpose of reversing them.

Many effects are irreversible by construction — [Specification Requirements](#73-specification-requirements) item 13 defines `destructive` in those terms — and for many others the state needed to reverse the effect is precisely the material the task existed to destroy or to disclose. Where an effect can be undone at all, the undoing is a distinct act with its own authority, its own audit trail, and frequently its own *Trust Task specification*; a *Trust Task specification* **MAY** name such a compensating task in its prose, and a *consumer* **MUST NOT** be presumed to perform one automatically.

What the framework requires instead is **information**: the response to a control operation reports which effects were created before the operation took hold, so that the *producer* can decide whether to invoke a compensating task itself.

<a id="125-suspension-and-resumption"></a>
### 11.5 Suspension and Resumption

A suspension **halts further effects while preserving the *consumer*'s current execution state**. It does not return the task to a pre-execution state, and it does not undo work already performed — that would be the rollback [Control Does Not Roll Back](#114-control-does-not-roll-back) declines to require. Resumption continues from the state the *consumer* holds.

A *consumer* **MUST NOT** resume a suspended task after the target document's `expiresAt`. Resumption is a fresh decision to proceed — the acceptance question of [Consumer Requirements](#72-consumer-requirements) item 4, asked a second time — whereas execution already in progress is protected by item 12's deliberate exclusion of expiry from the conditions it re-evaluates. A suspension preserves state; it does not preserve an indefinite right to restart. A *producer* that still wants the work issues a new document.

A control document **MUST NOT** carry an instruction that a suspension resume automatically after an interval of the *producer*'s choosing. Such an interval is a deadline the *producer* cannot calculate — it does not know how long the *consumer*'s work takes — and it lets a *producer* pin *consumer* state for a period of its own election. How long a *consumer* retains a suspended task is that *consumer*'s own policy, bounded by `expiresAt` where present.

<a id="126-notifications-and-the-meaning-of-silence"></a>
### 11.6 Notifications, and the Meaning of Silence

A *party* **SHOULD** notify its counterparty when a task is cancelled or when a suspended task lapses, so that the other side can release whatever state it holds. Such notifications are **fire-and-forget**: no response is expected, and a *consumer* that does not implement task control emits none.

Accordingly: **no *party* may infer the state of a task from the absence of a notification.** A notification may be lost, discarded by an intermediary, or never sent. A *producer* that reads silence as "still running" waits indefinitely; one that reads it as "safely abandoned" and reissues can cause exactly the second consequential effect that [Consumer Requirements](#72-consumer-requirements) item 11 exists to prevent.

The response to a control document is **not** such a notification. It is a reply to a request, its content is relied upon under [Control Does Not Roll Back](#114-control-does-not-roll-back), and it is subject to the ordinary rules of [Request and Response Variants](#441-request-and-response-variants).

<a id="127-transport-level-cancellation-is-not-semantic-cancellation"></a>
### 11.7 Transport-Level Cancellation Is Not Semantic Cancellation

Cancelling a transport operation — abandoning an HTTP request, cancelling a queue delivery, closing a session, discarding an execution handle — terminates *that delivery*. It **MUST NOT** be interpreted as semantic cancellation of the underlying *Trust Task*, unless the applicable *transport binding* carries a framework-defined control operation with equivalent semantics ([Transport Bindings](#9-transport-bindings)).

A *Trust Task document* that has been accepted, queued, forwarded, or handed to a worker survives the connection that delivered it, and may be held by a *consumer* the withdrawing *party* is no longer in contact with. A *consumer* that treats a dropped connection as a withdrawal will stop work the *producer* still wants; a *producer* that treats one as a withdrawal will believe it has stopped work that is still running.

<a id="128-support-is-optional"></a>
### 11.8 Support Is Optional

A *consumer* that does not implement task control rejects a control document with `unsupportedType` ([Standard Error Codes](#83-standard-error-codes)), as it would any other unrecognized `type`. Task control is therefore **best-effort**, and a *producer* **MUST NOT** rely on a cancellation having been honored in the absence of a response confirming it. A *consumer* that does implement it **SHOULD** advertise `trust-task-control` through discovery ([Discovery and Capability Negotiation](#10-discovery-and-capability-negotiation)), so that a *producer* can establish before the wire trip whether the work it is about to request can later be stopped.

<a id="10-security-and-privacy-considerations"></a>
## 12. Security Considerations

*This section is informative.* Future revisions are expected to make portions of it normative as individual Trust Task specifications surface concrete requirements.

A *Trust Task document* carries no inherent transport security. The framework's default rules for when an integrity proof is required of a document are given in [When to Include a Proof](#471-when-to-include-a-proof), and each *Trust Task specification* declares its own requirement under [Specification Requirements](#73-specification-requirements). When `proof` is included, it **MUST** conform to the W3C *Data Integrity* format defined in [VC Data Integrity](https://www.w3.org/TR/vc-data-integrity/) (see [Proof](#47-proof)); implementations select an appropriate cryptographic suite from the W3C-registered set based on the trust requirements agreed by the parties.

<a id="101-cross-recipient-replay"></a>
### 12.1 Cross-Recipient Replay

A *Trust Task document* signed without an in-band `recipient` provides no cryptographic binding between the *producer*'s assertion and the intended audience. An attacker who obtains such a document — from a *consumer*'s storage, an intermediate cache, or an exfiltration — can replay the bytes to a different *consumer*; the proof verifies against the original *producer*'s VID, and a recipient who does not know the *producer*'s out-of-band intent has no signal that the assertion was not made to them. The [Audience Binding](#482-audience-binding) rule is the primary defence: when `proof` is present, `recipient` is also required in-band, and consumers reject any document that violates this rule with `malformedRequest`. *Bearer specifications* ([Bearer Specifications](#483-bearer-specifications)) are the only specifications for which a `proof`-carrying document without an in-band `recipient` is conformant; bearer status is an intentional, normative property of the specification, not a consumer-side flag.

Replay of the same document by the *original* recipient back into the same *consumer* (within transport bounds) is also possible. For a *consequential Trust Task* this is not merely a threat to be mitigated by local caching: [Consumer Requirements](#72-consumer-requirements) item 11 makes duplicate-execution protection a normative consumer requirement, keyed on the document `id` and bounded by the *consumer*'s acceptance window. The rule deliberately does not distinguish a hostile replay from a legitimate transport retry, because at the document layer the two are indistinguishable — the same bytes arriving twice. What matters for interoperability is that the second arrival does not repeat the effect, whichever it was.

Consumers handling assertions whose effect persists between exchanges but whose task is **not** consequential are outside item 11 and **SHOULD** still maintain such a cache for the lifetime of the assertion's relevance.

<a id="102-parser-hardening"></a>
### 12.2 Parser Hardening

A *consumer* deserializing untrusted JSON into a *Trust Task document* is exposed to the standard hazards of unbounded JSON parsing: deeply nested structures can cause stack overflow, large strings or arrays can exhaust memory, and integer overflows can occur on size fields. A *consumer* **SHOULD** bound the body size at the transport layer and **SHOULD** configure a maximum parse depth on the JSON deserializer. The framework does not mandate specific limits because they vary by deployment, but a depth limit of 128 levels and a body-size limit appropriate to the *Trust Task specification*'s payload (typically a few hundred kilobytes) are reasonable defaults.

<a id="103-schema-validation-dos"></a>
### 12.3 Schema-Validation DoS

A *consumer* that validates `payload` values against a JSON Schema obtained dynamically (for example, via [Content Negotiation](#62-content-negotiation) over the network) **MUST** treat the schema as trusted only after authenticating its source. A maliciously-crafted schema can carry `pattern` regular expressions that exhibit catastrophic backtracking on otherwise-innocuous strings, causing the validator to consume unbounded CPU and effectively become a DoS oracle for any *producer* able to choose payload values. Consumers that compile schemas from arbitrary authorities **SHOULD** apply per-validation timeouts.

This consideration does **not** apply when the schema is embedded with the *consumer* at build time (for example, fetched from the registry once at release time, verified against [Stability](#64-stability) immutability, and shipped as part of the consumer's binary). It does apply to dynamic-registry scenarios and to consumers that accept private specifications ([Private and Unpublished Trust Task Specifications](#65-private-and-unpublished-trust-task-specifications)) over a runtime channel.

<a id="104-error-response-identity-leakage"></a>
### 12.4 Error-Response Identity Leakage

A *consumer* emitting an *error response* under [Error Responses](#8-error-responses) treats the error response's `payload.message` as a wire-exposed value. Free-text messages that reveal the *consumer*'s expected transport-authenticated identity, the contested in-band value of a mismatched party, or other consumer-internal state convert each error response into an identity- and reachability-probing oracle for an unauthenticated *producer*. This was guidance in earlier revisions and is now normative for every code, extended codes included: the enumerated prohibitions and the reasoning are in [What a `message` May Not Say](#821-what-a-message-may-not-say), and the corresponding bound on `details` is in [Bounding `details`](#822-bounding-details). The code-specific rule for `identityMismatch` — which also governs who the response is addressed to — remains in [The trust-task-error Specification](#81-the-trust-task-error-specification).

## 13. Privacy Considerations

*This section is informative except where a subsection states otherwise.* Future revisions are expected to make further portions of it normative as individual Trust Task specifications surface concrete requirements.

1. **Personal data is visible to every handler.** Personal data carried in a *Trust Task document* is visible to every *party* that handles the document. Individual *Trust Task specifications* **SHOULD** minimize personal data in their schemas to that strictly necessary to achieve the task's outcome, and **SHOULD** prefer references (e.g. DID URLs) to direct attribute values where the relying party is able to dereference them.
2. **Self-contained documents are durable evidence.** Because *Trust Task documents* are self-contained, a captured document remains evidence of its content after it has been delivered. Producers **SHOULD** consider whether the document's contents are appropriate for indefinite retention by the consumer.
3. **Discovery reveals an implementer's task set.** A discovery response tells the discoverer which *Trust Task specifications* the responder implements. Responders that consider that set sensitive **SHOULD** authenticate the discoverer before responding; see [Privacy of Discovery Responses](#106-privacy-of-discovery-responses).
4. **Error responses can be identity oracles.** Free-text `payload.message` values that echo consumer-internal authentication context leak identity information to an unauthenticated *producer*; see [Error-Response Identity Leakage](#124-error-response-identity-leakage).
5. **The document's own identifiers correlate the parties.** A *Trust Task document* names both parties in the clear and carries handles whose purpose is to join documents to one another. The normative rules that follow bound what those handles may be derived from and how widely the party identifiers may be reused; see [Identifier Correlation and Linkability](#131-identifier-correlation-and-linkability).
6. **Outcome evidence carries the exchange's correlators.** Presenting *outcome evidence* ([Evidence That a Cited Exchange Completed](#494-evidence-that-a-cited-exchange-completed)) discloses the initiating and terminal documents whole — `id`, `threadId`, `parentThreadId`, `issuer` and `recipient` included — to every party that evaluates it, and those handles join the presentation to the exchange and to every other presentation of the same evidence. Whether a composed presentation is unlinkable is therefore not a property of the citing artifact's own proof mechanism; it has to be assessed across the composition, which the Verifiable Trust Infrastructure specification addresses. A party **SHOULD** present outcome evidence only where the relying decision depends on the exchange having completed.

<a id="105-identifier-correlation-and-linkability"></a>
### 13.1 Identifier Correlation and Linkability

*This subsection is normative.*

A *Trust Task document* is, by construction, a record that names both parties in clear and survives delivery. [Top-Level Members](#42-top-level-members) carries `issuer` and `recipient` on the document rather than in *payload*; nearly every published *Trust Task specification* declares both **REQUIRED**; and a majority declare `proof` **REQUIRED** under [Specification Requirements](#73-specification-requirements) item 8. The median document on the wire is therefore a signed, retainable statement that a named party asked a named party to do a named thing at a named instant. Item 2 above observes that such a document is durable evidence of its *content*. This subsection addresses what it is evidence of about the *parties*, which until this version no rule in this framework constrained at all.

1. **Party identifiers SHOULD be relationship-scoped.** The `issuer` and `recipient` of a *Trust Task document* **SHOULD** be *VIDs* scoped to the relationship in which the document is exchanged — *pairwise* identifiers — rather than a single identifier the party presents to every counterparty it deals with. A *Trust Task specification* **MAY** declare a public, cross-relationship identifier for a party where that public identifier is **intrinsic** to what the task asserts — a registry that must be nameable to be resolved, an attesting authority whose statements are worthless unless they can be attributed publicly, a *bearer specification* whose purpose is unspecified consumption ([Bearer Specifications](#483-bearer-specifications)) — and where it does, it **MUST** state in its prose why. Convenience is not intrinsic. A party that presents one identifier to every counterparty makes every document it has ever issued joinable by any two of those counterparties who compare what they hold, and the framework's own rules make that holding likely: [Audience Binding](#482-audience-binding) puts the counterparty's identifier inside the signature, and [When to Include a Proof](#471-when-to-include-a-proof) makes the document worth retaining.

2. **The correlators MUST be freshly minted and unguessable.** `id` ([The `id` Member](#43-the-id-member)), `threadId` ([The `threadId` Member](#49-the-threadid-member)) and `ceremony.enactment` ([The `ceremony` Member](#411-the-ceremony-member)) exist so that documents can be joined to one another — and anything that joins documents for the parties joins them equally for an observer in the middle, for an intermediary, and for any party that retains one of them later. Each of the three **MUST** be a freshly minted value carrying sufficient entropy to be unguessable, and **MUST NOT** be derived — by hashing, encoding, truncation, or any other transformation — from subject data, an account number, a session identifier, a transaction reference, a counter, or any other value that carries meaning outside the document. A *producer* **MUST NOT** carry a `threadId` across exchanges that are not one exchange; `id` is already non-reusable under [The `id` Member](#43-the-id-member) and `enactment` under [The Identifiers Are Orthogonal](#4112-the-identifiers-are-orthogonal).

    A random UUIDv4 satisfies both requirements. A UUIDv5 over a subject identifier satisfies neither — it is stable across every document about that subject, so it *is* the subject identifier under an encoding. A sequential counter satisfies neither and additionally discloses the *producer*'s volume to every counterparty. Deriving a correlator from meaningful data is the specific failure this rule forecloses, because it looks like an implementation convenience and produces an identifier that is joinable outside the exchange forever.

3. **`issuedAt` MAY be coarsened.** Where the *Trust Task specification* does not need sub-minute freshness, a *producer* **MAY** round `issuedAt` down to a coarser granularity — the minute, or the hour — provided the value remains a conforming [RFC 3339](https://www.rfc-editor.org/rfc/rfc3339) timestamp and remains inside the acceptance window a *consumer* applies under [Consumer Requirements](#72-consumer-requirements) item 13. A full-precision timestamp is a fingerprint: two documents bearing unrelated identifiers and the same millisecond were produced by the same process, which is exactly the join the identifier rules above are written to prevent. Coarsening and freshness trade against each other — the coarser the value, the wider the skew tolerance a *consumer* needs in order to accept it — so a specification that expects its producers to coarsen **SHOULD** state by how much.

4. **A ceremony enactment links every counterparty in the flow.** `ceremony.enactment` is shared, by design, across every document of an *enactment*, and the steps of an enactment are typically bilateral exchanges with *different* counterparties ([The Identifiers Are Orthogonal](#4112-the-identifiers-are-orthogonal)). Carrying the member therefore hands each of those counterparties a handle that joins it to all the others, whether or not they were ever intended to learn of one another. That is the right default for a flow whose participants are already mutually visible and the wrong one for a flow whose participants are not. A *ceremony definition* whose participation must not be linkable across steps declares `enactmentPrivacy: blinded` — the mechanism the `trust-ceremony-receipt` registry entry defines against exactly this exposure — and a flow that cannot accept the enumeration **MUST NOT** instead rely on a receipt simply not being handed out: [Bearer Specifications](#483-bearer-specifications) governs audience, not distribution.

## 14. Governance Considerations

*This section is informative.*

This specification defines a document format and a namespace; it deliberately leaves the policy questions that surround them to the governance frameworks under which the parties operate, consistent with the [ToIP Governance Metamodel](https://trustoverip.org/wp-content/uploads/ToIP-Governance-Metamodel-Specification-V1.0-2021-12-21.pdf).

1. The process by which a slug is assigned, by which a *Trust Task specification* enters the registry, and by which its `status` is updated is governed by the registry policy maintained alongside the registry at <https://trusttasks.org/>. That policy is out of scope for this specification; see [Maturity Levels](#53-maturity-levels).
2. The stability commitment of [Stability](#64-stability) is made by the public registry for the specifications it hosts. Publishers of private specifications make an equivalent commitment scoped to their own trust boundary; see [Private and Unpublished Trust Task Specifications](#65-private-and-unpublished-trust-task-specifications).
3. This framework prescribes **no** authorization model. Whether a *producer* may request an outcome, and whether a *consumer* may perform it, is determined by the *consumer*'s own policy and by the trust or governance framework it operates under; see [Consumer Requirements](#72-consumer-requirements) item 10.
4. A *Trust Task specification* **MUST NOT** declare that a task does or does not require consent, human approval, or an authentication step-up. Such policy is not delegable to a specification or to the registry that serves it; see [Specification Requirements](#73-specification-requirements) items 13 and 14.
5. *Transport bindings* are not subject to a closed registry: new bindings **MAY** be published independently of this framework, under the namespace rules of [Binding Namespace](#93-binding-namespace).

## 15. Internationalization Considerations

*This section is informative.*

This specification defines a JSON document format. Two internationalization properties follow from it and are normative elsewhere in this document: a *Trust Task document* **MUST** be encoded as UTF-8 without a byte-order mark ([Encoding](#41-encoding)), and every framework-defined member name and enumerated value is an ASCII lowerCamelCase identifier that is never translated ([Naming Conventions](#410-naming-conventions)).

Human-readable values — the `message` member of an *error response*, and any natural-language field an individual *Trust Task specification* defines in its `payload` — may appear in any language. This framework defines no language-tagging mechanism for them. Guidance on language negotiation and tagging for such values will be completed before this specification advances beyond Working Draft status.

## 16. Accessibility Considerations

*This section is informative.*

This specification defines a data format exchanged between software components rather than a user interface, and places no direct accessibility requirements on implementers. Accessibility guidance for implementations that render *Trust Task documents* — in particular consent prompts, approval dialogs, and the operator-facing surfaces implied by [Specification Requirements](#73-specification-requirements) items 13 and 14 — will be completed before this specification advances beyond Working Draft status.

<a id="3-conformance"></a>
## 17. Conformance

*This section is normative.*

As well as sections marked as informative, all authoring guidelines, diagrams, examples, and notes in this specification are informative. Everything else in this specification is normative.

This specification defines normative requirements, using the keywords defined in [Requirements Language](#2-requirements-language), for the conformance targets below.

### 17.1 Conformance Targets

1. **Conforming Trust Task specification** — a document that satisfies [Trust Task Documents](#4-trust-task-documents), [Versioning](#5-versioning), [Namespace](#6-namespace), and [Specification Requirements](#73-specification-requirements).
2. **Conforming producer** — an implementation that emits *Trust Task documents* satisfying [Producer Requirements](#71-producer-requirements).
3. **Conforming consumer** — an implementation that processes *Trust Task documents* satisfying [Consumer Requirements](#72-consumer-requirements).

A *producer* or *consumer* that satisfies these requirements against a private, unpublished *Trust Task specification* is a *conforming producer* or *conforming consumer* of that specification, exactly as it would be for a registry-published one; see [Private and Unpublished Trust Task Specifications](#65-private-and-unpublished-trust-task-specifications).

### 17.2 Conformance Tests

A conformance test suite for this framework has not yet been published. The registry repository at <https://github.com/trustoverip/dtgwg-trust-tasks-tf> carries executable conformance material that is expected to form its basis: the framework envelope JSON Schema for each framework version, the payload schema of every registered *Trust Task specification*, and the generated Rust and TypeScript client libraries whose validation pipelines implement [Consumer Requirements](#72-consumer-requirements). A normative test suite will be defined before this specification advances beyond Working Draft status.

<a id="13-references"></a>
## 18. References

*This section is informative.*

<a id="131-normative-references"></a>
### 18.1 Normative References

- [IETF RFC 2119: Key words for use in RFCs to Indicate Requirement Levels](https://www.rfc-editor.org/rfc/rfc2119)
- [IETF RFC 8174: Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words](https://www.rfc-editor.org/rfc/rfc8174)
- [IETF RFC 3339: Date and Time on the Internet: Timestamps](https://www.rfc-editor.org/rfc/rfc3339)
- [IETF RFC 3986: Uniform Resource Identifier (URI): Generic Syntax](https://www.rfc-editor.org/rfc/rfc3986)
- [IETF RFC 8259: The JavaScript Object Notation (JSON) Data Interchange Format](https://www.rfc-editor.org/rfc/rfc8259)
- [IETF RFC 9110: HTTP Semantics](https://www.rfc-editor.org/rfc/rfc9110)
- [IETF RFC 8785: JSON Canonicalization Scheme (JCS)](https://www.rfc-editor.org/rfc/rfc8785)
- [IETF RFC 9562: Universally Unique IDentifiers (UUIDs)](https://www.rfc-editor.org/rfc/rfc9562)
- [IETF RFC 5234: Augmented BNF for Syntax Specifications (ABNF)](https://www.rfc-editor.org/rfc/rfc5234)
- [W3C Verifiable Credential Data Integrity 1.0](https://www.w3.org/TR/vc-data-integrity/)
- [JSON Schema: A Media Type for Describing JSON Documents, Draft 2020-12](https://json-schema.org/draft/2020-12/schema)
- [W3C Decentralized Identifiers (DIDs) v1.0](https://www.w3.org/TR/did-core/)
- [Semantic Versioning 2.0.0](https://semver.org/spec/v2.0.0.html)

<a id="132-informative-references"></a>
### 18.2 Informative References

- [W3C Verifiable Credentials Data Model v2.0](https://www.w3.org/TR/vc-data-model-2.0/)
- [W3C Manual of Style](https://www.w3.org/guide/manual-of-style/)
- [ToIP Governance Metamodel Specification V1.0](https://trustoverip.org/wp-content/uploads/ToIP-Governance-Metamodel-Specification-V1.0-2021-12-21.pdf)
- [Trust Tasks registry](https://trusttasks.org/) — the published *Trust Task specifications*, *transport bindings*, and *ceremony definitions* that conform to this framework
- [Trust Tasks registry repository](https://github.com/trustoverip/dtgwg-trust-tasks-tf) — the source of the registry, the transport bindings, and the generated client libraries
- [Decentralized Trust Graph Credentials — Core Specification](https://github.com/trustoverip/dtgwg-cred-spec) — the companion DTGWG credential specification whose `taskContext` binding cites Trust Task exchanges
- [Verifiable Trust Infrastructure (VTI) Specification](https://github.com/trustoverip/dtgwg-vti-spec) — the companion DTGWG system specification whose composition requirements govern what a relying party may conclude from a credential together with the outcome evidence of the exchange it cites

## Appendix A — Example Trust Task Specification

*This appendix is informative.*

This appendix shows the elements an individual *Trust Task specification* declares in order to satisfy [Specification Requirements](#73-specification-requirements). The declarations below are reproduced from the registered `acl/change-role` 0.1 specification, so every element has a live counterpart that can be dereferenced; that registry entry, not this appendix, is normative for the task itself.

### A.1 Front Matter

| Declaration | Value |
|---|---|
| Slug | `acl/change-role` |
| Version | `0.1` |
| *Type URI* | `https://trusttasks.org/spec/acl/change-role/0.1` |
| Target framework version | `0.5.0` |
| Maturity level | `draft` |
| `issuer` party | The changing authority. **REQUIRED**. No *VID* scheme restriction is declared. |
| `recipient` party | The ACL maintainer. **REQUIRED**. No *VID* scheme restriction is declared. |
| Outcome | The *issuer* records to the *recipient* the transition of a subject's role within an access-control list, subject to an optimistic concurrency check against the subject's prior role. |
| Proof requirement | **REQUIRED**. Rationale: role changes are the highest-impact ACL operation — a promotion extends privilege, a demotion withdraws it — so a non-repudiable, transport-independent record is necessary for audit, dispute resolution, and downstream parties that retained the prior grant (see [When to Include a Proof](#471-when-to-include-a-proof)). |
| `issuedAt` requirement | **REQUIRED**. Rationale: a role change overwrites the entry rather than incrementing it, so a stale copy applied out of order silently reinstates a role an operator has already moved the subject off; the issue time is what lets the maintainer order two changes to the same entry and refuse the older one. |
| Side effects | `mutating` — reassigns a subject's role in the ACL; recoverable by changing it back. |
| Exposure | `discloses: none`. The specification does not act as the subject. |
| Subject path | `/subject` |
| JSON-LD `@context` | Not published at this version. |

### A.2 Payload JSON Schema

Served at the *Type URI* under content negotiation for `application/schema+json`. The `$defs.Response` sub-schema is the payload of the `#response` variant defined in [Request and Response Variants](#441-request-and-response-variants); `$ref` values are relative to the schema's own position in the registry.

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://trusttasks.org/spec/acl/change-role/0.1",
  "title": "ACL Change Role — payload",
  "type": "object",
  "additionalProperties": false,
  "required": ["subject", "fromRole", "toRole"],
  "properties": {
    "subject": {
      "type": "string",
      "description": "VID of the party whose role is changing."
    },
    "fromRole": {
      "type": "string",
      "minLength": 1,
      "description": "The subject's prior role. Used by the maintainer for the optimistic concurrency check."
    },
    "toRole": {
      "type": "string",
      "minLength": 1,
      "description": "The role to transition the subject to."
    },
    "reason": {
      "type": "string",
      "maxLength": 1024,
      "description": "Optional human-readable rationale."
    },
    "ext": {
      "$ref": "../../../_framework/0.1/framework.schema.json#/$defs/Ext",
      "description": "Ecosystem-defined extension members per SPEC.md §4.5.1."
    }
  },
  "$defs": {
    "Response": {
      "$anchor": "response",
      "title": "ACL Change Role — response payload",
      "description": "The success response to an acl/change-role request. Carried in a Trust Task document whose type is https://trusttasks.org/spec/acl/change-role/0.1#response.",
      "type": "object",
      "additionalProperties": false,
      "required": ["entry"],
      "properties": {
        "entry": {
          "description": "The AclEntry the maintainer now holds for the subject. entry.role MUST equal the request's payload.toRole.",
          "$ref": "../../_shared/0.1/acl-entry.schema.json#/$defs/AclEntry"
        },
        "ext": {
          "$ref": "../../../_framework/0.1/framework.schema.json#/$defs/Ext",
          "description": "Ecosystem-defined extension members per SPEC.md §4.5.1."
        }
      }
    }
  }
}
```

### A.3 Task-Specific Error Codes

| Code | Meaning | Default `retryable` | `details` shape |
|---|---|---|---|
| `acl/change-role:roleNotRecognized` | The `fromRole` or `toRole` string is not part of the ACL maintainer's role vocabulary. | `false` | `{ "offendingRole": <string>, "knownRoles": <array of string> }` |
| `acl/change-role:stateMismatch` | The subject's current role does not match `payload.fromRole`; the change was based on stale state. | `true` | `{ "currentRole": <string> }` |

Both codes are namespaced under the emitting specification's own slug, per rule 1 of [Extension by Individual Trust Task Specifications](#85-extension-by-individual-trust-task-specifications). The `details` JSON Schema fragment for `acl/change-role:roleNotRecognized` is:

```json
{
  "type": "object",
  "additionalProperties": false,
  "properties": {
    "offendingRole": { "type": "string" },
    "knownRoles": {
      "type": "array",
      "items": { "type": "string" }
    }
  }
}
```

### A.4 An Example Conforming Document

```json
{
  "id": "1b3c5e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/acl/change-role/0.1",
  "issuer": "did:web:org.example",
  "recipient": "did:web:maintainer.example",
  "issuedAt": "2026-06-10T14:00:00Z",
  "payload": {
    "subject": "did:web:bob.example",
    "fromRole": "member",
    "toRole": "moderator",
    "reason": "Promoted after six months of community contributions."
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-rdfc-2022",
    "verificationMethod": "did:web:org.example#key-1",
    "created": "2026-06-10T14:00:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z5xy..."
  }
}
```

This document carries a `proof` member because the specification declares `proof` as **REQUIRED** in [Front Matter](#a1-front-matter). A *consumer*:

1. Resolves the document's `type` URI to learn the *target framework version* (`0.5.0`) and fetches the framework schema at `https://trusttasks.org/spec/trust-task/0.5.0`. The outer document structure is validated against it.
2. Fetches the payload schema at the same `type` URI under content negotiation for `application/schema+json`. The `payload` is validated against it.
3. Verifies the `proof` per [Proof](#47-proof) against the *VID* in `issuer`.
4. Confirms `recipient` matches the consumer's own *VID*, and that `issuedAt` does not lie in the consumer's own future beyond its skew tolerance. This document declares no `expiresAt`; where one is present it is checked here too.
5. Applies the checks the specification adds on top of the framework's — for `acl/change-role`, that the subject's current role equals `payload.fromRole`, failing which the *consumer* answers `acl/change-role:stateMismatch`. The framework knows nothing of this check; it is the part [Specification Requirements](#73-specification-requirements) obliges each specification to state for itself.

If any step fails, the *consumer* returns an *error response* per [Error Responses](#8-error-responses).

## Appendix B — Changelog

*This appendix is informative.*

<a id="06"></a>
### B.1 Framework version 0.6.0

This revision is **additive**. The document wire format gains no member and loses none, and every document conforming to 0.5.0 still conforms. It adds one new obligation on *consumers*, and one on *Trust Task specifications*. Each applies only where an exchange is cited from outside the framework as evidence that it completed.

* **Evidence that a cited exchange completed ([Evidence That a Cited Exchange Completed](#494-evidence-that-a-cited-exchange-completed)).** [Binding a Citation to the Document It Names](#493-binding-a-citation-to-the-document-it-names) fixed *which* exchange a citation names, and said nothing about whether that exchange **completed**. A credential citing a cancelled, failed or still-open exchange binds as well as one citing a completed exchange. The completion rule lived instead in individual *Trust Task specifications* (`witness/session/submit/0.1`, `vetting/session/0.1`) and in the companion credential specification, and none of those versions generalised. *Outcome evidence* is now defined once. It is the initiating document together with a terminal success document, which a *consumer* relying on a citation as evidence of completion **MUST** pair by four checks:
    1. the citation binds the initiating document;
    2. the terminal document belongs to that exchange, by `threadId` routed through the initiating document **and** by the binding its governing specification declares;
    3. its `type` is the *declared* outcome-evidence response;
    4. its `proof` verifies under the initiating document's `recipient`.

    Each check closes a case a weaker rule decides wrongly: a minted `threadId`, a cancelled task's control `#response`, a response signed by the holder, and a `threadId` reused across exchanges. All four were exercised against real Trust Task documents before release.

* **Declaring outcome evidence ([Specification Requirements](#73-specification-requirements) item 20).** A *Trust Task specification* whose exchanges may be cited as evidence of completion declares which success response is its outcome evidence. For that response it defines a payload schema, requires `proof` and `issuedAt`, and **SHOULD** declare `durable` retention. It declares the payload members that bind the response to the initiating document: the initiating document's `id` and task digest, or, only where the initiator must also issue the citing artifact, a fresh value such as a challenge. It requires the initiating document to carry `recipient`. The declaration is made by the specification governing the **initiating** document, because an exchange may close with the response to a task conducted on its thread under another specification. A specification that declares nothing is not thereby non-conforming, but its exchanges cannot be cited as evidence of completion.

* **Outcome evidence carries the exchange's correlators ([Privacy Considerations](#13-privacy-considerations) item 6).** Presenting outcome evidence discloses whole Trust Task documents, including their `id`, `threadId` and both parties' identifiers, whatever the citing artifact's own proof mechanism. Whether a composed presentation is unlinkable is therefore assessed across the composition. The Verifiable Trust Infrastructure specification is added as an informative reference for that.

* **Companion changes in the registry.** The registry already carries the declarations for its two cited exchanges, `witness/session/0.1` and `vetting/session/0.1`, through an `outcomeEvidence` front-matter key that the registry build checks against item 20. What remains is to publish a 0.6 framework envelope schema, identical in shape to 0.5's, so that a specification can declare 0.6.0 as its target framework version.

<a id="05"></a>
### B.2 Framework version 0.5.0

This revision is **additive**, with one deprecation. The document wire format gains no member and loses none; every document conforming to 0.4.0 still conforms. Three new front-matter declarations are **SHOULD**, each with a fail-safe reading of an absent value, so no already-published *Trust Task specification* becomes non-conforming for want of them.

* **Identifier correlation and linkability ([Identifier Correlation and Linkability](#131-identifier-correlation-and-linkability)).** The framework had, until this version, said nothing about what its own identifiers disclose about the parties. A census of the registry settles that this is not a hypothetical: 346 of 349 published specifications declare both `issuer` and `recipient` **REQUIRED**, and 210 declare `proof` **REQUIRED**, so the median *Trust Task document* is a signed, retainable record naming both parties in clear. [Privacy Considerations](#13-privacy-considerations) item 2 already observed that such a document is durable evidence of its *content*; nothing addressed what it is evidence of about the *parties*.

    Four rules, now normative. `issuer` and `recipient` **SHOULD** be relationship-scoped (pairwise) *VIDs* unless the specification states why a public identifier is intrinsic to what the task asserts. The three correlation handles — `id`, `threadId`, `ceremony.enactment` — **MUST** be freshly minted, unguessable, and **MUST NOT** be derived from subject data, an account number, a session identifier, or a counter; a UUIDv5 over a subject identifier is the subject identifier under an encoding, and a sequential counter additionally discloses volume. `issuedAt` **MAY** be coarsened where the specification does not need sub-minute freshness, because a full-precision timestamp is a fingerprint. And a `ceremony.enactment` links every counterparty in a flow by default — a *ceremony definition* whose participation must not be linkable declares `enactmentPrivacy: blinded` rather than relying on a receipt not being handed out.

* **One document lifecycle ([Document Lifecycle](#412-document-lifecycle)).** The lifecycle a *consumer* follows was implied in five places — the acceptance bound of [Consumer Requirements](#72-consumer-requirements) item 4, the duplicate record of item 11, the re-evaluation of item 12, [Task Control](#11-task-control), and [Error Responses](#8-error-responses) — and no section named the whole set, so implementations inferred five overlapping lifecycles from one document. A normative state table now names them once (`received`, `validated`, `accepted`, `executing`, `suspended`, `responded`, `errored`, `cancelled`, `expired`), together with the reserved reply that carries each transition.

    **Silence had acquired four meanings** — success for a fire-and-forget task, nothing for `trust-task-ok`, "duplicate absorbed" under item 11, and nothing again for control notifications — and the document's own text already warned that the silence of item 11 "must not be reused to signify work half-done". The table settles on the only safe reading: the absence of a reply distinguishes no two states, and a *producer* **MUST NOT** infer any state from it. A *transport binding* **MUST** map the table onto its own protocol, so that a `202` is not silently equated with acceptance, an acknowledged delivery with execution, or a dropped connection with cancellation.

* **Freshness ([Consumer Requirements](#72-consumer-requirements) item 13, [Specification Requirements](#73-specification-requirements) item 17).** The duplicate-execution record of item 11 is implementable only inside a bounded window, and nothing obliged a document to carry a timestamp that could place it in one — so a document with no `expiresAt` and no usable `issuedAt` never leaves the set a *consumer* must recognize, and its record has to be kept forever. A *consumer* **MUST** now reject a document whose `issuedAt` is in its own future beyond its skew tolerance, and one whose `expiresAt` is at or before its `issuedAt`. Both are `malformedRequest` rather than `expired`, because `expired` names a document that was once acceptable and neither of these ever was. A *Trust Task specification* defining a *consequential Trust Task* **MUST** require `issuedAt`, which puts the obligation on the party that can satisfy it for nothing.

* **Retention, ingested data, and identifier scope ([Specification Requirements](#73-specification-requirements) items 5, 14, and 18).** Three new front-matter declarations, each **descriptive, not prescriptive** on the exact terms of items 13 and 14: none may be read as obliging a *consumer* to do anything, none may carry a consent requirement, and each has a fail-safe reading of an absent or unresolvable value.

    `retention` (`class` of `transient`, `exchange`, or `durable`, plus a `rationale`) gives the framework a retention vocabulary it has never had. The gap was not neutral: every rule in this document and in the published corpus that uses the word *retain* mandates **more** retention — a proof so a document can be relied on later, a digest so a duplicate can be absorbed, an `inResponseTo` so an error can be evidence — and none bounds it, so the safe reading of every task has been to keep everything. Absent or unresolvable reads as `durable`.

    `exposure.ingests` (`none`, `metadata`, `personal`, `secret`) closes a blind spot in the exposure class: `discloses` is response-only, so a task whose request carries a full verifiable presentation, a document image, or free-text a person typed declares `discloses: none` entirely correctly and says nothing about the most sensitive data in the exchange. Absent or unresolvable reads as no less sensitive than `personal`.

    `parties[].identifierScope` (`pairwise`, `public`, `any`) makes the first rule of [Identifier Correlation and Linkability](#131-identifier-correlation-and-linkability) machine-readable, so a reviewer or a registry can see which specifications expect a public identifier without reading each one for the justification. Absent or unresolvable reads as no less correlatable than `public`.

    These three need front-matter meta-schema support in the registry repository before they can be declared; see the note at the end of this entry.

* **Free-text members are bounded ([Specification Requirements](#73-specification-requirements) item 19).** A free-text member is the one place in a *Trust Task document* where the schema constrains the shape and nothing constrains the content — so it is where personal data arrives in a task declaring it ingests none, where a secret arrives pasted by someone asked for a reason, and where instructions addressed to a downstream reader arrive in a field the specification took for a comment. It is also unbounded wire cost: [Parser Hardening](#122-parser-hardening) puts the limit at the transport layer, which is the right defence and the wrong place to pick the number, since one figure there covers every task the *consumer* implements.

    Any free-text member **MUST** declare a `maxLength`, **SHOULD** be **OPTIONAL**, and the specification **MUST** state who reads the value, whether it is retained, and whether it is trusted. A closed enumeration plus one bounded, optional note is preferred to free text carrying the meaning the enumeration should have carried. `task-consent/request/0.1`'s `note` is the pattern: 500 characters, optional, attributed to its author on every surface, declared explicitly untrusted, and forbidden from displacing the effects it appears beside.

* **A fire-and-forget task can be acknowledged, and `trust-task-ok` is deprecated ([Acknowledging a Fire-and-Forget Task](#442-acknowledging-a-fire-and-forget-task), [Reserved Response-Type Slugs](#86-reserved-response-type-slugs)).** Forbidding a fire-and-forget specification from emitting a `#response` is the sole reason `trust-task-ok` exists — and that slug is defined so that it cannot carry meaning: a *consumer* **MUST NOT** require any member of it, a *producer* **MUST NOT** rely on receiving one. A registry entry, a schema, and a generated type in every supported language, on a `type` bearing no relation to the one the *producer* sent, to convey "something arrived".

    A fire-and-forget specification's *consumer* **MAY** now return `<type>#response` with `payload` exactly `{}`, and a *producer* still **MUST NOT** rely on it. `trust-task-ok` is **deprecated** in its favour: a *consumer* **SHOULD NOT** emit one from this version, a *producer* **MUST** continue to accept one for as long as the specification is served, and the slug stays **RESERVED** permanently. Retirement of the registry entry is a separate change in the registry repository and is not effected by this revision.

    [Request and Response Variants](#441-request-and-response-variants) also gains a table of the **three response dispositions** — success `#response`, failure `trust-task-error`, continuation `trust-task-next-step` — beside the fragment rule. "Is this document a response?" was answerable in three separate places, so an implementation that found one of them hard-coded a list of the slugs it knew, which the next reserved slug silently invalidates.

* **Discovery hardening ([Authenticity of a Discovery Response](#105-authenticity-of-a-discovery-response), [Discovery Request](#101-discovery-request)).** The discovery specification justifies its **OPTIONAL** `proof` requirement on the ground that the parties "have already authenticated through the transport". That is false over the HTTPS binding, whose own security profile states plainly that it provides no producer-to-consumer end-to-end guarantee — and [Permitting `proof` to Be Omitted](#911-permitting-proof-to-be-omitted) already refuses to let a guarantee be read off a transport's name.

    The applicable `proof` requirement rises to **RECOMMENDED**, and a *discoverer* **MUST NOT** act on a *discovery response* whose origin it can authenticate neither in-band nor from the transport — the rule `trust-task-next-step` already imposes on a continuation, for the identical reason: an unauthenticated redirection is indistinguishable from an injected one. A responder's advertised requirements, such as a `requiredExt` namespace, are **untrusted input** and **MUST NOT** cause a *producer* to attach data it would not otherwise have sent; the contrary reading turns a discovery response into a data-collection instrument that costs an attacker one unauthenticated reply. A request's `patterns` list **SHOULD** be bounded, since the matching work is paid by the party that did not choose it.

* **The error model states what may not be said, and bounds what may ([What a `message` May Not Say](#821-what-a-message-may-not-say), [Bounding `details`](#822-bounding-details), [Effects Reported with `cancelled`](#831-effects-reported-with-cancelled)).** The message-sanitization rule was normative for `identityMismatch` alone and a **SHOULD** in an informative section for everything else — yet every rejection is emitted on the same path, to the same possibly-unauthenticated party, generally before any authorization decision has been reached. It is now normative for **every** code: a `message` **MUST NOT** reveal consumer-internal state, the contested value of a mismatched party, or resolver, verifier, or key-status internals. Otherwise each error response is an identity- and reachability-probing oracle that the *consumer* pays for and operates on the sender's behalf.

    `details` was the one error-payload member with no size bound, travelling in the direction no *producer*-side bound reaches. A specification defining a `details` shape **MUST** now declare a bound; where none is declared, 4096 bytes of JCS or 16 immediate members apply. An oversized `details` is ignored — never grounds to discard the `code`, which is what the receiving party actually needs.

    The `cancelled` code gains a framework-defined `details` shape: an `effects` array, deliberately the same array-of-effects that `trust-task-control` already defines for the response to a *producer*-requested stop. The two report the identical fact — what had already landed when the stop took hold — and differed only in that one was machine-readable and the other prose, according to which party decided. `effects: []` means nothing landed; an absent `effects` means the *consumer* did not report, and **MUST NOT** be read as "nothing landed".

* **Two digests, named ([Binding a Citation to the Document It Names](#493-binding-a-citation-to-the-document-it-names)).** `taskDigest` excludes the top-level `proof` and asks *what the document says*; `stepDigest` includes it and asks *which serialization arrived*. Both have been carried in members called `digestMultibase`, so an implementation with a single function of that name has in practice picked one answer and applies it to both questions — and whichever it picked is wrong for one of them. A short table now states the difference beside the citation rule that turns on it. Clarification only: neither is a document member, no computation changes, and every existing citation and step chain remains conforming.

* **Companion changes required in the registry.** Two items in this revision are not complete until the registry repository moves. The three new front-matter declarations — `retention`, `exposure.ingests`, and `parties[].identifierScope` — need support in `specs/spec.meta.schema.json` before a specification can declare them; until then they are declarable in prose only. And the deprecation of `trust-task-ok` is a statement of this framework's position: retiring the registry entry itself follows [Maturity Levels](#53-maturity-levels) and belongs to the registry's own change process.

<a id="04"></a>
### B.3 Framework version 0.4.0

* **The `ceremony` member ([The `ceremony` Member](#411-the-ceremony-member)).** A *Trust Task document* **MAY** now record that it is one step of a *Trust Ceremony* — a flow composed of several *Trust Tasks*. The framework has always modelled multi-party work as multiple bilateral tasks ([Terminology](#3-terminology)); what it lacked was a way for the collection to be named, identified, and evidenced, so every implementation held that knowledge in application code and no two could interoperate above the level of a single task. The member carries the *enactment* (globally unique and non-reusable, unlike `threadId`, because evidence about a flow needs a stable anchor), the step's name, an optional content-pinned reference to a published *ceremony definition*, and an optional set of predecessor digests.

    Three properties are deliberate. It is **carried on the document rather than in `payload`**, so no *Trust Task specification* changes and any existing task may be composed into a flow its author never anticipated. It is **covered by `proof`**, so a step cannot be lifted into a different enactment or reinterpreted under a different definition. And it **confers no authority** ([Membership Is a Claim, Not a Permission](#4114-membership-is-a-claim-not-a-permission), [Consumer Requirements](#72-consumer-requirements) item 9) — membership is an assertion by the *issuer*, not a verified fact, which is what makes it safe for a *consumer* to ignore the member entirely.

    Additive: the document wire format gains an optional member, and every document conforming to 0.3 still conforms.

* **The `/ceremony/` subtree and the `trust-ceremony` reservation ([Ceremony Namespace](#67-ceremony-namespace), [Type URI](#61-type-uri)).** *Ceremony definitions* are identified in a third subtree under the framework's authority, structurally disjoint from `/spec/` and `/binding/` on the same terms — no URI under it is a *Type URI*, and a document whose `type` is rooted there is malformed. The slug reservation of [Type URI](#61-type-uri) widens from `^trust-task($|-|/)` to `^trust-(task|ceremony)($|-|/)`; the new half is unused at this version and exists so the namespace cannot be claimed by another party before the layer that needs it is specified.

    The **content** of a ceremony definition is out of scope for this revision. This version defines where definitions live, how a step references one, and that the reference is by content as well as by name — a URI alone would leave a flow's rules mutable by whoever controls the URI, retroactively and for every enactment already performed.

* **Authorization is distinct from identity and proof ([Consumer Requirements](#72-consumer-requirements) item 10, [Specification Requirements](#73-specification-requirements) item 15).** A *consumer* **MUST NOT** treat successful validation of a *VID*, `issuer`, `recipient`, transport-derived identity, or `proof` as establishing that anyone is authorized to request or perform the task, and **MUST** evaluate authorization separately before executing. The framework already said this twice in narrow forms — ceremony membership grants nothing ([Membership Is a Claim, Not a Permission](#4114-membership-is-a-claim-not-a-permission), [Consumer Requirements](#72-consumer-requirements) item 9), and the side-effect and exposure classes describe without authorizing ([Specification Requirements](#73-specification-requirements) items 13 and 14) — but never for an ordinary task, leaving an implementer free to read *valid proof + recognized issuer + correct recipient* as an authorized instruction. That inference is the same confused-deputy vector [Membership Is a Claim, Not a Permission](#4114-membership-is-a-claim-not-a-permission) forecloses, and it is most dangerous where the *producer* is an agent that can prove its identity but holds no authority to act.

    The rule is deliberately **model-neutral**: it requires that an authorization decision be made, not how. A verified assertion may still *be* the authorization where a specification defines that role and the *consumer*'s policy accepts it — the `task-consent` design — but that is now an explicit declaration under item 15 rather than an available default.

    Additive: the wire format is unchanged and every document conforming to 0.3 still conforms. A *consumer* that already separated authorization from validation needs no change.

* **`trust-task-ok` published ([Reserved Response-Type Slugs](#86-reserved-response-type-slugs)).** The success acknowledgement reserved since 0.1 now has a registry entry, and the four revisions it spent unspecified are the reason it is narrower than its original description implied. A *consumer* **MAY** return one to confirm that it received and performed a task whose specification defines no success response of its own; it **MUST NOT** be sent in place of a response a specification does define, because two success dispositions for one task leave a *producer* unable to tell which is authoritative.

    **Its weakness is the design.** A *producer* **MUST NOT** rely on receiving one, and **the absence of one carries no information** — a *consumer* may not implement it, may implement it and stay silent, or the document may be lost. That rule is what keeps the specification safe to add at all: a *producer* that read absence as failure and reissued a *consequential Trust Task* would cause exactly the duplicate effect [Consumer Requirements](#72-consumer-requirements) item 11 exists to prevent, and it also leaves item 11's own use of silence — an absorbed duplicate of a fire-and-forget task — unambiguous.

    Accordingly an acknowledgement that genuinely matters is **not** this document. A task whose acknowledgement will be relied on, audited, or disputed declares its own success response, or a dedicated receipt task with its own proof requirement — the choice `chat/message/0.1` already made deliberately, so that its acknowledgement is a signed link in a chain rather than a transport-level ack.

    Additive: the wire format is unchanged, and a *consumer* that ignores acknowledgements loses nothing it was entitled to.

* **Task control ([Task Control](#11-task-control), [Type URI](#61-type-uri), [Standard Error Codes](#83-standard-error-codes)).** A *producer* can now withdraw or pause work a *consumer* has already accepted. The framework could express what should happen next — `parentThreadId`, ceremonies, `trust-task-next-step` — but nothing let a request be taken back, which for long-running and agentic execution is a **corrigibility** gap: an agent could be told to start and had no defined way to be told to stop. Transport-level cancellation is not a substitute, because a document that has been accepted, queued, or forwarded survives the connection that delivered it.

    The mechanism is deliberately small, because three rules added earlier in this revision already do most of the work. **[Consumer Requirements](#72-consumer-requirements) item 12 is where a control operation takes effect** — a received, authorized operation is one of the conditions it re-evaluates before each irreversible effect, and [When a Control Operation Takes Effect](#113-when-a-control-operation-takes-effect) says so explicitly rather than leaving it to be inferred. Item 12 already requires that the subsequent effect not be performed and that partial execution be reported distinguishably, so no separate race protocol was needed. Item 11's per-`id` record serves as the tombstone for a control document that arrives before the task it names, bounded by the same acceptance window. Item 10 settles who may ask.

    **Authorization is a floor, not a ceiling.** The target's `issuer` is authorized by default; whether a *consumer* honors any other party is its own decision under item 10. An absolute rule would have foreclosed the mandate holder and supervising principal — the delegated execution this framework exists to support — at framework level, where every other authorization decision is the *consumer*'s.

    **Cancellation prevents future effects and never undoes past ones** ([Control Does Not Roll Back](#114-control-does-not-roll-back)). Many effects are irreversible by construction, and the state needed to reverse one is frequently the material the task existed to destroy: retaining a superseded private key so a rotation could be rolled back would defeat the rotation. What the framework requires instead is information — the response reports what occurred, so the *producer* can invoke a compensating task itself.

    **Suspension halts further effects while preserving execution state** ([Suspension and Resumption](#115-suspension-and-resumption)); it does not rewind the task, which would be the rollback [Control Does Not Roll Back](#114-control-does-not-roll-back) declines to require. A *consumer* **MUST NOT** resume after `expiresAt`, because resumption is the acceptance question of item 4 asked a second time — while execution already under way stays protected by item 12's exclusion of expiry.

    **Silence carries no information** ([Notifications, and the Meaning of Silence](#116-notifications-and-the-meaning-of-silence)). Notifications are fire-and-forget and a *consumer* need not implement control at all, so a *producer* that reads silence as "safely abandoned" and reissues can cause the second consequential effect item 11 exists to prevent.

    Additive: the wire format is unchanged for every existing task, and a *consumer* that does not implement control rejects the new type as it would any other it does not recognize.

* **`cancelled` ([Standard Error Codes](#83-standard-error-codes)).** A new standard error code for a *consumer* that stops a task on its own initiative — operator action, policy, capacity, a compliance hold. Named for what happened rather than who caused it, since an operator is one reason among several. It is distinct from a *producer*-requested cancellation, which is answered by a response to the control document: without the distinction, no party and no auditor reading the retained documents could tell a withdrawal from a refusal, and the two imply opposite things about whether to try again. Carried by `trust-task-error/0.5`.

* **Validity during execution ([Consumer Requirements](#72-consumer-requirements) item 12, [Specification Requirements](#73-specification-requirements) item 16, [Top-Level Members](#42-top-level-members)).** Validating a document established that it was eligible for processing *at the instant it was validated*. For execution that is delayed, long-running, resumed, or agentic, that instant and the instant a consequential effect actually lands can be far apart — and the authority in between can evaporate. A *consumer* **MUST** now re-evaluate, immediately before each irreversible or externally visible effect, every condition its policy and the *Trust Task specification* require: delegation, mandate, capability, membership, standing, credential or key status, subject relationship. `task-consent/decision/0.1` already required this locally, re-checking policy and approver enrolment so a device revoked during the approval window cannot carry a task through; item 12 makes the general case normative.

    **The rule is about authority, not the clock.** `expiresAt` is deliberately excluded, and [Top-Level Members](#42-top-level-members) now says plainly that it bounds *acceptance* and does not abort work under way. Re-checking it mid-execution would turn a statement about a request's staleness into an execution timeout the *producer* never set and could not compute, since it does not know how long the *consumer*'s work takes. A task with a genuine completion deadline declares it in its own `payload`, where the specification can define what lapsing means — as `task-consent/request/0.1` already does — and item 12 then re-evaluates it like any other condition. No new framework member was needed.

    **Stopping is not automatically safe.** Once an irreversible effect has occurred, declining the next one does not undo it, and abandoning a partly applied change can leave the *recipient party* in a state neither party asked for. Item 12 sits *before* each effect for that reason, [Specification Requirements](#73-specification-requirements) item 16 asks multi-stage specifications to say when partial application is unsafe, and a *consumer* that does stop **SHOULD** report partial execution distinguishably from never having begun — a *producer* that cannot tell the two apart cannot decide whether to reissue.

* **A transport binding must justify any allowance to omit `proof` ([What a Transport Binding Specifies](#91-what-a-transport-binding-specifies), [Permitting `proof` to Be Omitted](#911-permitting-proof-to-be-omitted)).** [When to Include a Proof](#471-when-to-include-a-proof) has always let a document omit `proof` where the transport provides end-to-end integrity and authentication between *producer* and *consumer*, but [What a Transport Binding Specifies](#91-what-a-transport-binding-specifies) only **SHOULD**ed the security profile that would establish whether it does. A binding could therefore rest a proof allowance on "the transport is authenticated" without saying which party is authenticated, how that principal becomes a VID, what an intermediary can do to the bytes, or where the guarantee stops — leaving *consumers* to infer a security boundary from a transport's name.

    The profile is now **MUST** for any binding from which a framework security or identity requirement is derived, and a binding permitting omission must address eight specific properties, saying explicitly where one does not apply. Two rules carry most of the weight: hop authentication is **not** producer authentication where an intermediary can re-originate undetected, and an allowance **MUST** be stated per mode where a transport has both an end-to-end mode and a relayed one.

    **Silence is not permission.** A binding that does not address omission is not to be read as permitting it. A binding whose transport genuinely cannot offer the guarantee **SHOULD** say so plainly — that statement is as useful as an allowance, and it is what stops a familiar transport name from being taken for a guarantee it does not give.

* **Duplicate-execution protection ([Consumer Requirements](#72-consumer-requirements) item 11, [Standard Error Codes](#83-standard-error-codes), [Retry Semantics](#84-retry-semantics)).** A *consumer* **MUST NOT** let the same *Trust Task document* cause a consequential effect twice. The pieces were all present — unique `id`s ([The `id` Member](#43-the-id-member)), audience binding ([Audience Binding](#482-audience-binding)), bit-for-bit retry ([Retry Semantics](#84-retry-semantics)), and an idempotency cache recommended in [Cross-Recipient Replay](#121-cross-recipient-replay) — but the last of those was a **SHOULD** in a non-normative section, so two conforming consumers could both validate a repeated document correctly and one of them execute the transfer, the deletion, or the key rotation a second time. The framework had strong document-identifier uniqueness and no execution uniqueness to match it.

    The rule keys on the document `id` and is deliberately blind to intent: at the document layer a hostile replay and a legitimate transport retry are the same bytes arriving twice, and what matters is that the second arrival does not repeat the effect. Three questions the requirement has to answer, and does: a *consumer* retains a **digest**, not just an `id`, because an `id` alone cannot tell the retry it must absorb from the conflict it must reject; retention is bounded by the **same window** over which the *consumer* will still execute the document, so the rule never demands unbounded memory; and where a specification defines no success response, the duplicate is simply not executed and the silence is correct rather than an error.

    [Retry Semantics](#84-retry-semantics) gains the other half of the story — retry is safe *because* item 11 absorbs the duplicate — and records that a *producer* that re-signs or re-stamps has not retried but has issued a different document under a reused `id`.

* **`idConflict` ([Standard Error Codes](#83-standard-error-codes)).** A new standard error code for a document whose `id` matches one already accepted but whose content differs. Distinguishing this from a retry is the point: a retry is absorbed silently, a conflict is refused. Consumers at earlier framework versions will not recognize the code; `trust-task-error/0.4` carries it.

* **The term *consequential Trust Task* ([Terminology](#3-terminology)).** The predicate `sideEffects.level ∈ {mutating, destructive} ∨ exposure.discloses = secret ∨ exposure.actsAsSubject = true` is now named once rather than re-spelled at each use, with the fail-safe reading of an absent or unresolvable declaration folded into the definition. No new obligation attaches to the term itself.

* **Binding a citation to the document it names ([Binding a Citation to the Document It Names](#493-binding-a-citation-to-the-document-it-names)).** [Naming an Exchange from Outside the Framework](#491-naming-an-exchange-from-outside-the-framework) has required an external citation to name an exchange by the initiating document's `id` since 0.3, and an `id` turns out to be only half of an anchor. [The `id` Member](#43-the-id-member)'s uniqueness obligation binds *conforming producers*; it stops nobody from writing a different document — different parties, different `payload` — and giving it the same `id`. A verifier pairing a credential with a document by `id` equality alone accepts that counterfeit and then reports an event the documents do not attest. The gap was found from the DTG Core Credentials side, on a Verifiable Witness Credential whose `taskContext` is exactly such a citation.

    A citation relied upon outside the exchange now **SHOULD** carry a **task digest** over the document it names, and the computation is fixed where one is carried: JCS over the document with its **top-level `proof` removed**, hashed, multihash-tagged, multibase-encoded. Excluding `proof` is what makes the value well-defined — [Proof](#47-proof) already excludes `proof` from the content a proof covers, so the digest and the signature commit to the same content, and a document has one task digest whether or not it was ever signed. A digest that included `proof` would be undefined for every document [When to Include a Proof](#471-when-to-include-a-proof) permits to carry none, and would change value at the moment of signing.

    Two rules exist because the obvious implementations get them wrong. Comparison is over the **decoded multihash bytes**, never the encoded string: `DigestMultibase` admits both base58btc and base64url, so two conforming encodings of one digest are different strings and a string compare rejects a valid pairing. And an unimplemented hash algorithm makes a citation **unverified** — never recomputed under a substitute algorithm, and never silently downgraded to `id` comparison.

    The section states plainly what the mechanism does not do. The digest attests content, not authenticity; it is load-bearing because the *citing artifact* signs it, and a `proof`-stripped copy of a genuine document reproduces the same value by design. It is also **not** the document identity of [Consumer Requirements](#72-consumer-requirements) item 11, which asks which serialization arrived and counts a re-signed `proof` as a different document — that distinction is `idConflict` and remains untouched. Additive and non-breaking: no document member is added, and no existing citation becomes non-conforming.

* **`trust-task-next-step` published ([Reserved Response-Type Slugs](#86-reserved-response-type-slugs)).** The continuation response reserved since 0.1 now has a registry entry defining its payload. A *next step* is a **third** disposition alongside the success response and the *error response*: the two of those close the originating task, and a next step leaves it **open**. A *consumer* **MUST NOT** report a blocked task as an error, nor a refusal as a next step.

* **This framework specification is now versioned under Semantic Versioning ([Versioning of This Framework Specification](#511-versioning-of-this-framework-specification)).** Framework releases are numbered `MAJOR.MINOR.PATCH` and published at the full three-part version: this document's *Type URI* becomes `https://trusttasks.org/spec/trust-task/0.4.0`, and the `targetFrameworkVersion` a specification declares is that same three-part value. The point of the `PATCH` component is that errata to a published framework version get a number and a URI of their own. Previously they had nowhere to go: correcting `0.4` in place is exactly what [Stability](#64-stability) forbids for every other artifact in the registry, and minting `0.5` for a typo overstates what changed.

    **It applies to this document alone.** Individual *Trust Task specifications* keep the two-part `MAJOR.MINOR` of [Version Scheme](#51-version-scheme) — including the framework-defined ones, so `trust-task-error/0.4` is unchanged — and no *Type URI* outside the reserved `trust-task` slug moves.

    **Nothing already published stops resolving.** `0.1` through `0.4` denote `0.1.0` through `0.4.0`; `0.4` and `0.4.0` are the same release, and the entries below stand as published. A two-part `targetFrameworkVersion` resolves as `M.N.0`, and the registry continues to serve the two-part framework URIs as aliases. Specifications published or re-issued from this version onward declare the three-part form. The document wire format is unchanged.

<a id="03"></a>
### B.4 Framework version 0.3

* **Error responses can identify what failed ([Error Payload](#82-error-payload)).** The error payload gains an optional `inResponseTo` member carrying the reported-on document's `type` and `id`. Previously an *error response* was correlated only by `threadId`, which means something to a party that saw the originating request and nothing to anyone else — so an error retained as evidence named neither the task it terminated nor the instance, and for the standard codes of [Standard Error Codes](#83-standard-error-codes) carried no signal of origin at all. **SHOULD** in general, **MUST** where the error will be relied upon beyond the original *producer*. Published as `trust-task-error/0.3`; optional in this version so a 0.2 consumer's output remains valid, with a future major version expected to require it and 0.1/0.2 retired once consumers have moved.
* **Per-variant proof requirements ([Specification Requirements](#73-specification-requirements) item 8).** A *Trust Task specification* may now declare the `proof` requirement for its *request* and *response* variants separately, rather than one value covering both. The two are relied upon differently — a response retained as evidence outside the original exchange can need a proof where the request that triggered it does not, and a request that destroys state needs attribution where its acknowledgement protects nothing — and a single value forces the stricter onto both. The single form remains valid and unchanged; where the per-variant form omits the *response*, the *request*'s value applies, so an omission cannot weaken a variant. The error variant stays undeclarable: an error response's `type` names `trust-task-error`, a different specification, so a declaration here could not reach it. Additive — every existing declaration keeps its meaning.
* **The `parentThreadId` member ([The `parentThreadId` Member](#492-the-parentthreadid-member)).** A *Trust Task document* **MAY** now carry the `threadId` of the exchange that contains it, so a party holding a document from a nested exchange can find the exchange it was conducted within — something a flat `threadId` cannot express, and which specifications were otherwise forced to invent per-family payload conventions for. It takes `threadId`'s posture: optional, no normative validation semantics, consumers **MUST NOT** reject on it alone. It records one level of containment deliberately, rather than half-defining an ancestry chain. Where a transport carries its own parent-thread concept the two **MUST** agree when both are present, with the in-band member authoritative. Additive: the document wire format gains an optional member, and every document conforming to 0.2 still conforms.
* **Naming an exchange from outside the framework ([Naming an Exchange from Outside the Framework](#491-naming-an-exchange-from-outside-the-framework)).** Added the rule that anything referring to an exchange as evidence of an event — a credential citing the exchange that established what it attests, an audit record, a governance decision — **MUST** name the *innermost* exchange whose documents attest that event, by the `id` of the document that initiated it. A `threadId` names one exchange and expresses no containment, so where exchanges nest, more than one thread is open when an event occurs and only one attests it; naming an enclosing exchange collects evidence of the wrong event. Clarification only — no member is added and no existing behaviour changes.
* **Family namespaces for extended error codes ([Extension by Individual Trust Task Specifications](#85-extension-by-individual-trust-task-specifications)).** The namespace of an extended `code` may now be either the emitting specification's own slug (as before) or a *family namespace* — a proper path prefix of that slug — for a condition whose meaning is defined once across a family in a shared convention, such as `did-management:unknownDomain` on every `did-management/*` specification. Previously the namespace **MUST** have equalled the slug exactly, which gave a family-wide failure mode no way to be named once; specifications expressed it anyway, so the rule was already being broken to say something true. The relaxation is deliberately narrow: because a family namespace is always a prefix of the emitting slug, a *consumer* can still verify a received code's namespacing against the document's `type` alone, and a *sibling's* slug remains forbidden. Additive — every previously conforming code remains conforming. The prefix relationship is now enforced by the registry build, which never checked the original rule either.
* **Draft editorial changes stay in place ([Compatibility Rules](#52-compatibility-rules)).** An editorial or normalization change to a `draft` artifact — casing normalization per [Naming Conventions](#410-naming-conventions), a framework or shared-schema-component `$ref` re-pin with no wire effect, prose rewording — is now made in place and **MUST NOT** mint a new version. A wire-identical version minted before this rule **MAY** declare the new optional `wireCompatibleWith` front-matter field naming its predecessor, so consumers can dual-accept by mechanical normalization.
* **Side-effect and exposure classes ([Specification Requirements](#73-specification-requirements) items 13–14).** Every conforming specification now **MUST** declare two orthogonal, descriptive classifications of what executing the task does: a *side-effect class* (`none` / `mutating` / `destructive` — the integrity effect on recipient state) and an *exposure class* (`discloses` of `none` / `metadata` / `secret`, plus an `actsAsSubject` flag — the confidentiality and agency effect). Both are descriptive only — a specification **MUST NOT** derive a consent requirement from them — and exist so a delegated-execution consumer can decide whether to seek human approval without per-task code. This is a breaking change to the specification-authoring contract, carried by the internal `spec-meta/2.0` front-matter meta-schema; the **document wire format is unchanged from 0.2**, so `targetFrameworkVersion` and document validation are unaffected and specifications keep their existing framework-version targets.

<a id="02"></a>
### B.5 Framework version 0.2

* **Naming conventions ([Naming Conventions](#410-naming-conventions)).** Added a normative section defining casing: framework-defined members and values use **lowerCamelCase**; payload member names and specification-defined enumerated values **SHOULD** use lowerCamelCase; externally-owned values (WebAuthn, JOSE, `SameSite`, W3C *Data Integrity*, …) are carried verbatim.
* **Standard error codes re-cased ([Standard Error Codes](#83-standard-error-codes)).** The standard error `code` identifiers are now lowerCamelCase: `malformedRequest`, `unsupportedType`, `unsupportedVersion`, `proofRequired`, `proofInvalid`, `permissionDenied`, `wrongRecipient`, `identityMismatch`, `taskFailed`, `internalError` (the single-word codes `expired`, `unavailable` are unchanged). This is a breaking change carried by `trust-task-error/0.2`; the snake_case `0.1` codes remain valid for documents whose `type` resolves to a `0.1` specification.
* **Shared schema components ([Shared Schema Components](#66-shared-schema-components)).** Added a section giving shared schema fragments first-class, independently-versioned status, with a mandatory version-pinning rule and the schema/specification version-coupling rule.
* **Migration guidance ([Migrating Between Versions](#54-migrating-between-versions)).** Added the non-normative receiver-before-sender (expand/contract) migration sequence and the coupling of schema and specification versions.
* **Draft version caveat ([Compatibility Rules](#52-compatibility-rules)).** Clarified that a breaking change to a `draft` artifact MAY be released as a `MINOR` increment.
* Affected `0.1` specifications were re-published as `0.2` with lowerCamelCase enumerated values; `0.1` remains served unchanged for backwards compatibility and will be `retired` once consumers have migrated.

<a id="01"></a>
### B.6 Framework version 0.1

* Initial working draft of the Trust Tasks framework.

<a id="14-acknowledgments"></a>
## Appendix C — Acknowledgements

The editors thank the members of the Trust Over IP Foundation Decentralized Trust Graph Working Group for their ongoing review and contributions to this specification. The editors also thank the participants of the DTG Credentials Task Force, whose review of Trust Task citation from the credential side produced the task-digest requirement of [Binding a Citation to the Document It Names](#493-binding-a-citation-to-the-document-it-names).

Copyright © 2026 Trust Over IP (ToIP) Contributors  
This work is licensed under a Creative Commons Attribution 4.0 International License.
