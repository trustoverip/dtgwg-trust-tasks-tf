#!/usr/bin/env bash
#
# Fails a pull request that breaks the public API of the core Go module
# (trust-tasks-go) without declaring it.
#
#   PR_TITLE='feat(go)!: …' scripts/check-go-api-compat.sh
#
# Environment:
#   PR_TITLE          the pull request title (REQUIRED; passed through the
#                     environment, never spliced into a workflow `run:`)
#   BASE_REF          the PR's base commit, for the "already breaking" history
#                     check below (default: HEAD)
#   APIDIFF           the apidiff binary (default: `apidiff` on PATH)
#   GO_API_BASELINE   compare against this tag instead of the newest
#                     trust-tasks-go/v* (for testing the script)
#
# Run from the repository root with the trust-tasks-go/v* tags present
# (`fetch-depth: 0` in CI).
#
# ── Why ──────────────────────────────────────────────────────────────────────
#
# A Go release is sized by conventional commits (scripts/release-bump-go.sh): a
# `!` makes it breaking, anything else is a patch. Rust has cargo-semver-checks
# to catch a break nobody declared; Go had nothing, so an accidental break
# shipped as a patch — and a Go tag can never be retracted. This compares the
# PR's API with the LAST RELEASE (the newest trust-tasks-go/v* tag, which IS the
# publication — see release-bump-go.sh), using golang.org/x/exp/cmd/apidiff over
# every package of the module. The nested modules (tsp, proof, didcomm,
# capabilityclient) are separate modules and are not part of `apidiff -m` on
# the core; `internal` packages are not API and apidiff skips them.
#
# ── The verdict ──────────────────────────────────────────────────────────────
#
#   no incompatible change                    → pass
#   incompatible, and the title is `<type>(…go…)!:` → pass, listing the breaks
#   incompatible, and a commit already on the base branch since the tag
#     declared a break (by the same rule release-bump-go.sh sizes the release
#     with)                                   → pass: the release is breaking anyway
#   otherwise                                 → FAIL, listing the breaks
#
# ── What is NOT a break here, although apidiff says it is ───────────────────
#
# apidiff reports every changed constant VALUE as incompatible (a constant can
# size an array or collide in a switch). Four generated constants and one
# hand-written one change value by design, and a string constant's new value
# breaks no build:
#
#   specs/**.PayloadSchemaJSON, ResponsePayloadSchemaJSON
#       the schema text: it moves with every additive in-place edit (a new
#       optional member, a description). Whether the SCHEMA changed compatibly
#       is the published-schemas check's job (scripts/check-published-schemas.mjs),
#       not this one's.
#   specs/**.Deprecation, specs/**.Extends
#       lifecycle metadata: "" → "superseded by …" when a newer version lands is
#       the point of the constant.
#   trusttasks.Version
#       moved by every Release PR.
#
# Everything else apidiff calls incompatible counts: a removed package (a
# withdrawn spec version — the withdrawal batch's title already carries
# `go` in a `!` scope), a removed or renamed type, field, function or constant,
# a changed signature or field type, a changed TypeURI.
#
# What this cannot see: a behavioural break with no API change — a spec that
# starts requiring a proof flips `Spec.IsProofRequired`, which is a var's
# initialiser and invisible to apidiff. Declare those yourself.

set -euo pipefail

APIDIFF="${APIDIFF:-apidiff}"
BASE_REF="${BASE_REF:-HEAD}"
MODULE="github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go"
PKG_DIR="trust-tasks-go"
TAG_PREFIX="trust-tasks-go/v"
# The same watch set as release-bump-go.sh's core module, for the history check.
WATCH=("specs" "$PKG_DIR" "scripts/build-go-bindings.mjs"
       ":(exclude)$PKG_DIR/proof" ":(exclude)$PKG_DIR/didcomm" ":(exclude)$PKG_DIR/tsp"
       ":(exclude)$PKG_DIR/capabilityclient")
# Lines of apidiff's incompatible section that are not breaks (see above).
EXEMPT='^- \./specs/[^ ]*\.(PayloadSchemaJSON|ResponsePayloadSchemaJSON|Deprecation|Extends): value changed from |^- \./trusttasks\.Version: value changed from '

