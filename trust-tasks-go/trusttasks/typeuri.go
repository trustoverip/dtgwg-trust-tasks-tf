package trusttasks

// Parsed Trust Task Type URI.
//
// Models every form SPEC.md §6.1 defines — the public registry's canonical
// form, and the two shapes a private specification (SPEC.md §6.5) may use under
// any other authority:
//
//	https://trusttasks.org/spec/<slug>/<MAJOR.MINOR>          public registry
//	<prefix>/spec/<slug>/<MAJOR.MINOR>                        path form (https, DID URL, …)
//	urn:<NID>:<prefix>:spec:<slug-segments>:<MAJOR.MINOR>     URN form
//
// each optionally followed by #request or #response (§4.4.1). A Type URI whose
// scheme is `urn` must use the URN form, and every other scheme the path form;
// `http` is refused in every form. The authority is kept verbatim and is part of
// identity: the same slug under two authorities is two specifications.
//
// This is a port of trust-tasks-rs/src/type_uri.rs, whose tests it shares.

import (
	"fmt"
	"strconv"
	"strings"
)

// TypeURIVariant is the #request / #response fragment of a Type URI (SPEC
// §4.4.1). The zero value means no fragment, which is equivalent to
// [VariantRequest].
type TypeURIVariant string

// The Type URI variants.
const (
	VariantNone     TypeURIVariant = ""
	VariantRequest  TypeURIVariant = "request"
	VariantResponse TypeURIVariant = "response"
)

// TypeURI is a parsed Trust Task Type URI. Obtain one with [ParseTypeURI]; the
// zero value is not a valid Type URI.
//
// TypeURI is comparable, and two values are == exactly when their String forms
// are equal: a Type URI is a name compared by its whole string (§6.1), so the
// authority is part of equality.
type TypeURI struct {
	authority string
	urn       bool
	slug      string
	major     uint32
	minor     uint32
	variant   TypeURIVariant
}

// Authority is everything before `/spec/` (path form) or `:spec:` (URN form),
// verbatim — e.g. "https://trusttasks.org", "did:web:acme.example" or
// "urn:example:tasks".
func (u TypeURI) Authority() string { return u.authority }

// IsURN reports whether u uses the URN form.
func (u TypeURI) IsURN() bool { return u.urn }

// Slug is the specification's slug, '/'-separated in every form — the URN
// form's "acl:grant" is "acl/grant".
func (u TypeURI) Slug() string { return u.slug }

// Major is the MAJOR version.
func (u TypeURI) Major() uint32 { return u.major }

// Minor is the MINOR version.
func (u TypeURI) Minor() uint32 { return u.minor }

// Version is "MAJOR.MINOR".
func (u TypeURI) Version() string {
	return strconv.FormatUint(uint64(u.major), 10) + "." + strconv.FormatUint(uint64(u.minor), 10)
}

// Variant is the fragment, or [VariantNone] when there is none.
func (u TypeURI) Variant() TypeURIVariant { return u.variant }

// IsResponse reports whether u names the success-response variant.
func (u TypeURI) IsResponse() bool { return u.variant == VariantResponse }

// IsRegistry reports whether u is in the public registry's canonical form,
// https://trusttasks.org/spec/….
func (u TypeURI) IsRegistry() bool { return !u.urn && u.authority == registryAuthority }

// Bare returns u without its fragment — the form a schema is dereferenced by.
func (u TypeURI) Bare() TypeURI { return u.WithVariant(VariantNone) }

// WithVariant returns u with its fragment set to v ([VariantNone] strips it).
func (u TypeURI) WithVariant(v TypeURIVariant) TypeURI {
	u.variant = v
	return u
}

// WithVersion returns u at major.minor — the same authority, form, slug and
// variant. A declared-extension chain (SPEC §5.2) names only a version, so this
// is how its links are followed, and why a chain never crosses authorities.
func (u TypeURI) WithVersion(major, minor uint32) TypeURI {
	u.major, u.minor = major, minor
	return u
}

// SameSpecification reports whether u and other name versions of the same
// specification: the same authority, form and slug, whatever their version and
// variant. The same slug under two authorities is two specifications (§6.1).
func (u TypeURI) SameSpecification(other TypeURI) bool {
	return u.authority == other.authority && u.urn == other.urn && u.slug == other.slug
}

