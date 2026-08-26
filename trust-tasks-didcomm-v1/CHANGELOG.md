# Changelog

All notable changes to `trust-tasks-didcomm-v1` are documented in this file.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [0.10.0] - 2026-08-26

### Added

- **The `legacy-basic-message` feature — a sunset for the `0.1` carriage.**
  Binding `didcomm-v1/0.2` §2.3 requires a `0.2` consumer to accept `0.1`'s
  Aries `basic-message` carriage as well as this binding's dedicated message
  type. `basic-message` is the Aries **chat** type, so while that gate is open
  any chat message from any established connection that carries a `trust-task`
  attachment is a framework input — from every peer, with no end date. §2.3
  puts the contraction in a future MAJOR but gave implementations no way to
  reach it early.

  The legacy carriage now sits behind a Cargo feature. It is **on by default**,
  because §2.3 makes accepting it a MUST and nothing in this repository depends
  on it either way; turning it off is a deliberate departure from that MUST,
  available to a deployment that knows all its peers have migrated:

  ```toml
  trust-tasks-didcomm-v1 = { version = "0.10", default-features = false }
  ```

  With the feature off, such a message is `DidcommV1Error::WrongMessageType`.

- **The legacy carriage is surfaced as superseded**, which is §2.3's SHOULD and
  was previously not done at all: every message arriving on it is logged at
  `warn` through the `log` facade (naming the sender, so an operator can see
  *which* peers have not migrated), and reported as
  `Carriage::LegacyBasicMessage` on the handler for callers that would rather
  meter it than grep logs. `DidcommV1Handler::carriage()` is the accessor;
  `with_carriage` sets it.

- `log` is a new dependency (the facade only — a binary that installs no logger
  pays nothing).

### Notes

Default-on means the default build behaves exactly as 0.9.0 did, so this is a
minor bump. The observable additions are the `Carriage` enum and the handler
accessor.

## [0.9.0] and earlier

See the repository history; this crate kept no changelog before 0.10.0.
