package trusttasks

import (
	"encoding/json"
	"os"
	"reflect"
	"testing"
	"time"
)

// The shared acceptance-window vectors (fixtures/consume/freshness.json at the
// repository root), which the Rust, TypeScript and Dart runtimes also run:
// item 4's expiry check, then the freshness bound, then the record expiry, so
// that the four runtimes reach the same verdict on the same document and keep
// its replay record for the same time (SPEC §7.2, *Bounding the record*).

type freshnessVectorPolicy struct {
	SkewMs          int64  `json:"skewMs"`
	MaxAgeMs        *int64 `json:"maxAgeMs"`
	ExpiryHorizonMs *int64 `json:"expiryHorizonMs"`
	RequireIssuedAt bool   `json:"requireIssuedAt"`
}

func (v freshnessVectorPolicy) policy() FreshnessPolicy {
	p := FreshnessPolicy{
		Skew:            time.Duration(v.SkewMs) * time.Millisecond,
		RequireIssuedAt: v.RequireIssuedAt,
	}
	if v.MaxAgeMs != nil {
		// Go spells "no maximum age" as zero; the vectors never set a zero one.
		p.MaxAge = time.Duration(*v.MaxAgeMs) * time.Millisecond
	}
	if v.ExpiryHorizonMs != nil {
		p.ExpiryHorizon = ptrTo(time.Duration(*v.ExpiryHorizonMs) * time.Millisecond)
	}
	return p
}

type freshnessVectors struct {
	Policies map[string]freshnessVectorPolicy `json:"policies"`
	Cases    []struct {
		Name         string           `json:"name"`
		Policy       string           `json:"policy"`
		Now          string           `json:"now"`
		IssuedAt     *string          `json:"issuedAt"`
		ExpiresAt    *string          `json:"expiresAt"`
		Verdict      string           `json:"verdict"`
		RecordExpiry *json.RawMessage `json:"recordExpiry"`
	} `json:"cases"`
}

func loadFreshnessVectors(t *testing.T) freshnessVectors {
	t.Helper()
	raw, err := os.ReadFile("../../fixtures/consume/freshness.json")
	if err != nil {
		t.Fatal(err)
	}
	var vectors freshnessVectors
	if err := json.Unmarshal(raw, &vectors); err != nil {
		t.Fatal(err)
	}
	if len(vectors.Cases) == 0 {
		t.Fatal("no vectors")
	}
	return vectors
}

func TestFreshnessVectorPoliciesAreTheLibraryDefaults(t *testing.T) {
	vectors := loadFreshnessVectors(t)
	if got, want := vectors.Policies["default"].policy(), DefaultFreshness(); !reflect.DeepEqual(got, want) {
		t.Errorf("default = %+v, want %+v", got, want)
	}
	if got, want := vectors.Policies["consequential"].policy(), ConsequentialFreshness(); !reflect.DeepEqual(got, want) {
		t.Errorf("consequential = %+v, want %+v", got, want)
	}
}

func TestSharedFreshnessVectors(t *testing.T) {
	vectors := loadFreshnessVectors(t)
	for _, c := range vectors.Cases {
		t.Run(c.Name, func(t *testing.T) {
			vp, ok := vectors.Policies[c.Policy]
			if !ok {
				t.Fatalf("unknown policy %q", c.Policy)
			}
			policy := vp.policy()
			now, err := time.Parse(time.RFC3339Nano, c.Now)
			if err != nil {
				t.Fatal(err)
			}
			doc := &Document[json.RawMessage]{
				ID:        "req-1",
				Type:      "https://trusttasks.org/spec/acl/show/0.2",
				Payload:   json.RawMessage(`{}`),
				IssuedAt:  c.IssuedAt,
				ExpiresAt: c.ExpiresAt,
			}

			reason := ValidateBasic(doc, now, "")
			if reason == nil {
				reason = ValidateFreshness(doc, now, policy)
			}
			got := "accept"
			if reason != nil {
				got = string(reason.Code)
			}
			if got != c.Verdict {
				t.Fatalf("verdict = %s (%v), want %s", got, reason, c.Verdict)
			}
			if reason != nil {
				return
			}

			retainUntil, bounded := RecordExpiry(doc, policy, now)
			if c.RecordExpiry == nil || string(*c.RecordExpiry) == "null" {
				if bounded {
					t.Errorf("record expiry = %s, want unbounded", retainUntil)
				}
				return
			}
			var s string
			if err := json.Unmarshal(*c.RecordExpiry, &s); err != nil {
				t.Fatal(err)
			}
			want, err := time.Parse(time.RFC3339Nano, s)
			if err != nil {
				t.Fatal(err)
			}
			if !bounded || !retainUntil.Equal(want) {
				t.Errorf("record expiry = %s (bounded %v), want %s", retainUntil, bounded, want)
			}
		})
	}
}
