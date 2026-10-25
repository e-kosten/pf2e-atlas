import { createFileRoute } from "@tanstack/react-router";
import { EncounterEditView } from "../features/encounters/EncounterEditView";
export const Route = createFileRoute("/encounters/$slug/edit")({
  staticData: { section: "encounters" },
  component: Page,
});
function Page() {
  const params = Route.useParams();
  return <EncounterEditView route={{ slug: params.slug }} />;
}