// String reproduces the Type URI; for a parsed one, exactly the parsed string.
func (u TypeURI) String() string {
	var b strings.Builder
	b.WriteString(u.authority)
	if u.urn {
		b.WriteString(":spec:")
		b.WriteString(strings.ReplaceAll(u.slug, "/", ":"))
		b.WriteByte(':')
	} else {
		b.WriteString("/spec/")
		b.WriteString(u.slug)
		b.WriteByte('/')
	}
	b.WriteString(u.Version())
	if u.variant != VariantNone {
		b.WriteByte('#')
		b.WriteString(string(u.variant))
	}
	return b.String()
}

// registryAuthority is the public registry's authority (§6.1).
const registryAuthority = "https://trusttasks.org"

// TypeURIErrorKind classifies why a string is not a Type URI.
type TypeURIErrorKind int

// The reasons [ParseTypeURI] refuses a string.
const (
	// TypeURIErrScheme: no scheme, or the scheme is `http`, which §6.1 forbids
	// in every form.
	TypeURIErrScheme TypeURIErrorKind = iota + 1
	// TypeURIErrShape: neither the path form nor (for a `urn:`) the URN form.
	TypeURIErrShape
	// TypeURIErrSlug: the slug fails the §6.1 grammar.
	TypeURIErrSlug
	// TypeURIErrReservedSlug: the slug is in the §6.1 reserved namespace and is
	// not a framework-defined one.
	TypeURIErrReservedSlug
	// TypeURIErrVersion: the version is not MAJOR.MINOR (§5.1).
	TypeURIErrVersion
	// TypeURIErrFragment: a fragment other than #request or #response.
	TypeURIErrFragment
)

// TypeURIError is the error [ParseTypeURI] returns.
type TypeURIError struct {
	Kind TypeURIErrorKind
	// Input is the string that failed to parse.
	Input string
	// Part is the offending component (the slug, version or fragment), when the
	// failure is in one.
	Part string
}

func (e *TypeURIError) Error() string {
	switch e.Kind {
	case TypeURIErrScheme:
		return fmt.Sprintf("trusttasks: not a Trust Task Type URI: %q must have a scheme other than http (SPEC §6.1)", e.Input)
	case TypeURIErrSlug:
		return fmt.Sprintf("trusttasks: Type URI %q: slug %q does not satisfy the SPEC §6.1 grammar", e.Input, e.Part)
	case TypeURIErrReservedSlug:
		return fmt.Sprintf("trusttasks: Type URI %q: slug %q is reserved per SPEC §6.1", e.Input, e.Part)
	case TypeURIErrVersion:
		return fmt.Sprintf("trusttasks: Type URI %q: version %q must be MAJOR.MINOR per SPEC §5.1", e.Input, e.Part)
	case TypeURIErrFragment:
		return fmt.Sprintf("trusttasks: Type URI %q: only #request and #response fragments are permitted (got #%s)", e.Input, e.Part)
	default:
		return fmt.Sprintf("trusttasks: not a Trust Task Type URI: %q must be <prefix>/spec/<slug>/<major.minor>, "+
			"or urn:<nid>:<prefix>:spec:<slug-segments>:<major.minor>", e.Input)
	}
}

