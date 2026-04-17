import type { Pf2eTopLevelArea } from "./area-menu-screen.js";

export const PF2E_TERMINAL_TITLE = "PF2E Terminal";

export const PF2E_APP_AREAS: Pf2eTopLevelArea[] = [
  {
    id: "tag_refinement",
    audience: "dev",
    label: "Tag Refinement",
    description: "Review authored queue items and create AI proposal, legacy-seed, legacy-rule, and exemplar-cleanup sessions.",
  },
  {
    id: "ontology_search",
    audience: "user",
    label: "Ontology Search",
    description: "Browse category -> family -> tag -> record and inspect how derived tags map onto live indexed records.",
  },
  {
    id: "search",
    audience: "user",
    label: "Search",
    description: "User-facing lookup and search over the same indexed PF2E data surfaced by the MCP server.",
  },
];
