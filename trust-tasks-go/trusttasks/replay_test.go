// SPEC.md §7.2 item 11, §4.2 freshness, and canonicalization conformance tests.
//
// Mirror trust-tasks-ts/test/replay.test.ts and the corresponding cases in
// trust-tasks-rs/src/replay.rs.
package trusttasks_test

import (
	"context"
	"encoding/json"
	"errors"
	"strings"
	"testing"
	"time"

	tt "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/trusttasks"
)

// consequential runs the pipeline with item 11 enforced against guard, counting
// how many times the handler actually executed.
func consequential(
	t *testing.T,
	guard tt.ReplayGuard,
	d *tt.Document[payload],
	executions *int,
	now time.Time,
) tt.ConsumeOutcome[response] {
	t.Helper()
	outcome, err := tt.ConsumeInbound(context.Background(), tt.ConsumeOptions[payload, response]{
		Transport:     tt.UnauthenticatedTransport{},
		Spec:          relaxedSpec,
		ProofPolicy:   tt.ProofPolicy{Kind: tt.ProofAcceptUnverified},
		PayloadPolicy: tt.PayloadPolicy{Kind: tt.PayloadAcceptUnvalidated},
		Checks:        tt.ConsequentialChecks(guard),
		Doc:           d,
		MyVID:         me,
		Now:           now,
		NewErrorID:    func() string { return "err-1" },
		Clock:         fixedClock,
		Handler: func(_ context.Context, accepted *tt.Document[payload], _ tt.ResolvedParties) (*tt.Document[response], error) {
			*executions++
			return tt.RespondWith[payload, response](accepted, "resp-1", response{OK: true}, fixedClock), nil
		},
	})
	if err != nil {
		t.Fatalf("ConsumeInbound: %v", err)
	}
	return outcome
}

// consequentialDoc carries the issuedAt that ConsequentialChecks requires.
func consequentialDoc(mutate func(*tt.Document[payload])) *tt.Document[payload] {
	return doc(func(d *tt.Document[payload]) {
		d.Recipient = nil
		d.IssuedAt = ptr("2026-01-01T00:00:00Z")
		if mutate != nil {
			mutate(d)
		}
	})
}

func TestItem11AbsorbsABitForBitResendAndNeverExecutesTwice(t *testing.T) {
	guard := tt.NewInMemoryReplayGuard(0)
	executions := 0

	first := consequential(t, guard, consequentialDoc(nil), &executions, testNow)
	if first.Kind != tt.OutcomeHandled {
		t.Fatalf("expected handled, got %q", first.Kind)
	}

	// §8.4: a retry is a bit-for-bit identical resend.
	second := consequential(t, guard, consequentialDoc(nil), &executions, testNow)
	if second.Kind != tt.OutcomeDuplicate {
		t.Fatalf("expected duplicate, got %q", second.Kind)
	}
	if executions != 1 {
		t.Fatalf("the consequential effect happened %d times", executions)
	}
	// §7.2 (*Disposition of a duplicate*): return the prior response.
	if second.PriorResponse == nil {
		t.Fatal("expected the prior response to be returned")
	}
	var prior tt.Document[response]
	if err := json.Unmarshal(second.PriorResponse, &prior); err != nil {
		t.Fatalf("prior response is not a document: %v", err)
	}
	if prior.ID != "resp-1" {
		t.Errorf("expected the first execution's response, got %q", prior.ID)
	}
}

func TestItem11RejectsDifferingContentUnderAReusedIDWithIDConflict(t *testing.T) {
	guard := tt.NewInMemoryReplayGuard(0)
	executions := 0

	consequential(t, guard, consequentialDoc(nil), &executions, testNow)
	escalated := consequentialDoc(func(d *tt.Document[payload]) { d.Payload = payload{Role: "owner"} })
	outcome := consequential(t, guard, escalated, &executions, testNow)

	if outcome.Kind != tt.OutcomeRejected {
		t.Fatalf("expected rejected, got %q", outcome.Kind)
	}
	if outcome.Error.Payload.Code != string(tt.CodeIDConflict) {
		t.Errorf("expected idConflict, got %q", outcome.Error.Payload.Code)
	}
	if executions != 1 {
		t.Errorf("the escalated document executed: %d executions", executions)
	}
}

