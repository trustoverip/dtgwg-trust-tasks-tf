package didcomm

import (
	"crypto/ecdh"
	"crypto/rand"
	"encoding/json"
	"strings"
	"testing"
)

// sealed is a JWE built field by field, so a test can seal a correctly
// encrypted envelope whose protected header or JSON structure differs from what
// packAuthcrypt writes.
type sealed struct {
	header     map[string]any   // protected header
	recipients []map[string]any // each entry's "header"; "encrypted_key" is filled in
	extra      map[string]any   // further top-level JWE members
}

// standardHeader is the protected header packAuthcrypt writes for one
// recipient, keyed by name so a test can change any member.
func standardHeader(senderKid string, recipientKids []string, ephemeral *ecdh.PrivateKey) map[string]any {
	return map[string]any{
		"alg":  algAuthcrypt,
		"enc":  encA256CBC,
		"typ":  mediaTypeEncrypted,
		"skid": senderKid,
		"apu":  b64.EncodeToString([]byte(senderKid)),
		"apv":  b64.EncodeToString(computeAPV(recipientKids)),
		"epk":  map[string]any{"kty": "OKP", "crv": "X25519", "x": b64.EncodeToString(ephemeral.PublicKey().Bytes())},
	}
}

// seal encrypts plaintext under s, wrapping the CEK for recipientPub in every
// recipient entry. The KDF reads apu and apv from s.header exactly as written,
// so the result opens only if the receiver derives the same inputs.
func (s sealed) seal(t *testing.T, plaintext []byte, ephemeral, senderPriv *ecdh.PrivateKey, recipientPub *ecdh.PublicKey) []byte {
	t.Helper()
	protectedJSON, err := json.Marshal(s.header)
	if err != nil {
		t.Fatal(err)
	}
	protectedB64 := b64.EncodeToString(protectedJSON)
	cek := make([]byte, cekLen)
	if _, err := rand.Read(cek); err != nil {
		t.Fatal(err)
	}
	iv, ciphertext, tag, err := encryptA256CBCHS512(cek, plaintext, []byte(protectedB64))
	if err != nil {
		t.Fatal(err)
	}
	ze, _ := ephemeral.ECDH(recipientPub)
	zs, _ := senderPriv.ECDH(recipientPub)
	apu, _ := b64.DecodeString(stringOr(s.header["apu"]))
	apv, _ := b64.DecodeString(stringOr(s.header["apv"]))
	kek := concatKDF(append(append([]byte{}, ze...), zs...), []byte(algAuthcrypt), apu, apv, kekBits, tag)
	wrapped, err := aesKeyWrap(kek, cek)
	if err != nil {
		t.Fatal(err)
	}
	recipients := make([]any, 0, len(s.recipients))
	for _, h := range s.recipients {
		recipients = append(recipients, map[string]any{"header": h, "encrypted_key": b64.EncodeToString(wrapped)})
	}
	out := map[string]any{
		"protected":  protectedB64,
		"recipients": recipients,
		"iv":         b64.EncodeToString(iv),
		"ciphertext": b64.EncodeToString(ciphertext),
		"tag":        b64.EncodeToString(tag),
	}
	for k, v := range s.extra {
		out[k] = v
	}
	wire, err := json.Marshal(out)
	if err != nil {
		t.Fatal(err)
	}
	return wire
}

func stringOr(v any) string {
	s, _ := v.(string)
	return s
}

const (
	hdrSenderKid    = "did:example:alice#key-agreement-1"
	hdrRecipientKid = "did:example:bob#key-agreement-1"
)

type headerCase struct {
	sender, recipient, ephemeral *ecdh.PrivateKey
}

func newHeaderCase(t *testing.T) headerCase {
	return headerCase{sender: genX25519(t), recipient: genX25519(t), ephemeral: genX25519(t)}
}

func (c headerCase) standard() sealed {
	return sealed{
		header:     standardHeader(hdrSenderKid, []string{hdrRecipientKid}, c.ephemeral),
		recipients: []map[string]any{{"kid": hdrRecipientKid}},
	}
}

func (c headerCase) open(wire []byte) (*authcrypted, error) {
	return unpackAuthcrypt(wire, c.recipient, hdrRecipientKid, func(skid string) (*ecdh.PublicKey, error) {
		return c.sender.PublicKey(), nil
	})
}

// The helper itself produces an envelope that opens, so each refusal below is
// down to the one thing its case changes.
func TestSealedHelperOpens(t *testing.T) {
	c := newHeaderCase(t)
	got, err := c.open(c.standard().seal(t, []byte(`{}`), c.ephemeral, c.sender, c.recipient.PublicKey()))
	if err != nil {
		t.Fatal(err)
	}
	if got.recipientKid != hdrRecipientKid || got.senderKid != hdrSenderKid {
		t.Errorf("kids = %q, %q", got.senderKid, got.recipientKid)
	}
}

