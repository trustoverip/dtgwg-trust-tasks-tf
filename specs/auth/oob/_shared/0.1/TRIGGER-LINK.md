# auth/oob — binding note: the `sign-in` trigger link

This note says how a starter hands an `auth/oob` request to an approver. It is
a binding of the family to the **trigger link** of the Verifiable Trust
Infrastructure specification, chapter *Trigger Links* (`VTI-LNK-*`), and
uses the link's registered `sign-in` flow. It does not restate that chapter:
the link's form, how a reader parses it, its outcomes and messages, and the
producer and host rules are VTI-LNK's, and this note only says how the
`auth/oob` members fill it.

It replaces the `trusttasks://oob?v=1&svc=…&id=…` scheme of the original
design. That scheme is not defined and **MUST NOT** be emitted.

## 1. The link

For `purpose: login` with `mode: scan`, the starter shows:

```
https://<link host>/t#_from=<service DID>&_id=<requestId>&_exp=<claimDeadline>&_type=/vti/flow/sign-in/0.1
```

| Parameter | Value |
|---|---|
| `_from` | The service's DID: the `recipient` of the `auth/oob/request` that opened the request. Percent-encode `&`, `=`, `#` and `%`, and nothing else. |
| `_id` | The `requestId` from the `auth/oob/request` response, as received (VTI-LNK-033, VTI-LNK-103). |
| `_exp` | The `claimDeadline` from the same response, as UTC epoch seconds. VTI-LNK-100 caps it at 300 s after the code is made; this family recommends 120 s. |
| `_type` | The `sign-in` flow, `/vti/flow/sign-in/0.1` (path form, VTI-LNK-042). |

The **link host** is configuration, defaulting to `link.trustoverip.org`, and
**MUST NOT** be on the portal's own domain (VTI-LNK-084). The link carries
nothing else: no endpoint, no purpose and no origin. A reader takes the
transport from the service's verified DID document (VTI-LNK-053) and selects
the service's existing Trust Task HTTPS service by `type`.

The producer emits ASCII only and stays within the size budget of VTI-LNK-081
at QR error-correction level M. Render rules — level M, no logo, a quiet zone
of at least 4 modules, at least 4 CSS px per module, dark on light, never
inverted — are VTI-LNK's rendering guidance.

## 2. The code is also a link

The page wraps the QR code in `<a href="…">` carrying the same `https` text
(VTI-LNK-086). There is no separate "open in your wallet" link. On a phone the
link host's association opens the wallet; on a desktop with a browser plugin the
plugin handles the click; with neither, the click lands on the link host's page
for people with no wallet (VTI-LNK-091). Pages that carry the link follow
VTI-LNK-082.

## 3. The service's DID document

For `sign-in`, the service's DID document lists a `SignInPortal` service
(VTI-LNK-102):

```json
{ "id": "<did>#sign-in-portal", "type": "SignInPortal", "serviceEndpoint": "https://<portal host>/<path>" }
```

The **portal origin** is the origin of that `serviceEndpoint`. It is the
`origin` the service returns in the `auth/oob/claim` response, and the `origin`
the approver signs into `auth/oob/grant`. A reader that saw the click compares
the page's origin with it (VTI-LNK-105).

## 4. From the link to the first task

The reader follows VTI-LNK-050 to VTI-LNK-056 and the `sign-in` rules
(VTI-LNK-100 to VTI-LNK-105). Only after the person continues does the approver
generate its lock key `K_a` and send [`auth/oob/claim`](../../claim/0.1/spec.md),
which is the first request of VTI-LNK-054:

- `issuer` is `K_a`, fresh for this exchange;
- `recipient` is the service DID (`_from`);
- `id` is unique;
- `parentThreadId` is the handle (`_id`), equal to `payload.requestId`;
- it is signed by `K_a`.

From there the exchange is the `auth/oob` family: claim, prove (carrying
identify), respond (carrying grant), with the starter redeeming.

## 5. Handle security

The `requestId` is a handle, not a credential (VTI-LNK-070). Possession of it
lets a party *claim* the request and nothing more: the first claim locks it to
that party's `K_a`, the starter sees the claim at once, and only a member who
can see the starter's screen can pass `prove`. Fetching or previewing the link
spends nothing (VTI-LNK-072).
