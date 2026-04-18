import type { Pf2eTopLevelArea } from "./area-menu-screen.js";

export const PF2E_TERMINAL_TITLE = "PF2E Terminal";

export const PF2E_APP_AREAS: Pf2eTopLevelArea[] = [
  {
    id: "tag_refinement",
    audience: "dev",
    label: "Tag Refinement",
    description:
      "Review authored queue items and create AI proposal, legacy-seed, legacy-rule, and exemplar-cleanup sessions.",
  },
  {
    id: "ontology_search",
    audience: "user",
    label: "Ontology Search",
    description:
      "Browse ontology-backed domains such as derived tags, categories, and search semantics over the live indexed corpus.",
  },
  {
    id: "search",
    audience: "user",
    label: "Browse/Search",
    description:
      "A keyboard-first workbench for corpus browsing, exact lookup, ranked search, and facet-driven refinement.",
  },
];
