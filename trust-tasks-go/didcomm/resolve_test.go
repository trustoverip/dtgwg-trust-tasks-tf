package didcomm

import (
	"crypto/ecdh"
	"testing"
)

// A resolver that answers (nil, nil) for a sender it does not know refuses the
// envelope as unresolved rather than failing inside the key agreement.
func TestResolverReturningNilKeyIsUnresolved(t *testing.T) {
	alice, _ := party(t, "did:example:alice")
	bob, bobPub := party(t, "did:example:bob")
	wire, err := PackTrustTask(echoDoc(defaultID, "hello", nil), alice, bobPub)
	if err != nil {
		t.Fatal(err)
	}
	nilResolver := func(string) (*PeerKey, error) { return nil, nil }
	_, err = UnpackTrustTask(wire, bob, nilResolver)
	e := assertFailure(t, err, FailSenderUnresolved)
	if e.Sender != alice.Kid {
		t.Fatalf("sender = %q, want %q", e.Sender, alice.Kid)
	}
}

// A PeerKey not built with NewPeerKey carries no key and is refused the same way.
func TestResolverReturningZeroPeerKeyIsUnresolved(t *testing.T) {
	alice, _ := party(t, "did:example:alice")
	bob, bobPub := party(t, "did:example:bob")
	wire, err := PackTrustTask(echoDoc(defaultID, "hello", nil), alice, bobPub)
	if err != nil {
		t.Fatal(err)
	}
	zeroResolver := func(string) (*PeerKey, error) { return &PeerKey{Kid: alice.Kid}, nil }
	_, err = UnpackTrustTask(wire, bob, zeroResolver)
	assertFailure(t, err, FailSenderUnresolved)
}

// The envelope layer itself refuses a nil sender key, whatever resolver sits
// above it.
func TestUnpackAuthcryptNilSenderKeyIsUnresolved(t *testing.T) {
	alice, _ := party(t, "did:example:alice")
	bob, bobPub := party(t, "did:example:bob")
	wire, err := PackTrustTask(echoDoc(defaultID, "hello", nil), alice, bobPub)
	if err != nil {
		t.Fatal(err)
	}
	_, err = unpackAuthcrypt(wire, bob.priv, bob.Kid, func(string) (*ecdh.PublicKey, error) { return nil, nil })
	assertFailure(t, err, FailSenderUnresolved)
}

// Packing to a recipient key that a resolver failed to supply is an error, not
// a panic — for PackTrustTask and for PackReply, which a caller feeds from the
// same resolver as Receive.
func TestPackWithoutRecipientKeyIsError(t *testing.T) {
	alice, _ := party(t, "did:example:alice")
	doc := echoDoc(defaultID, "hello", nil)
	for name, recipient := range map[string]*PeerKey{
		"nil":  nil,
		"zero": {Kid: "did:example:bob#key-agreement-1"},
	} {
		if _, err := PackTrustTask(doc, alice, recipient); err == nil {
			t.Fatalf("PackTrustTask with %s recipient succeeded", name)
		}
		if _, err := PackReply(alice, recipient, doc); err == nil {
			t.Fatalf("PackReply with %s recipient succeeded", name)
		}
	}
}

func TestPackWithoutSenderIdentityIsError(t *testing.T) {
	_, bobPub := party(t, "did:example:bob")
	doc := echoDoc(defaultID, "hello", nil)
	for name, sender := range map[string]*Identity{
		"nil":  nil,
		"zero": {DID: "did:example:alice", Kid: "did:example:alice#key-agreement-1"},
	} {
		if _, err := PackTrustTask(doc, sender, bobPub); err == nil {
			t.Fatalf("PackTrustTask with %s sender succeeded", name)
		}
	}
}
