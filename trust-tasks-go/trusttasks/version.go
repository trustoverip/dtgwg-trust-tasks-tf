package trusttasks

// Version is the released version of this module.
//
// # Why a constant exists at all
//
// A Go module has no manifest. `go.mod` names the module but carries no
// version — the version of a Go module *is* its git tag, resolved by
// proxy.golang.org, and nothing in the tree records it. That is fine for
// consumers (`go get` pins it in their go.mod) but it leaves the release
// automation in this repo with nothing to bump: `release-plz` moves a
// `Cargo.toml`, `scripts/release-ts-pr.sh` moves a `package.json`, and the Go
// equivalent would move nothing at all, so the Release PR would be an empty
// diff and `publish-go` would have no value to compare against the tags.
//
// So this constant is the manifest. `scripts/release-go-pr.sh` rewrites it, and
// the `publish-go` job in `.github/workflows/publish.yml` reads it to decide
// whether `trust-tasks-go/v<Version>` needs tagging.
//
// It is also genuinely useful: a consumer can log which binding version
// produced a document, which the other two SDKs get for free from their
// package metadata and Go otherwise cannot answer at runtime.
//
// DO NOT EDIT BY HAND. See RELEASING.md — a version in a feature PR collides
// with every other open PR.
const Version = "0.1.11"