func TestItem11TreatsAReSignedProofOverIdenticalContentAsAConflict(t *testing.T) {
	// §7.2: "a re-signed proof over identical content makes a different document
	// — that is the idConflict case, and the distinction is the whole point."
	withProof := func(value string) *tt.Document[payload] {
		return consequentialDoc(func(d *tt.Document[payload]) {
			signed := *testProof
			signed.Cryptosuite = "eddsa-jcs-2022"
			signed.ProofValue = value
			d.Proof = &signed
		})
	}
	a, err := tt.DocumentDigest(withProof("zAAA"))
	if err != nil {
		t.Fatalf("digest: %v", err)
	}
	b, err := tt.DocumentDigest(withProof("zBBB"))
	if err != nil {
		t.Fatalf("digest: %v", err)
	}
	if a == b {
		t.Error("expected a re-signed proof to change the document identity")
	}
}

func TestItem11DoesNotBurnTheIDOfADocumentRefusedBeforeTheClaim(t *testing.T) {
	guard := tt.NewInMemoryReplayGuard(0)
	executions := 0

	// Refused by §7.2 item 4 — expiry — which runs well before the claim.
	refused := consequentialDoc(func(d *tt.Document[payload]) {
		d.ExpiresAt = ptr("2025-01-01T00:00:00Z")
	})
	if outcome := consequential(t, guard, refused, &executions, testNow); outcome.Kind != tt.OutcomeRejected {
		t.Fatalf("expected rejected, got %q", outcome.Kind)
	}
	if guard.Size() != 0 {
		t.Fatalf("expected no record for a document that was never accepted, got %d", guard.Size())
	}

	// The corrected resend under the same id must not come back as idConflict.
	corrected := consequentialDoc(nil)
	if outcome := consequential(t, guard, corrected, &executions, testNow); outcome.Kind != tt.OutcomeHandled {
		t.Fatalf("expected the corrected resend to be handled, got %q", outcome.Kind)
	}
}

func TestItem11RefusesADocumentItCannotPlaceInAnyWindow(t *testing.T) {
	// §7.2 (*Bounding the record*): no expiresAt and no usable age means no
	// window, and a consequential task MUST NOT execute.
	guard := tt.NewInMemoryReplayGuard(0)
	executions := 0
	unbounded := tt.ConsumeChecks{
		// A window-less freshness policy alongside a guard: the combination §7.2
		// forbids, and the one this check exists to catch.
		Freshness: tt.DefaultFreshness(),
		Replay:    tt.ReplayPolicy{Kind: tt.ReplayGuarded, Guard: guard},
	}

	outcome, err := tt.ConsumeInbound(context.Background(), tt.ConsumeOptions[payload, response]{
		Transport:     tt.UnauthenticatedTransport{},
		Spec:          relaxedSpec,
		ProofPolicy:   tt.ProofPolicy{Kind: tt.ProofAcceptUnverified},
		PayloadPolicy: tt.PayloadPolicy{Kind: tt.PayloadAcceptUnvalidated},
		Checks:        unbounded,
		Doc:           doc(func(d *tt.Document[payload]) { d.Recipient = nil }),
		MyVID:         me,
		Now:           testNow,
		NewErrorID:    func() string { return "err-1" },
		Clock:         fixedClock,
		Handler: func(_ context.Context, accepted *tt.Document[payload], _ tt.ResolvedParties) (*tt.Document[response], error) {
			executions++
			return tt.RespondWith[payload, response](accepted, "resp-1", response{OK: true}, fixedClock), nil
		},
	})
	if err != nil {
		t.Fatalf("ConsumeInbound: %v", err)
	}
	if outcome.Kind != tt.OutcomeRejected {
		t.Fatalf("expected rejected, got %q", outcome.Kind)
	}
	if outcome.Error.Payload.Code != string(tt.CodeExpired) {
		t.Errorf("expected expired, got %q", outcome.Error.Payload.Code)
	}
	if executions != 0 {
		t.Errorf("executed a document it could not place in a window")
	}
}

// failingGuard cannot consult its record.
type failingGuard struct{}

func (failingGuard) Claim(context.Context, string, string, time.Time, time.Time) (tt.ReplayVerdict, error) {
	return tt.ReplayVerdict{}, errors.New("redis://cache.internal:6379 connection refused")
}
func (failingGuard) RecordResponse(context.Context, string, json.RawMessage) error { return nil }
func (failingGuard) Release(context.Context, string, string) error                 { return nil }

