package didcomm

import (
	"crypto/sha256"
	"encoding/json"
	"os"
	"sort"
	"strings"
	"testing"
)

// authcrypt-interop-multi.fixture.json is a JWE packed by affinidi to two
// recipients (carol listed first, so the entries are not in apv's sorted
// order). Each recipient opens it, which pins the apv check against a value
// another implementation computed, not only against computeAPV itself.
func TestInteropUnpacksAffinidiMultiRecipientJWE(t *testing.T) {
	raw, err := os.ReadFile("testdata/authcrypt-interop-multi.fixture.json")
	if err != nil {
		t.Fatal(err)
	}
	var f struct {
		JWE             string `json:"jwe"`
		SenderKid       string `json:"senderKid"`
		SenderPubX25519 string `json:"senderPubX25519"`
		Recipients      []struct {
			Kid        string `json:"kid"`
			PrivX25519 string `json:"privX25519"`
		} `json:"recipients"`
	}
	if err := json.Unmarshal(raw, &f); err != nil {
		t.Fatal(err)
	}

	// The protected apv is SHA-256 of the sorted kids joined by ".", as
	// affinidi computed it.
	var env jweJSON
	if err := json.Unmarshal([]byte(f.JWE), &env); err != nil {
		t.Fatal(err)
	}
	protectedJSON, _ := b64.DecodeString(env.Protected)
	var header protectedHeader
	_ = json.Unmarshal(protectedJSON, &header)
	kids := []string{f.Recipients[0].Kid, f.Recipients[1].Kid}
	sort.Strings(kids)
	want := sha256.Sum256([]byte(strings.Join(kids, ".")))
	if header.Apv != b64.EncodeToString(want[:]) {
		t.Fatalf("affinidi apv %s is not SHA-256 of the sorted kids", header.Apv)
	}
	if env.Recipients[0].Header.Kid != f.Recipients[1].Kid {
		t.Fatalf("fixture recipients are in sorted order; the test needs them unsorted")
	}

	senderDID := bareDID(f.SenderKid)
	resolve := func(did string) (*PeerKey, error) {
		if did != senderDID {
			return nil, errUnknownSender(did)
		}
		return NewPeerKey(f.SenderKid, mustHex(t, f.SenderPubX25519))
	}
	for _, r := range f.Recipients {
		t.Run(r.Kid, func(t *testing.T) {
			me, err := NewIdentity(bareDID(r.Kid), r.Kid, mustHex(t, r.PrivX25519))
			if err != nil {
				t.Fatal(err)
			}
			unpacked, err := UnpackTrustTask([]byte(f.JWE), me, resolve)
			if err != nil {
				t.Fatalf("unpack: %v", err)
			}
			if unpacked.Transport.Local != me.DID || unpacked.Transport.Peer != senderDID {
				t.Errorf("transport = %+v", unpacked.Transport)
			}
		})
	}

	// Dropping either entry breaks apv.
	var full map[string]any
	_ = json.Unmarshal([]byte(f.JWE), &full)
	full["recipients"] = full["recipients"].([]any)[1:]
	trimmed, _ := json.Marshal(full)
	bob, _ := NewIdentity(bareDID(f.Recipients[0].Kid), f.Recipients[0].Kid, mustHex(t, f.Recipients[0].PrivX25519))
	_, err = UnpackTrustTask(trimmed, bob, resolve)
	assertFailure(t, err, FailNotAuthcrypt)
}
