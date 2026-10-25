import { createFileRoute } from "@tanstack/react-router";
import { EncounterDetailView } from "../features/encounters/EncounterDetailView";
export const Route = createFileRoute("/encounters/$slug/")({
  staticData: { section: "encounters" },
  component: Page,
});
function Page() {
  const params = Route.useParams();
  return <EncounterDetailView route={{ slug: params.slug }} />;
}
