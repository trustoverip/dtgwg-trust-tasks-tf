package trusttasks

import (
	"encoding/json"
	"os"
	"testing"
)

// The shared RFC 3339 vectors (fixtures/consume/timestamps.json at the
// repository root), which the Rust, TypeScript and Dart runtimes also run:
// every field of issuedAt / expiresAt must be in range. time.Parse refuses
// most out-of-range fields itself, but accepts an offset of +24:00 or +23:60.
func TestSharedTimestampVectors(t *testing.T) {
	raw, err := os.ReadFile("../../fixtures/consume/timestamps.json")
	if err != nil {
		t.Fatal(err)
	}
	var vectors struct {
		Valid []struct {
			Value   string `json:"value"`
			EpochMs int64  `json:"epochMs"`
			Why     string `json:"why"`
		} `json:"valid"`
		Invalid []struct {
			Value string `json:"value"`
			Why   string `json:"why"`
		} `json:"invalid"`
	}
	if err := json.Unmarshal(raw, &vectors); err != nil {
		t.Fatal(err)
	}
	if len(vectors.Valid) == 0 || len(vectors.Invalid) == 0 {
		t.Fatal("no vectors")
	}
	for _, v := range vectors.Valid {
		got, err := parseTimestamp(v.Value)
		if err != nil {
			t.Errorf("%s (%s): %v", v.Value, v.Why, err)
			continue
		}
		if got.UnixMilli() != v.EpochMs {
			t.Errorf("%s (%s) = %d ms, want %d", v.Value, v.Why, got.UnixMilli(), v.EpochMs)
		}
	}
	for _, v := range vectors.Invalid {
		if got, err := parseTimestamp(v.Value); err == nil {
			t.Errorf("%s (%s) parsed to %s, want an error", v.Value, v.Why, got)
		}
	}
}