func TestItem11FailsClosedWhenTheRecordCannotBeConsulted(t *testing.T) {
	executions := 0
	outcome := consequential(t, failingGuard{}, consequentialDoc(nil), &executions, testNow)

	if outcome.Kind != tt.OutcomeRejected {
		t.Fatalf("expected rejected, got %q", outcome.Kind)
	}
	if outcome.Error.Payload.Code != string(tt.CodeUnavailable) {
		t.Errorf("expected unavailable, got %q", outcome.Error.Payload.Code)
	}
	if !outcome.Error.Payload.Retryable {
		t.Error("expected the refusal to be retryable — the resend will be absorbed once the store is back")
	}
	if executions != 0 {
		t.Error("executed without satisfying item 11")
	}
	// §10.4: the store's hostname must not reach the wire.
	if message := *outcome.Error.Payload.Message; strings.Contains(message, "redis") {
		t.Errorf("leaked the store's connection string: %q", message)
	}
}

func TestItem11KeepsNoRecordWhenTheCallerDeclaresTheTaskNotConsequential(t *testing.T) {
	executions := 0
	for i := 0; i < 2; i++ {
		outcome, err := tt.ConsumeInbound(context.Background(), tt.ConsumeOptions[payload, response]{
			Transport:     tt.UnauthenticatedTransport{},
			Spec:          relaxedSpec,
			ProofPolicy:   tt.ProofPolicy{Kind: tt.ProofAcceptUnverified},
			PayloadPolicy: tt.PayloadPolicy{Kind: tt.PayloadAcceptUnvalidated},
			Checks:        tt.NotConsequentialChecks(),
			Doc:           doc(func(d *tt.Document[payload]) { d.Recipient = nil }),
			MyVID:         me,
			Now:           testNow,
			NewErrorID:    func() string { return "err-1" },
			Clock:         fixedClock,
			Handler: func(_ context.Context, accepted *tt.Document[payload], _ tt.ResolvedParties) (*tt.Document[response], error) {
				executions++
				return tt.RespondWith[payload, response](accepted, "resp-1", response{OK: true}, fixedClock), nil
			},
		})
		if err != nil {
			t.Fatalf("ConsumeInbound: %v", err)
		}
		if outcome.Kind != tt.OutcomeHandled {
			t.Fatalf("expected handled, got %q", outcome.Kind)
		}
	}
	if executions != 2 {
		t.Errorf("expected both deliveries to execute, got %d", executions)
	}
}

/* ── InMemoryReplayGuard ─────────────────────────────────────────────────── */

func TestInMemoryGuardEvictsTheLeastRecentlyUsedRecordAtCapacity(t *testing.T) {
	guard := tt.NewInMemoryReplayGuard(2)
	ctx := context.Background()
	deadline := testNow.Add(time.Hour)

	mustClaim := func(id, digest string) tt.ReplayVerdict {
		t.Helper()
		verdict, err := guard.Claim(ctx, id, digest, deadline, testNow)
		if err != nil {
			t.Fatalf("claim: %v", err)
		}
		return verdict
	}

	mustClaim("a", "d1")
	mustClaim("b", "d2")
	mustClaim("a", "d1") // touch a, making b the least recently used
	mustClaim("c", "d3") // evicts b

	if guard.Size() != 2 {
		t.Fatalf("expected 2 records, got %d", guard.Size())
	}
	// `a` was touched and so survived; assert that before re-claiming `b`, which
	// would itself evict something.
	if verdict := mustClaim("a", "d1"); verdict.Kind != tt.VerdictDuplicate {
		t.Errorf("expected a to have been retained, got %q", verdict.Kind)
	}
	if verdict := mustClaim("b", "d2"); verdict.Kind != tt.VerdictFresh {
		t.Errorf("expected b to have been evicted, got %q", verdict.Kind)
	}
}

