import type { RemasterRelationshipsView } from "../generated/atlas";
import { rootNavigation } from "../shared/records/useReferenceReading";
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
        explanation_links: [],
        truncated: false,
      },
    ],
    truncated: false,
  };
}
