import { createFileRoute } from "@tanstack/react-router";
import { routeSearch } from "../app/routeSearch";
import { EquipmentFamilyPage } from "../features/records/EquipmentFamilyPage";

export const Route = createFileRoute("/families/equipment/$familyId")({
  ...routeSearch("empty"),
  staticData: { section: "reference" },
  component: Page,
});

function Page() {
  const { familyId } = Route.useParams();
  return <EquipmentFamilyPage key={familyId} familyId={familyId} />;
}