if [ -z "${PR_TITLE+x}" ]; then
  echo "::error::PR_TITLE is not set."
  exit 2
fi

tag="${GO_API_BASELINE:-$(git tag -l "${TAG_PREFIX}*" --sort=-version:refname | head -1)}"
if [ -z "$tag" ]; then
  echo "::error::No ${TAG_PREFIX}* tag found — fetch tags (fetch-depth: 0)."
  exit 2
fi

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

# The released tree, as a plain export: no second worktree to clean up, and the
# module loads from its own directory exactly as the tag's consumers build it.
mkdir -p "$work/old"
git archive "$tag" "$PKG_DIR" | tar -x -C "$work/old"

start=$(date +%s)
(cd "$work/old/$PKG_DIR" && "$APIDIFF" -m -w "$work/old.export" "$MODULE")
(cd "$PKG_DIR" && "$APIDIFF" -m -w "$work/new.export" "$MODULE")
"$APIDIFF" -m "$work/old.export" "$work/new.export" >"$work/report.txt"
elapsed=$(( $(date +%s) - start ))

# apidiff prints "Incompatible changes:" then "Compatible changes:", each
# followed by "- …" lines. Its exit status does not reflect the verdict.
incompatible=$(awk '/^Incompatible changes:/{s=1;next} /^Compatible changes:/{s=0} s && /^- /' "$work/report.txt")
compatible_count=$(awk '/^Compatible changes:/{s=1;next} s && /^- /' "$work/report.txt" | wc -l | tr -d ' ')
breaks=$(grep -vE "$EXEMPT" <<<"$incompatible" || true)
exempt_count=$(grep -cE "$EXEMPT" <<<"$incompatible" || true)

summary="$MODULE vs $tag: $compatible_count compatible change(s), ${exempt_count:-0} constant-value change(s) exempt by design, in ${elapsed}s"

if [ -z "$breaks" ]; then
  echo "::notice::No incompatible API change. $summary."
  exit 0
fi

count=$(wc -l <<<"$breaks" | tr -d ' ')
echo "Incompatible changes ($count):"
echo "$breaks"
echo

# Does the title declare a break of THIS package? `!` with `go` among its
# scopes (commit-lint already refuses a `!` scoped to anything but packages).
declared=no
if scopes=$(printf '%s' "$PR_TITLE" | sed -nE 's/^[a-z]+\(([^)]*)\)!:.*/\1/p') && [ -n "$scopes" ]; then
  if printf '%s\n' "$scopes" | tr ',' '\n' | sed 's/^ *//; s/ *$//' | grep -qx 'go'; then
    declared=yes
  fi
fi
if [ "$declared" = yes ]; then
  echo "::notice::$count incompatible API change(s), declared by the PR title's \`go\` scope. $summary."
  exit 0
fi

# A break already merged since the tag, by release-bump-go.sh's own rule, makes
# the next Go release breaking whatever this PR is titled. Captured before
# grepping, not piped, for the SIGPIPE reason given in release-bump-go.sh. Keep
# this regex identical to that script's: if it ever becomes scope-aware, this
# must follow, or the two disagree about how big the next release is.
log=$(git log --format='%s%n%b' "$tag..$BASE_REF" -- "${WATCH[@]}")
if prior=$(grep -m1 -E '^[a-z]+(\([^)]*\))?!:|^BREAKING[ -]CHANGE' <<<"$log"); then
  echo "::notice::$count incompatible API change(s); the next Go release is already breaking (since $tag: \"$prior\"). $summary."
  exit 0
fi

while IFS= read -r line; do
  printf '::error::Undeclared Go API break: %s\n' "${line#- }"
done <<<"$breaks"
cat <<EOF

This PR makes $count incompatible change(s) to the public API of $MODULE
since $tag, and its title does not declare a break of the Go module.

Either make the change compatible (add, don't remove or rename; a new spec
version is a new package), or retitle the PR with \`go\` in a \`!\` scope —
e.g. \`feat(go)!: …\` — so the release is sized as breaking. See RELEASING.md.
EOF
exit 1
