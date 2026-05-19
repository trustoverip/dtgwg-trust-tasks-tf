//! Compile-time assertions that codegen emits `IS_BEARER` and
//! `IS_PROOF_REQUIRED` correctly for every spec in `specs/`.
//!
//! These pin the front-matter → trait-const wiring so a regression in
//! `trust-tasks-codegen` (or a typo in a spec's front matter) fails
//! the workspace build instead of silently shipping a spec whose
//! runtime contract no longer matches its prose.
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