func TestInMemoryGuardDoesNotLetAConflictDisplaceTheRecordItConflictsWith(t *testing.T) {
	guard := tt.NewInMemoryReplayGuard(2)
	ctx := context.Background()
	deadline := testNow.Add(time.Hour)

	_, _ = guard.Claim(ctx, "a", "d1", deadline, testNow)
	_, _ = guard.Claim(ctx, "b", "d2", deadline, testNow)
	// A flood of conflicts on `a` must not refresh its recency and evict `b`.
	for i := 0; i < 5; i++ {
		verdict, _ := guard.Claim(ctx, "a", "other", deadline, testNow)
		if verdict.Kind != tt.VerdictConflict {
			t.Fatalf("expected conflict, got %q", verdict.Kind)
		}
	}
	_, _ = guard.Claim(ctx, "c", "d3", deadline, testNow) // evicts the LRU, which is a
	if verdict, _ := guard.Claim(ctx, "b", "d2", deadline, testNow); verdict.Kind != tt.VerdictDuplicate {
		t.Errorf("expected b retained, got %q", verdict.Kind)
	}
}

func TestInMemoryGuardTreatsARecordPastItsDeadlineAsAbsent(t *testing.T) {
	guard := tt.NewInMemoryReplayGuard(0)
	ctx := context.Background()
	deadline := testNow.Add(time.Minute)

	_, _ = guard.Claim(ctx, "a", "d1", deadline, testNow)
	later := testNow.Add(2 * time.Minute)
	verdict, err := guard.Claim(ctx, "a", "different", deadline, later)
	if err != nil {
		t.Fatalf("claim: %v", err)
	}
	if verdict.Kind != tt.VerdictFresh {
		t.Errorf("expected an expired record to be treated as absent, got %q", verdict.Kind)
	}
}

func TestInMemoryGuardReleasesAnUnfinishedClaimButNotACompletedOne(t *testing.T) {
	guard := tt.NewInMemoryReplayGuard(0)
	ctx := context.Background()
	deadline := testNow.Add(time.Hour)

	_, _ = guard.Claim(ctx, "a", "d1", deadline, testNow)
	if err := guard.Release(ctx, "a", "d1"); err != nil {
		t.Fatalf("release: %v", err)
	}
	if guard.Size() != 0 {
		t.Errorf("expected the unfinished claim released, %d records remain", guard.Size())
	}

	_, _ = guard.Claim(ctx, "b", "d2", deadline, testNow)
	_ = guard.RecordResponse(ctx, "b", json.RawMessage(`{"ok":true}`))
	if err := guard.Release(ctx, "b", "d2"); err != nil {
		t.Fatalf("release: %v", err)
	}
	if guard.Size() != 1 {
		t.Errorf("expected a completed claim to survive release, %d records remain", guard.Size())
	}

	// A different document's cleanup must not take the key away.
	_, _ = guard.Claim(ctx, "c", "d3", deadline, testNow)
	if err := guard.Release(ctx, "c", "some-other-digest"); err != nil {
		t.Fatalf("release: %v", err)
	}
	if guard.Size() != 2 {
		t.Errorf("another document's release dropped a live claim")
	}
}

func TestInMemoryGuardNeverRetainsNothing(t *testing.T) {
	// A guard that retains nothing answers fresh to everything, which is a silent
	// total defeat of item 11 rather than a visible misconfiguration.
	for _, capacity := range []int{0, -1} {
		guard := tt.NewInMemoryReplayGuard(capacity)
		ctx := context.Background()
		deadline := testNow.Add(time.Hour)
		_, _ = guard.Claim(ctx, "a", "d1", deadline, testNow)
		verdict, _ := guard.Claim(ctx, "a", "d1", deadline, testNow)
		if verdict.Kind != tt.VerdictDuplicate {
			t.Errorf("capacity %d retained no record", capacity)
		}
	}
}

func TestInMemoryGuardPurgesExpiredRecords(t *testing.T) {
	guard := tt.NewInMemoryReplayGuard(0)
	ctx := context.Background()
	_, _ = guard.Claim(ctx, "a", "d1", testNow.Add(time.Minute), testNow)
	_, _ = guard.Claim(ctx, "b", "d2", testNow.Add(time.Hour), testNow)
	guard.PurgeExpired(testNow.Add(2 * time.Minute))
	if guard.Size() != 1 {
		t.Errorf("expected one record purged, %d remain", guard.Size())
	}
}

/* ── SPEC §4.2 / §7.2 — freshness bounds ─────────────────────────────────── */

func freshnessDoc(issuedAt, expiresAt *string) *tt.Document[payload] {
	return doc(func(d *tt.Document[payload]) {
		d.IssuedAt = issuedAt
		d.ExpiresAt = expiresAt
	})
}