// ParseTypeURI parses any Type URI form SPEC §6.1 defines (see [TypeURI]).
// A failure is a *[TypeURIError].
func ParseTypeURI(s string) (TypeURI, error) {
	fail := func(kind TypeURIErrorKind, part string) (TypeURI, error) {
		return TypeURI{}, &TypeURIError{Kind: kind, Input: s, Part: part}
	}

	withoutFragment, fragment, hasFragment := strings.Cut(s, "#")
	variant := VariantNone
	if hasFragment {
		switch fragment {
		case "request":
			variant = VariantRequest
		case "response":
			variant = VariantResponse
		default:
			return fail(TypeURIErrFragment, fragment)
		}
	}

	// SPEC §6.1: any absolute URI, but never `http` — an http Type URI would
	// normalise a transport-downgrade path for consumers that dereference it
	// under §6.2 content negotiation, so it is refused in every form.
	scheme, _, ok := strings.Cut(withoutFragment, ":")
	if !ok || !validScheme(scheme) || strings.EqualFold(scheme, "http") {
		return fail(TypeURIErrScheme, "")
	}

	var authority, slug, version string
	urn := strings.EqualFold(scheme, "urn")
	if urn {
		// urn:<NID>:<prefix>:spec:<seg>:…:<MAJOR.MINOR>. The slug is the
		// segments between the first :spec: and the version.
		prefix, rest, ok := strings.Cut(withoutFragment, ":spec:")
		if !ok {
			return fail(TypeURIErrShape, "")
		}
		parts := strings.SplitN(prefix, ":", 3)
		if len(parts) < 3 || parts[1] == "" || parts[2] == "" || strings.Contains(rest, "/") {
			return fail(TypeURIErrShape, "")
		}
		i := strings.LastIndexByte(rest, ':')
		if i < 0 {
			return fail(TypeURIErrShape, "")
		}
		authority, slug, version = prefix, strings.ReplaceAll(rest[:i], ":", "/"), rest[i+1:]
	} else {
		// <prefix>/spec/<slug>/<MAJOR.MINOR> — an https authority (with an
		// optional path prefix), a DID URL, or any other scheme's prefix.
		specIdx := strings.Index(withoutFragment, "/spec/")
		if specIdx < 0 {
			return fail(TypeURIErrShape, "")
		}
		authority = withoutFragment[:specIdx]
		afterScheme := authority[len(scheme)+1:]
		if strings.TrimLeft(afterScheme, "/") == "" ||
			(strings.EqualFold(scheme, "https") && !strings.HasPrefix(afterScheme, "//")) {
			return fail(TypeURIErrShape, "")
		}
		rest := withoutFragment[specIdx+len("/spec/"):]
		// The version is the final '/' segment; everything before it is the slug.
		i := strings.LastIndexByte(rest, '/')
		if i < 0 {
			return fail(TypeURIErrShape, "")
		}
		slug, version = rest[:i], rest[i+1:]
	}

	if kind := checkSlug(slug); kind != 0 {
		return fail(kind, slug)
	}
	major, minor, ok := parseVersion(version)
	if !ok {
		return fail(TypeURIErrVersion, version)
	}
	return TypeURI{
		authority: authority,
		urn:       urn,
		slug:      slug,
		major:     major,
		minor:     minor,
		variant:   variant,
	}, nil
}

// validScheme is RFC 3986's scheme grammar: ALPHA *( ALPHA / DIGIT / "+" / "-" / "." ).
func validScheme(scheme string) bool {
	if scheme == "" || !isASCIIAlpha(scheme[0]) {
		return false
	}
	for i := 1; i < len(scheme); i++ {
		c := scheme[i]
		if !isASCIIAlpha(c) && !(c >= '0' && c <= '9') && c != '+' && c != '-' && c != '.' {
			return false
		}
	}
	return true
}

func isASCIIAlpha(c byte) bool { return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') }

// parseVersion reads MAJOR.MINOR: two decimal uint32s, neither with a leading
// zero. Only ASCII digits are accepted — no sign.
func parseVersion(s string) (major, minor uint32, ok bool) {
	maj, min, found := strings.Cut(s, ".")
	if !found {
		return 0, 0, false
	}
	parse := func(p string) (uint32, bool) {
		if p == "" || (len(p) > 1 && p[0] == '0') {
			return 0, false
		}
		for i := 0; i < len(p); i++ {
			if p[i] < '0' || p[i] > '9' {
				return 0, false
			}
		}
		n, err := strconv.ParseUint(p, 10, 32)
		return uint32(n), err == nil
	}
	major, okMaj := parse(maj)
	minor, okMin := parse(min)
	return major, minor, okMaj && okMin
}

// checkSlug returns 0 for a valid slug, or why it is not one.
func checkSlug(slug string) TypeURIErrorKind {
	if !validNamespace(slug) {
		return TypeURIErrSlug
	}
	if isReservedNamespace(slug) && !isAllowedFrameworkSlug(slug) {
		return TypeURIErrReservedSlug
	}
	return 0
}

// isReservedNamespace reports whether slug is in the namespace SPEC §6.1
// reserves, ^trust-(task|ceremony)($|-|/) — both halves.
func isReservedNamespace(slug string) bool {
	first, _, _ := strings.Cut(slug, "/")
	for _, stem := range []string{"trust-task", "trust-ceremony"} {
		if first == stem || strings.HasPrefix(first, stem+"-") {
			return true
		}
	}
	return false
}

// isAllowedFrameworkSlug lists the framework slugs permitted inside the
// reservation. It is a hand-maintained copy of
// #/properties/slug/anyOf[1]/enum in specs/spec.meta.schema.json (and of
// is_allowed_framework_slug in trust-tasks-rs); a test reads the meta-schema
// and fails when they disagree.
func isAllowedFrameworkSlug(slug string) bool {
	switch slug {
	case "trust-task",
		"trust-task-error",
		"trust-task-ok",
		"trust-task-next-step",
		"trust-task-discovery",
		"trust-task-control",
		"trust-ceremony-receipt":
		return true
	}
	return false
}
