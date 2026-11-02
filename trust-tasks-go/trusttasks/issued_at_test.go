// #970: every instant this library stamps is written in whole seconds, and
// every RFC 3339 precision is still read. Mirrors trust-tasks-rs's
// tests/issued_at_precision.rs and trust-tasks-ts's test/issued-at.test.ts.
//
// SPEC §13.1 item 3 calls a full-precision timestamp a fingerprint and lets a
// producer round issuedAt down. Go has always stamped whole seconds; this pins
// it, so the four libraries cannot drift apart again.
package trusttasks_test

import (
	"regexp"
	"testing"
	"time"

	tt "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/trusttasks"
)

var wholeSeconds = regexp.MustCompile(`^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$`)

func precisionRequest(issuedAt string) *tt.Document[map[string]any] {
	issuer, recipient := "did:key:z6MkProducer", "did:key:z6MkConsumer"
	return &tt.Document[map[string]any]{
		ID:        "5b0e8f3a-91c2-8d4e-a6f0-2c7d93b1e4a5",
		Type:      "https://real-life.org/trust-tasks/spec/sealed-envelope/0.1",
		Issuer:    &issuer,
		Recipient: &recipient,
		IssuedAt:  &issuedAt,
		Payload:   map[string]any{},
	}
}

func TestStampedInstantsAreWholeSeconds(t *testing.T) {
	if s := tt.SystemClock(); !wholeSeconds.MatchString(s) {
		t.Fatalf("SystemClock() = %q, want whole seconds in UTC", s)
	}
	req := precisionRequest("2026-10-27T18:04:11Z")
	errResp := tt.RejectWith(req, "urn:uuid:2", tt.ErrorPayload{Code: "taskFailed"}, nil)
	if !wholeSeconds.MatchString(*errResp.IssuedAt) {
		t.Fatalf("RejectWith issuedAt = %q", *errResp.IssuedAt)
	}
	resp := tt.RespondWith(req, "urn:uuid:3", map[string]any{}, nil)
	if !wholeSeconds.MatchString(*resp.IssuedAt) {
		t.Fatalf("RespondWith issuedAt = %q", *resp.IssuedAt)
	}
}

func TestEveryRFC3339PrecisionIsRead(t *testing.T) {
	issued := time.Date(2026, 10, 27, 18, 4, 11, 0, time.UTC)
	for _, issuedAt := range []string{
		"2026-10-27T18:04:11Z",
		"2026-10-27T18:04:11.5Z",
		"2026-10-27T18:04:11.720Z",
		"2026-10-27T18:04:11.720263Z",
		"2026-10-27T18:04:11.720263396Z",
		"2026-10-27T20:04:11.720+02:00",
	} {
		if reason := tt.ValidateFreshness(precisionRequest(issuedAt), issued.Add(time.Second), tt.ConsequentialFreshness()); reason != nil {
			t.Errorf("%s: %+v", issuedAt, reason)
		}
	}
}
