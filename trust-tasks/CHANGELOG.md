# Changelog

## [0.18.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.18.0...trust-tasks-v0.18.1) — 2026-09-06


## [0.18.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.17.10...trust-tasks-v0.18.0) — 2026-09-06


## [0.17.10](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.17.9...trust-tasks-v0.17.10) — 2026-09-05


## [0.17.9](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.17.8...trust-tasks-v0.17.9) — 2026-09-05


## [0.17.8](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.17.7...trust-tasks-v0.17.8) — 2026-09-04


## [0.17.7](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.17.5...trust-tasks-v0.17.7) — 2026-09-02


## [0.17.5](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.17.4...trust-tasks-v0.17.5) — 2026-09-02


## [0.17.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.17.3...trust-tasks-v0.17.4) — 2026-09-01


## [0.17.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.17.2...trust-tasks-v0.17.3) — 2026-08-28


## [0.17.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.17.1...trust-tasks-v0.17.2) — 2026-08-28


## [0.17.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.17.0...trust-tasks-v0.17.1) — 2026-08-27


## [0.17.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.2.0...trust-tasks-v0.17.0) — 2026-08-27


### Changed

- **versioning**: Release the trust-tasks-rs-exposing crates in lockstep ([#315](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/315))


### Specifications

- Bound every free-text payload member with a maxLength (§7.3) ([#296](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/296))

* spec: bound every free-text payload member with a maxLength

  SPEC.md §7.3 (framework 0.5.0) requires that any member holding free
  text declare a `maxLength`. 92 free-text string members across 83 draft
  schemas carried none, leaving the wire contract unbounded and every
  consumer to invent its own ceiling — or none, which is what §10.3
  (schema-validation DoS) exists to prevent.

  Bounds are chosen per member from the vocabulary the registry already
  uses rather than applied uniformly:

    256   `label`, `comment` — a display name or an OpenSSH key comment;
          matches the existing 256 on provision/integration `label` and
          the `name` members alongside it.
    500   requester-authored prose that a surface renders to a human who
          is deciding something; matches task-consent/request/0.1 `note`,
          the registry's considered consent-surface bound.
    1024  `reason`, `description`, `message` — operator or service prose
          recorded for audit or returned as a diagnostic; matches the six
          existing `reason: 1024` and the `description: 1024` in policy/
          and vtc/endorsement-type.
    16384 chat/message `text` — the task's actual content rather than
          metadata about it; matches the corpus's long-form bound on
          vault `secureNotes`.

  All amended specifications are `status: draft`, so the change is made in
  place per SPEC §5.2. Deliberately untouched:

    * 17 members in `retired` specifications, frozen by SPEC §6.4.
    * messaging/_shared/0.1 `AuditEntry.detail` and did-management/
      _shared/0.1 `DomainEntry.label` — shared $defs reachable from a
      retired specification, so bounding them would change a frozen
      specification's effective wire contract.
    * vault/_shared/{0.1,0.2,0.3} `TspMessageEnvelope.message` — opaque
      base64url TSP bytes, not free text.

  The `label` description in vault/_shared/*/vault-entry.schema.json said
  the wire spec enforced no maximum length. It now does, so the sentence
  is corrected rather than left contradicting the schema it annotates.

  `npm run validate` re-checks all 533 fenced example documents against
  the amended schemas; none is rejected by a new bound.



## [0.2.0] - 2026-08-26

### Changed

- **`trust-tasks-https` requirement moved to `0.16`.** That release makes
  `HttpsServer::on` require `RequestPayload` and adds `on_ack`; this crate
  re-exports the type, so the leading component moves with it. No change to
  this crate's own API.

