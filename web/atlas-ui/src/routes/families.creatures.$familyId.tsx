import { createFileRoute } from "@tanstack/react-router";
import { routeSearch } from "../app/routeSearch";
import { CreatureFamilyPage } from "../features/records/CreatureFamilyPage";

export const Route = createFileRoute("/families/creatures/$familyId")({
  ...routeSearch("empty"),
  staticData: { section: "reference" },
  component: Page,
});

function Page() {
  const { familyId } = Route.useParams();
  return <CreatureFamilyPage key={familyId} familyId={familyId} />;
}
