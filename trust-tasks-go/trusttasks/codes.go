package trusttasks

import (
	"fmt"
	"regexp"
	"strings"
)

// StandardCode is a framework-defined standard error code (SPEC.md §8.3).
//
// # The asymmetry with TypeScript, and why Go sides with Rust
//
// @openvtc/trust-tasks declares its StandardCode as a closed union, so adding a
// framework standard code is a breaking change there: a switch that covers every
// member stops being exhaustive. trust-tasks-rs marks its enum #[non_exhaustive]
// (since 0.7.0), so the same addition is additive.
//
// Go has no closed enum. A named string type is open by construction, which puts
// this package in Rust's position: adding a standard code is additive here, a Go
// switch is never exhaustive, and a default arm was always required. That is the
// better trade for the same reason it was in Rust — the cost of a new standard
// code should not be a major version of the SDK.
//
// What it does not buy you is spell-checking: StandardCode("proofRequred")
// compiles. Use the constants below, and [IsStandardCode] to narrow a wire value.
type StandardCode string

// The framework-defined standard error codes (SPEC.md §8.3).
const (
	CodeMalformedRequest   StandardCode = "malformedRequest"
	CodeUnsupportedType    StandardCode = "unsupportedType"
	CodeUnsupportedVersion StandardCode = "unsupportedVersion"
	CodeExpired            StandardCode = "expired"
	CodeProofRequired      StandardCode = "proofRequired"
	CodeProofInvalid       StandardCode = "proofInvalid"
	CodePermissionDenied   StandardCode = "permissionDenied"
	CodeWrongRecipient     StandardCode = "wrongRecipient"
	CodeIdentityMismatch   StandardCode = "identityMismatch"
	CodeIDConflict         StandardCode = "idConflict"
	CodeCancelled          StandardCode = "cancelled"
	CodeTaskFailed         StandardCode = "taskFailed"
	CodeUnavailable        StandardCode = "unavailable"
	CodeInternalError      StandardCode = "internalError"
)

// StandardCodes lists every §8.3 code, in the order SPEC.md declares them.
//
// Returned as a fresh slice so a caller cannot reorder or overwrite the
// framework's own list — a package-level slice var would be mutable by anyone
// who imports it.
func StandardCodes() []StandardCode {
	return []StandardCode{
		CodeMalformedRequest, CodeUnsupportedType, CodeUnsupportedVersion,
		CodeExpired, CodeProofRequired, CodeProofInvalid, CodePermissionDenied,
		CodeWrongRecipient, CodeIdentityMismatch, CodeIDConflict, CodeCancelled,
		CodeTaskFailed, CodeUnavailable, CodeInternalError,
	}
}

var standardSet = func() map[string]StandardCode {
	set := make(map[string]StandardCode)
	for _, code := range StandardCodes() {
		set[string(code)] = code
	}
	return set
}()

// legacyStandard maps the frozen framework 0.1 snake_case spellings to their 0.2
// lowerCamelCase form, so a 0.2 consumer can still read an error response from a
// 0.1 peer. `expired` and `unavailable` are single words and unchanged.
var legacyStandard = map[string]StandardCode{
	"malformed_request":   CodeMalformedRequest,
	"unsupported_type":    CodeUnsupportedType,
	"unsupported_version": CodeUnsupportedVersion,
	"proof_required":      CodeProofRequired,
	"proof_invalid":       CodeProofInvalid,
	"permission_denied":   CodePermissionDenied,
	"wrong_recipient":     CodeWrongRecipient,
	"identity_mismatch":   CodeIdentityMismatch,
	"id_conflict":         CodeIDConflict,
	"canceled":            CodeCancelled,
	"task_failed":         CodeTaskFailed,
	"internal_error":      CodeInternalError,
}

// NormalizeCode maps a wire code to its canonical 0.2 spelling when it is a
// standard code, or returns it unchanged.
//
// A consumer comparing a received code against the §8.3 set must normalize
// first, or a 0.1 peer's proof_required reads as an unrecognized extended code
// and falls through to taskFailed (§8.5), losing the meaning.
func NormalizeCode(code string) string {
	if _, ok := standardSet[code]; ok {
		return code
	}
	if canonical, ok := legacyStandard[code]; ok {
		return string(canonical)
	}
	return code
}

// IsStandardCode reports whether code is a standard §8.3 code, in either casing,
// and returns its canonical form.
func IsStandardCode(code string) (StandardCode, bool) {
	canonical, ok := standardSet[NormalizeCode(code)]
	return canonical, ok
}

