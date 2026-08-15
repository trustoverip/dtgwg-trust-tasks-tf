//! Compile-time spot-checks that codegen emits `IS_BEARER` and
//! `IS_PROOF_REQUIRED` correctly, for a handful of representative specs.
//!
//! **Not exhaustive, despite what this comment used to claim.** It said these
//! covered "every spec in `specs/`"; they cover six, by hand, out of three
//! hundred. A test that misstates its own coverage is worse than a missing one,
//! because it is exactly what stops someone writing the real check — which is
//! the same hand-maintained-list failure that hid a stale category enum and two
//! unregistered transport bindings.
//!
//! The real check is `scripts/check-bindings-conformance.mjs`, run by the
//! `bindings match specs` CI job. It compares front matter, Rust and TypeScript
//! for every published spec version, and it is what would have caught the
//! response-type drop (#174) and the `Payload` alias (#215).
//!
//! These assertions are kept because they are free and they fail during
//! `cargo test`, without Node, so a Rust-side regression surfaces where a Rust
//! developer is already looking. Treat them as a fast canary, not as coverage.
//!
//! The assertions live in a `const _: () = …` so they fire at
//! compile time. No runtime harness needed.

use trust_tasks_rs::{
    specs::acl::{change_role, grant, list, revoke, show},
    specs::trust_task_discovery,
    Payload,
};

// ─── IS_PROOF_REQUIRED ─────────────────────────────────────────────────
//
// Front-matter `proofRequirement.requirement: REQUIRED` → const set
// to true. `RECOMMENDED` / `OPTIONAL` → const stays at trait default
// (`false`). The Response impl gets the same value as the request impl
// because codegen emits both symmetrically; only the request-side
// check matters at runtime, but the const is reachable on both for
// generic-context inspection.

// REQUIRED specs.
const _: () = assert!(<grant::v0_1::Payload as Payload>::IS_PROOF_REQUIRED);
const _: () = assert!(<grant::v0_1::Response as Payload>::IS_PROOF_REQUIRED);
const _: () = assert!(<revoke::v0_1::Payload as Payload>::IS_PROOF_REQUIRED);
const _: () = assert!(<revoke::v0_1::Response as Payload>::IS_PROOF_REQUIRED);
const _: () = assert!(<change_role::v0_1::Payload as Payload>::IS_PROOF_REQUIRED);
const _: () = assert!(<change_role::v0_1::Response as Payload>::IS_PROOF_REQUIRED);

// RECOMMENDED specs — MUST NOT have the const set.
const _: () = assert!(!<list::v0_1::Payload as Payload>::IS_PROOF_REQUIRED);
const _: () = assert!(!<list::v0_1::Response as Payload>::IS_PROOF_REQUIRED);
const _: () = assert!(!<show::v0_1::Payload as Payload>::IS_PROOF_REQUIRED);
const _: () = assert!(!<show::v0_1::Response as Payload>::IS_PROOF_REQUIRED);

// OPTIONAL specs — MUST NOT have the const set.
const _: () = assert!(!<trust_task_discovery::v0_1::Payload as Payload>::IS_PROOF_REQUIRED);
const _: () = assert!(!<trust_task_discovery::v0_1::Response as Payload>::IS_PROOF_REQUIRED);

// ─── IS_RECIPIENT_REQUIRED ─────────────────────────────────────────────
// acl/grant declares both parties REQUIRED, so request and response both set
// the const (the response's recipient is the request's issuer — also REQUIRED).
const _: () = assert!(<grant::v0_1::Payload as Payload>::IS_RECIPIENT_REQUIRED);
const _: () = assert!(<grant::v0_1::Response as Payload>::IS_RECIPIENT_REQUIRED);

// ─── IS_BEARER ─────────────────────────────────────────────────────────
//
// No spec in this registry declares `bearer: true` in front matter, so
// every const should be the trait default (`false`). Pinning this
// prevents a future codegen regression from silently bypassing the
// audience-binding check (SPEC §4.8.2 / §7.2 item 8).

const _: () = assert!(!<grant::v0_1::Payload as Payload>::IS_BEARER);
const _: () = assert!(!<revoke::v0_1::Payload as Payload>::IS_BEARER);
const _: () = assert!(!<change_role::v0_1::Payload as Payload>::IS_BEARER);
const _: () = assert!(!<list::v0_1::Payload as Payload>::IS_BEARER);
const _: () = assert!(!<show::v0_1::Payload as Payload>::IS_BEARER);
const _: () = assert!(!<trust_task_discovery::v0_1::Payload as Payload>::IS_BEARER);

#[test]
fn const_assertions_are_reachable() {
    // The `const _: () = …` items above fire at compile time. This
    // test only exists so the integration-test target picks the file
    // up; if it isn't reachable the const assertions don't run either.
}
