package proof

// Version is the released version of this module (trust-tasks-go/proof).
//
// A Go module has no manifest, and the version of a Go module *is* its git tag
// (here `trust-tasks-go/proof/v<Version>`, resolved by proxy.golang.org). As
// trust-tasks-go/trusttasks/version.go explains at length, that leaves the
// release automation with nothing to bump, so this constant stands in for the
// manifest: scripts/release-go-pr.sh rewrites it, and the `publish-go` job in
// .github/workflows/publish.yml reads it to decide whether
// `trust-tasks-go/proof/v<Version>` needs tagging.
//
// DO NOT EDIT BY HAND. See RELEASING.md — a version in a feature PR collides
// with every other open PR.
const Version = "0.1.1"
