// The DIDComm v2 transport binding is a SEPARATE module from trust-tasks-go,
// like tsp and proof, so the core stays dependency-free. It rolls DIDComm v2.1
// authcrypt (ECDH-1PU key agreement, A256KW key wrapping, A256CBC-HS512 content
// encryption) on the Go standard library's crypto/ecdh — no third-party crypto,
// the same roll-our-own basis as trust-tasks-go/proof and the place PQC lands
// later. Its only requirement is the core module whose §7.2 pipeline and
// TransportHandler seam it plugs into.
module github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/didcomm

go 1.22

require github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go v0.1.2