func TestJWEHeaderMembersAreValidated(t *testing.T) {
	cases := map[string]func(s *sealed){
		"recipient entry with no kid": func(s *sealed) {
			s.recipients = []map[string]any{{}}
		},
		"apv names other recipients": func(s *sealed) {
			s.header["apv"] = b64.EncodeToString(computeAPV([]string{"did:example:dave#k"}))
		},
		"apv is not base64url": func(s *sealed) { s.header["apv"] = "not base64!" },
		"apv is absent":        func(s *sealed) { delete(s.header, "apv") },
		"apu names another key": func(s *sealed) {
			s.header["apu"] = b64.EncodeToString([]byte("did:example:carol#key-agreement-1"))
		},
		"apu is not base64url": func(s *sealed) { s.header["apu"] = "not base64!" },
		"apu is absent":        func(s *sealed) { delete(s.header, "apu") },
		"crit":                 func(s *sealed) { s.header["crit"] = []string{"exp"}; s.header["exp"] = 1 },
		"zip":                  func(s *sealed) { s.header["zip"] = "DEF" },
		"typ is not DIDComm":   func(s *sealed) { s.header["typ"] = "JWT" },
		"protected members differing only by case": func(s *sealed) {
			s.header["SKID"] = "did:example:carol#key-agreement-1"
		},
		"shared unprotected header": func(s *sealed) {
			s.extra = map[string]any{"unprotected": map[string]any{"cty": "x"}}
		},
		"aad": func(s *sealed) { s.extra = map[string]any{"aad": b64.EncodeToString([]byte("x"))} },
		"flattened header": func(s *sealed) {
			s.extra = map[string]any{"header": map[string]any{"kid": hdrRecipientKid}}
		},
		"recipient header repeating a protected parameter": func(s *sealed) {
			s.recipients[0]["skid"] = "did:example:carol#key-agreement-1"
		},
		"recipient header carrying crit": func(s *sealed) { s.recipients[0]["crit"] = []string{"x"} },
		"JWE members differing only by case": func(s *sealed) {
			s.extra = map[string]any{"Recipients": []any{}}
		},
	}
	wantDetail := map[string]string{
		"recipient entry with no kid": "carries no kid",
		"apv names other recipients":  "apv does not match",
		"apv is not base64url":        "apv is absent or not base64url",
		"apv is absent":               "apv is absent or not base64url",
		"apu names another key":       "apu does not name the skid",
		"apu is not base64url":        "apu is absent or not base64url",
		"apu is absent":               "apu is absent or not base64url",
		"crit":                        `"crit"`,
		"zip":                         `"zip"`,
		"typ is not DIDComm":          `typ "JWT"`,
		"protected members differing only by case": "protected header member",
		"shared unprotected header":                `"unprotected"`,
		"aad":                                      `"aad"`,
		"flattened header":                         `"header"`,
		"recipient header repeating a protected parameter": `recipient header carries "skid"`,
		"recipient header carrying crit":                   `recipient header carries "crit"`,
		"JWE members differing only by case":               "only by case",
	}
	for name, mutate := range cases {
		t.Run(name, func(t *testing.T) {
			c := newHeaderCase(t)
			s := c.standard()
			mutate(&s)
			_, err := c.open(s.seal(t, []byte(`{}`), c.ephemeral, c.sender, c.recipient.PublicKey()))
			e := assertFailure(t, err, FailNotAuthcrypt)
			if !strings.Contains(e.Detail, wantDetail[name]) {
				t.Errorf("detail = %q, want it to mention %q", e.Detail, wantDetail[name])
			}
		})
	}
}

func TestJWEHeaderAcceptsWhatDIDCommAllows(t *testing.T) {
	cases := map[string]func(s *sealed){
		"no typ": func(s *sealed) { delete(s.header, "typ") },
		"a member JWE does not define": func(s *sealed) {
			s.extra = map[string]any{"x-note": "ignored"}
		},
		"a recipient header parameter the protected header lacks": func(s *sealed) {
			s.recipients[0]["x-hint"] = "ignored"
		},
	}
	for name, mutate := range cases {
		t.Run(name, func(t *testing.T) {
			c := newHeaderCase(t)
			s := c.standard()
			mutate(&s)
			if _, err := c.open(s.seal(t, []byte(`{}`), c.ephemeral, c.sender, c.recipient.PublicKey())); err != nil {
				t.Fatal(err)
			}
		})
	}
}

