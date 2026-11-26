// The TSP transport binding is a SEPARATE module from trust-tasks-go so the
// core stays dependency-free (its selling point, and what the website claims).
// This module pulls in affinidi-tsp-go and, through it, the PQ crypto libraries.
//
// affinidi-tsp-go is not tagged yet, so it is required at a pseudo-version from
// its public repository — the Go proxy resolves it. Bump to a real tag when one
// is published.
module github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/tsp

go 1.27

require (
	github.com/affinidi/affinidi-tsp-go v0.2.0
	github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go v0.13.4
)

require (
	github.com/cloudflare/circl v1.6.5 // indirect
	golang.org/x/crypto v0.59.0 // indirect
	golang.org/x/sys v0.50.0 // indirect
)
