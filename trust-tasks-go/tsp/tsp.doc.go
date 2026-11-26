// Package tsp is the ToIP Trust Spanning Protocol (TSP) transport binding
// (bindings/tsp/0.3) for the Trust Tasks framework, built on
// github.com/affinidi/affinidi-tsp-go.
//
// PackTrustTask seals a document into a Direct TSP message; UnpackTrustTask
// opens one; Consumer is the guarded inbound path that runs the SPEC §7.2
// pipeline with the duplicate-execution record on by default. The sealed
// envelope object is the same {type, document} shape the Rust trust-tasks-tsp
// crate and the Dart trust_tasks_tsp package seal, so a document sealed by any
// of them opens in the others.
//
// It speaks TSP 1.0 ([TSPVersion]) and nothing else: a message from a peer
// still on a TSP draft is refused [FailUnsupportedTSPVersion].
package tsp