func TestFreshnessRejectsAnIssuedAtBeyondTheSkewTolerance(t *testing.T) {
	policy := tt.DefaultFreshness()

	inside := freshnessDoc(ptr("2026-01-01T00:00:30Z"), nil) // 30s ahead, inside 60s skew
	if reason := tt.ValidateFreshness(inside, testNow, policy); reason != nil {
		t.Errorf("expected a document inside the skew to pass, got %q", reason.Message)
	}

	outside := freshnessDoc(ptr("2026-01-01T00:05:00Z"), nil) // 5m ahead
	reason := tt.ValidateFreshness(outside, testNow, policy)
	if reason == nil {
		t.Fatal("expected a future-dated document to be rejected")
	}
	if reason.Code != tt.CodeMalformedRequest || reason.Message != tt.FutureIssuedAt {
		t.Errorf("expected the FutureIssuedAt reason, got %q / %q", reason.Code, reason.Message)
	}
	// §10.4: the message must not render the consumer's clock or tolerance.
	if strings.Contains(reason.Message, "2026") || strings.Contains(reason.Message, "60") {
		t.Errorf("future-dated message leaked consumer state: %q", reason.Message)
	}
}

func TestFreshnessRejectsAnExpiresAtAtOrBeforeIssuedAt(t *testing.T) {
	policy := tt.DefaultFreshness()
	same := freshnessDoc(ptr("2026-01-01T00:00:00Z"), ptr("2026-01-01T00:00:00Z"))
	reason := tt.ValidateFreshness(same, testNow, policy)
	if reason == nil || reason.Message != tt.ExpiryNotAfterIssuance {
		t.Fatalf("expected an empty validity interval to be rejected, got %v", reason)
	}
}

func TestFreshnessBoundsTheAcceptanceWindowWithMaxAge(t *testing.T) {
	policy := tt.ConsequentialFreshness()

	recent := freshnessDoc(ptr("2025-12-31T23:58:00Z"), nil) // 2m old, inside 5m
	if reason := tt.ValidateFreshness(recent, testNow, policy); reason != nil {
		t.Errorf("expected a recent document to pass, got %q", reason.Message)
	}

	old := freshnessDoc(ptr("2025-12-31T23:00:00Z"), nil) // an hour old
	reason := tt.ValidateFreshness(old, testNow, policy)
	if reason == nil || reason.Code != tt.CodeExpired || reason.Message != tt.StaleWireMessage {
		t.Fatalf("expected a stale document to be rejected, got %v", reason)
	}
}

func TestFreshnessRefusesADocumentWithNoTimestampOnceAWindowIsConfigured(t *testing.T) {
	windowed := tt.FreshnessPolicy{Skew: tt.DefaultSkew, MaxAge: tt.DefaultMaxAge}
	reason := tt.ValidateFreshness(freshnessDoc(nil, nil), testNow, windowed)
	if reason == nil || reason.Code != tt.CodeExpired {
		t.Fatalf("expected a document with no timestamp to be refused, got %v", reason)
	}

	// An expiresAt is enough to place it, even with no issuedAt.
	placed := freshnessDoc(nil, ptr("2026-01-01T01:00:00Z"))
	if reason := tt.ValidateFreshness(placed, testNow, windowed); reason != nil {
		t.Errorf("expected expiresAt to place the document, got %q", reason.Message)
	}
}

func TestFreshnessRequireIssuedAtRejectsADocumentWithout(t *testing.T) {
	policy := tt.ConsequentialFreshness()
	reason := tt.ValidateFreshness(freshnessDoc(nil, ptr("2026-01-01T01:00:00Z")), testNow, policy)
	if reason == nil || reason.Message != tt.IssuedAtRequired {
		t.Fatalf("expected IssuedAtRequired, got %v", reason)
	}
}