// Several recipients: apv covers all their kids, sorted, and each opens its own
// entry.
func TestJWEHeaderAPVCoversEveryRecipient(t *testing.T) {
	c := newHeaderCase(t)
	kids := []string{"did:example:zed#k", hdrRecipientKid}
	s := sealed{
		header:     standardHeader(hdrSenderKid, kids, c.ephemeral),
		recipients: []map[string]any{{"kid": kids[0]}, {"kid": kids[1]}},
	}
	got, err := c.open(s.seal(t, []byte(`{}`), c.ephemeral, c.sender, c.recipient.PublicKey()))
	if err != nil {
		t.Fatal(err)
	}
	if got.recipientKid != hdrRecipientKid {
		t.Errorf("recipientKid = %q", got.recipientKid)
	}

	// Dropping the other recipient's entry no longer matches apv.
	s.recipients = s.recipients[1:]
	_, err = c.open(s.seal(t, []byte(`{}`), c.ephemeral, c.sender, c.recipient.PublicKey()))
	assertFailure(t, err, FailNotAuthcrypt)
}

// The per-recipient kid is not covered by the AEAD, so an intermediary can
// rewrite or remove it. Removing it used to leave a single unlabelled entry the
// receiver took to be its own, and the receiver then reported an empty local
// DID. Now the entry is refused, and the local identity is always the one the
// envelope was opened with.
func TestRecipientKidCannotBeStrippedOrRewritten(t *testing.T) {
	alice, alicePub := party(t, "did:example:alice")
	bob, bobPub := party(t, "did:example:bob")
	wire, err := PackTrustTask(echoDoc(defaultID, "hello", nil), alice, bobPub)
	if err != nil {
		t.Fatal(err)
	}
	rewrite := func(kid any) []byte {
		var env map[string]any
		if err := json.Unmarshal(wire, &env); err != nil {
			t.Fatal(err)
		}
		r := env["recipients"].([]any)[0].(map[string]any)
		if kid == nil {
			r["header"] = map[string]any{}
		} else {
			r["header"] = map[string]any{"kid": kid}
		}
		out, _ := json.Marshal(env)
		return out
	}

	_, err = UnpackTrustTask(rewrite(nil), bob, resolverFor(alice.DID, alicePub))
	assertFailure(t, err, FailNotAuthcrypt)

	// Relabelled to another DID: not ours, and apv would not match either.
	_, err = UnpackTrustTask(rewrite("did:example:carol#key-agreement-1"), bob, resolverFor(alice.DID, alicePub))
	assertFailure(t, err, FailNotAuthcrypt)

	unpacked, err := UnpackTrustTask(wire, bob, resolverFor(alice.DID, alicePub))
	if err != nil {
		t.Fatal(err)
	}
	if unpacked.Transport.Local != bob.DID || unpacked.Transport.Peer != alice.DID {
		t.Errorf("transport = %+v", unpacked.Transport)
	}
}

// The plaintext from and to are inside the AEAD; a message whose from is not
// the authenticated sender, or whose to does not name the recipient, is
// refused.
func TestPlaintextFromAndToMustMatchTheEnvelope(t *testing.T) {
	alice, alicePub := party(t, "did:example:alice")
	bob, bobPub := party(t, "did:example:bob")
	seal := func(edit func(m map[string]any)) []byte {
		msg := map[string]any{
			"id":   NewMessageID(),
			"type": EnvelopeType,
			"from": alice.DID,
			"to":   []string{bob.DID},
			"body": json.RawMessage(echoDoc(defaultID, "hello", nil)),
		}
		edit(msg)
		plaintext, _ := json.Marshal(msg)
		wire, err := packAuthcrypt(plaintext, alice.priv, alice.Kid, bobPub.pub, bobPub.Kid)
		if err != nil {
			t.Fatal(err)
		}
		return wire
	}
	open := func(wire []byte) error {
		_, err := UnpackTrustTask(wire, bob, resolverFor(alice.DID, alicePub))
		return err
	}

	if err := open(seal(func(map[string]any) {})); err != nil {
		t.Fatalf("matching from and to: %v", err)
	}
	if err := open(seal(func(m map[string]any) { delete(m, "from"); delete(m, "to") })); err != nil {
		t.Fatalf("no from or to: %v", err)
	}
	if err := open(seal(func(m map[string]any) { m["from"] = alice.Kid })); err != nil {
		t.Fatalf("from as the sender's DID URL: %v", err)
	}

	e := assertFailure(t, open(seal(func(m map[string]any) { m["from"] = "did:example:carol" })), FailDecrypt)
	if !strings.Contains(e.Detail, "not the authenticated sender") {
		t.Errorf("detail = %q", e.Detail)
	}
	assertFailure(t, open(seal(func(m map[string]any) { m["to"] = []string{"did:example:carol"} })), FailNotForThisRecipient)
	assertFailure(t, open(seal(func(m map[string]any) { m["to"] = []string{} })), FailNotForThisRecipient)
}