// localRE is the local part of an extended code: a lowercase letter, then
// letters of either case, digits, or underscores.
//
// Both casings are accepted so framework 0.2 lowerCamelCase locals
// (documentRevoked) and frozen 0.1 snake_case locals (document_revoked) parse
// under one rule. SPEC §4.10 item 4 SHOULDs lowerCamelCase for new
// specifications; only the first character is required to be lowercase.
var localRE = regexp.MustCompile(`^[a-z][A-Za-z0-9_]*$`)

// segmentRE matches one path segment of a slug: lowercase, hyphen-separated
// (§6.1).
var segmentRE = regexp.MustCompile(`^[a-z][a-z0-9]*(-[a-z0-9]+)*$`)

func validNamespace(namespace string) bool {
	if namespace == "" {
		return false
	}
	for _, segment := range strings.Split(namespace, "/") {
		if !segmentRE.MatchString(segment) {
			return false
		}
	}
	return true
}

// ExtendedCode builds an extended error code under a specification's own slug
// (SPEC §8.5).
//
// typeURI is normally a generated package's TypeURI, so the namespace cannot
// drift from the type's identity. The #response fragment is stripped: an error
// raised while handling a response still belongs to the bare slug.
func ExtendedCode(typeURI, local string) (string, error) {
	slug, err := SlugFromTypeURI(typeURI)
	if err != nil {
		return "", err
	}
	if !localRE.MatchString(local) {
		return "", fmt.Errorf(
			"trusttasks: extended-code local part %q must match %s "+
				"(a lowercase first character, then letters, digits or underscores)",
			local, localRE,
		)
	}
	return slug + ":" + local, nil
}

// FamilyCode builds an extended error code under a family namespace — a proper
// path prefix of the specification's slug (SPEC §8.5 rule 2).
//
// For a condition whose meaning is defined once across a family rather than per
// specification, such as did-management:unknownDomain on every did-management/*
// task. Prefer [ExtendedCode] otherwise: a family namespace claims the condition
// means the same thing across every sibling.
//
// namespace is checked against the slug derived from typeURI rather than taken
// on trust, so §8.5's prefix rule holds by construction. A sibling's slug is
// rejected — it shares a prefix but is not itself one, which is exactly the
// confusion §8.5 forbids.
func FamilyCode(typeURI, namespace, local string) (string, error) {
	slug, err := SlugFromTypeURI(typeURI)
	if err != nil {
		return "", err
	}
	segs := strings.Split(slug, "/")
	permitted := make([]string, 0, len(segs))
	for i := range segs {
		permitted = append(permitted, strings.Join(segs[:i+1], "/"))
	}
	found := false
	for _, candidate := range permitted {
		if candidate == namespace {
			found = true
			break
		}
	}
	if !found {
		return "", fmt.Errorf(
			"trusttasks: namespace %q is neither the slug %q nor a path prefix of it "+
				"(SPEC §8.5 rule 2); permitted: %s",
			namespace, slug, strings.Join(permitted, ", "),
		)
	}
	if !localRE.MatchString(local) {
		return "", fmt.Errorf("trusttasks: extended-code local part %q must match %s", local, localRE)
	}
	return namespace + ":" + local, nil
}

// typeURIPrefix is the scheme and host every Trust Task Type URI carries.
const typeURIPrefix = "https://trusttasks.org/spec/"

// SlugFromTypeURI returns the slug of a Type URI, with any #request / #response
// fragment removed.
func SlugFromTypeURI(typeURI string) (string, error) {
	if !strings.HasPrefix(typeURI, typeURIPrefix) {
		return "", fmt.Errorf("trusttasks: not a Trust Task Type URI: %q", typeURI)
	}
	rest := bareTypeURI(strings.TrimPrefix(typeURI, typeURIPrefix))
	// The trailing segment is the MAJOR.MINOR version; everything before it is
	// the slug.
	parts := strings.Split(rest, "/")
	if len(parts) < 2 {
		return "", fmt.Errorf("trusttasks: Type URI %q carries no slug", typeURI)
	}
	slug := strings.Join(parts[:len(parts)-1], "/")
	if !validNamespace(slug) {
		return "", fmt.Errorf("trusttasks: Type URI %q yielded an invalid slug", typeURI)
	}
	return slug, nil
}
