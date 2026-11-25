module example.com/trust-tasks-hcp-demo-go

go 1.27

require (
	github.com/affinidi/affinidi-tsp-go v0.0.0-20260916130837-d32ca7cdde20
	github.com/santhosh-tekuri/jsonschema/v6 v6.0.3
	github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go v0.13.3
	github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/proof v0.0.0-00010101000000-000000000000
	github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/specs v0.0.0-00010101000000-000000000000
	github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/tsp v0.0.0-00010101000000-000000000000
)

require (
	github.com/cloudflare/circl v1.6.5 // indirect
	golang.org/x/crypto v0.57.0 // indirect
	golang.org/x/sys v0.48.0 // indirect
	golang.org/x/text v0.42.0 // indirect
)

// The example builds against this checkout's runtime, proof and TSP modules and
// generator, so it tests them before they are released. An application outside
// this repository requires published versions and has no replace.
replace (
	github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go => ../../trust-tasks-go
	github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/proof => ../../trust-tasks-go/proof
	github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/specs => ../../trust-tasks-go/specs
	github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/tsp => ../../trust-tasks-go/tsp
)
