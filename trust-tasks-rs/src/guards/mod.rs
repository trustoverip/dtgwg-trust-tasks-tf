//! Helpers for spec-handler policy checks that are easy to get wrong
//! and worth centralising.
//!
//! Each submodule corresponds to a *Trust Task specification* family and
//! supplies one or more pure functions a maintainer can call from inside
//! their handler. The functions are deliberately generic over the
//! consumer's entry type — webvh-flavoured `AclEntry`s carrying ecosystem
//! `ext` fields work without conversion.
//!
//! This module is hand-written and sits outside the codegen tree so it
//! survives `cargo run -p trust-tasks-codegen` without further plumbing.

pub mod acl;
