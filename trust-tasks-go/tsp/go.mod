// The TSP transport binding is a SEPARATE module from trust-tasks-go so the
// core stays dependency-free (its selling point, and what the website claims).
// This module pulls in affinidi-tsp-go and, through it, the PQ crypto libraries.
//
// affinidi-tsp-go v0.2 speaks TSP 1.0 (YTSP-BAA) only, like the Rust
// affinidi-tsp and @openvtc/vti-tsp-js the other TSP bindings use: it refuses
// the Rev 3 and Rev 2 drafts, and TSP has no version negotiation, so a move to
// a later TSP version is a deliberate bump here, with the interop fixtures in
// interop_test.go regenerated.
module github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/tsp

go 1.27

require (
	github.com/affinidi/affinidi-tsp-go v0.2.0
	github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go v0.13.3
)

require (
	github.com/cloudflare/circl v1.6.5 // indirect
	golang.org/x/crypto v0.59.0 // indirect
	golang.org/x/sys v0.50.0 // indirect
)