func TestRecordExpiryPrefersExpiresAtThenIssuedAtPlusWindow(t *testing.T) {
	policy := tt.ConsequentialFreshness()

	stated := freshnessDoc(ptr("2026-01-01T00:00:00Z"), ptr("2026-01-01T02:00:00Z"))
	got, bounded := tt.RecordExpiry(stated, policy, testNow)
	if !bounded || !got.Equal(time.Date(2026, 1, 1, 2, 0, 0, 0, time.UTC)) {
		t.Errorf("expected expiresAt to fix the retention deadline, got %v (%v)", got, bounded)
	}

	aged := freshnessDoc(ptr("2026-01-01T00:00:00Z"), nil)
	got, bounded = tt.RecordExpiry(aged, policy, testNow)
	if !bounded || !got.Equal(testNow.Add(tt.DefaultMaxAge)) {
		t.Errorf("expected issuedAt + MaxAge, got %v (%v)", got, bounded)
	}

	// No window at all: the caller must not execute a consequential task.
	if _, bounded := tt.RecordExpiry(freshnessDoc(nil, nil), tt.DefaultFreshness(), testNow); bounded {
		t.Error("expected an unbounded policy to report no retention deadline")
	}
}

/* ── Canonicalization and digest ─────────────────────────────────────────── */

func TestSHA256MatchesThePublishedVectors(t *testing.T) {
	cases := []struct{ input, want string }{
		{"", "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"},
		{"abc", "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"},
		{
			"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
			"248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1",
		},
	}
	for _, c := range cases {
		if got := tt.SHA256Hex([]byte(c.input)); got != c.want {
			t.Errorf("SHA256Hex(%q) = %s, want %s", c.input, got, c.want)
		}
	}
}

func TestCanonicalJSONOrdersMembersAndIgnoresWhitespace(t *testing.T) {
	a, err := tt.CanonicalJSON([]byte(`{ "b" : 2, "a" : [ 1, {"d":4,"c":3} ] }`))
	if err != nil {
		t.Fatalf("canonicalize: %v", err)
	}
	if string(a) != `{"a":[1,{"c":3,"d":4}],"b":2}` {
		t.Errorf("unexpected canonical form: %s", a)
	}

	b, err := tt.CanonicalJSON([]byte(`{"a":[1,{"c":3,"d":4}],"b":2}`))
	if err != nil {
		t.Fatalf("canonicalize: %v", err)
	}
	if string(a) != string(b) {
		t.Errorf("member order changed the canonical form: %s vs %s", a, b)
	}
}

func TestCanonicalJSONAppliesOnlyTheJCSEscapeSet(t *testing.T) {
	// Escapes are assembled from `bs` rather than written literally, so this
	// source file carries none of the sequences the test is about.
	bs := `\`
	lineSeparator := string(rune(0x2028))

	// encoding/json would render <, > and & as six-character Unicode escapes for
	// HTML safety, and U+2028 for JavaScript safety. JCS does neither, and a
	// runtime that did would disagree with the Rust and TypeScript ones about what
	// a document canonicalizes to.
	got, err := tt.CanonicalJSON([]byte(`{"k":"a<b>c&d` + bs + `u2028e"}`))
	if err != nil {
		t.Fatalf("canonicalize: %v", err)
	}
	if !strings.Contains(string(got), "a<b>c&d"+lineSeparator+"e") {
		t.Errorf("canonical form escaped characters JCS leaves alone: %q", got)
	}
	if strings.Contains(string(got), bs+"u003c") || strings.Contains(string(got), bs+"u2028") {
		t.Errorf("canonical form carried a non-JCS escape: %q", got)
	}

	// Control characters are escaped, and only as JCS spells them.
	want := `{"k":"a` + bs + `u0001b` + bs + `tc"}`
	got, err = tt.CanonicalJSON([]byte(want))
	if err != nil {
		t.Fatalf("canonicalize: %v", err)
	}
	if string(got) != want {
		t.Errorf("unexpected control-character escaping: got %q, want %q", got, want)
	}
}

func TestDocumentDigestIsStableAcrossMemberOrder(t *testing.T) {
	first, err := tt.DocumentDigest(consequentialDoc(nil))
	if err != nil {
		t.Fatalf("digest: %v", err)
	}
	second, err := tt.DocumentDigest(consequentialDoc(nil))
	if err != nil {
		t.Fatalf("digest: %v", err)
	}
	if first != second {
		t.Errorf("the same document produced two digests: %s vs %s", first, second)
	}

	changed, err := tt.DocumentDigest(consequentialDoc(func(d *tt.Document[payload]) {
		d.Payload = payload{Role: "owner"}
	}))
	if err != nil {
		t.Fatalf("digest: %v", err)
	}
	if changed == first {
		t.Error("a different payload produced the same digest")
	}
}
