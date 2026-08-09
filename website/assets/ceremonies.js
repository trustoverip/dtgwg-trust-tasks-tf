/* ============================================================
   Trust Tasks — Ceremony definition registry
   ------------------------------------------------------------
   Hand-edited list of *ceremony definitions* under
   `ceremonies/<slug>/<version>/ceremony.json` in the source tree.
   The `definitionPath` is fetched at view time and rendered on the
   Ceremony detail page, the same way BindingSpecPage renders a
   binding's prose — except a definition is JSON rather than
   markdown, so the page renders its structure rather than its text.

   Source, not generated: this file is committed by hand, exactly as
   `bindings.js` and `data.js` are. `npm run build` syncs the
   ceremonies/ tree into website/ceremonies/ but does not write here.
   ============================================================ */

window.TT_CEREMONIES = [
  {
    id: "vtc/member-onboarding/0.1",
    slug: "vtc/member-onboarding",
    version: "0.1",
    title: "VTC Member Onboarding",
    summary:
      "Admit a member to a Verifiable Trust Community: application, decision, and the reciprocal membership credential that closes the bidirectional edge. Composed entirely of Trust Tasks the registry already serves.",
    definitionURI: "https://trusttasks.org/ceremony/vtc/member-onboarding/0.1",
    status: "draft",
    accent: "navy",
    definitionPath: "/ceremonies/vtc/member-onboarding/0.1/ceremony.json",
  },
];
