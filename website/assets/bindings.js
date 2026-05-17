/* ============================================================
   Trust Tasks — Transport binding registry
   ------------------------------------------------------------
   Hand-edited list of *transport binding* specifications under
   `bindings/<slug>/<version>/spec.md` in the source tree. The
   `prosePath` is fetched at view time and rendered as markdown
   on the Binding detail page, the same way SpecPage renders
   Trust Task specs.
   ============================================================ */

window.TT_BINDINGS = [
  {
    id: "https/0.1",
    slug: "https",
    version: "0.1",
    title: "HTTPS",
    summary:
      "Carries Trust Task documents as JSON over HTTP/1.1 POST to a single endpoint. Transport-authenticated sender identity comes from a bearer-token mapping to a VID; the standard error codes map to HTTP statuses informatively.",
    bindingURI: "https://trusttasks.org/binding/https/0.1",
    envelopeType: null,
    status: "draft",
    accent: "teal",
    prosePath: "/bindings/https/0.1/spec.md",
    implementations: [
      {
        name: "trust-tasks-https",
        href: "https://github.com/trustoverip/dtgwg-trust-tasks-tf/tree/main/trust-tasks-https",
        language: "Rust",
      },
    ],
  },
  {
    id: "didcomm/0.1",
    slug: "didcomm",
    version: "0.1",
    title: "DIDComm v2.1",
    summary:
      "Carries Trust Task documents inside DIDComm v2.1 authcrypt envelopes. The verified sender_kid maps to the framework's transport-authenticated party identity; authcrypt provides end-to-end integrity and sender authentication so in-band proof is optional.",
    bindingURI: "https://trusttasks.org/binding/didcomm/0.1",
    envelopeType: "https://trusttasks.org/binding/didcomm/0.1/envelope",
    status: "draft",
    accent: "coral",
    prosePath: "/bindings/didcomm/0.1/spec.md",
    implementations: [
      {
        name: "trust-tasks-didcomm",
        href: "https://github.com/trustoverip/dtgwg-trust-tasks-tf/tree/main/trust-tasks-didcomm",
        language: "Rust",
      },
    ],
  },
];
