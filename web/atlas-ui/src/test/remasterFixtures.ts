import type {
  RemasterDeclarationView,
  RemasterRelationshipsView,
} from "../generated/atlas";
import { rootNavigation } from "../shared/records/useReferenceReading";
import { summaryFixture } from "./fixtures";

export function remasterDeclarationFixture(): RemasterDeclarationView {
  return {
    id: "declaration-1",
    heading: "Renamed in the remaster",
    summary: "The current name is New name.",
    kinds: ["renamed"],
    authored_status: null,
    context: ["Class features"],
    before: [
      {
        text: "Old name",
        terms: [],
        endpoints: [
          {
            label: "Old name",
            context: null,
            resolution: {
              kind: "current",
              record: summaryFixture("rules:old", "Old name"),
              navigation: rootNavigation("rules:old"),
            },
          },
        ],
        truncated: false,
      },
    ],
    after: [
      {
        text: "New name",
        terms: [],
        endpoints: [
          {
            label: "New name",
            context: null,
            resolution: {
              kind: "current",
              record: summaryFixture("rules:new", "New name"),
              navigation: rootNavigation("rules:new"),
            },
          },
        ],
        truncated: false,
      },
    ],
    citations: [
      {
        origin: "foundry",
        attribution: "Remaster Changes — Class Features",
        revision: "upstream-revision",
        statement: "Old name was renamed to New name.",
        shortened: false,
        destination: { kind: "attribution", reason: null },
      },
    ],
    citations_truncated: false,
    truncated: false,
  };
}
export function remasterInformationFixture(): RemasterRelationshipsView {
  return {
    notices: [
      {
        declaration_id: "declaration-1",
        heading: "Renamed in the remaster",
        description: "The current name is New name.",
        links: [{ label: "New name", navigation: rootNavigation("rules:new") }],
        attribution: null,
      },
    ],
    declarations: [remasterDeclarationFixture()],
    truncated: false,
  };
}
