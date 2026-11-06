import type {
  RemasterDeclarationView,
  RemasterRelationshipsView,
} from "../generated/atlas";
import { rootNavigation } from "../shared/records/useReferenceReading";
import { summaryFixture } from "./fixtures";

export function remasterDeclarationFixture(): RemasterDeclarationView {
  return {
    id: "declaration-1",
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
    origins: ["foundry"],
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
    relationships: [
      {
        id: "relationship-1",
        label: "Renamed to",
        parts: [
          {
            kind: "reference",
            label: "New name",
            navigation: rootNavigation("rules:new"),
          },
        ],
        declaration_ids: ["declaration-1"],
        explanation_links: [],
        truncated: false,
      },
    ],
    declarations: [remasterDeclarationFixture()],
    truncated: false,
  };
}
