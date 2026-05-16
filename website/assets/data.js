/* ============================================================
   Trust Tasks — Taxonomy
   ------------------------------------------------------------
   This file holds the hand-edited taxonomy (categories).
   Trust Task entries (window.TT_TASKS, window.TT_STATS) are
   generated from specs/<slug>/<version>/ by
   scripts/build-registry.mjs and written to
   tasks.generated.js, which is loaded after this file in
   index.html.
   ============================================================ */

window.TT_CATEGORIES = [
  {
    id: "framework",
    name: "Framework",
    color: "navy",
    blurb: "Framework-defined response and meta types that every Trust Task ecosystem reuses (e.g. trust-task-error).",
    icon: "anchor"
  },
  {
    id: "permission",
    name: "Permission",
    color: "violet",
    blurb: "Granting, revoking, and managing access-control privileges between parties.",
    icon: "key"
  }
];
