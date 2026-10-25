import { createFileRoute } from "@tanstack/react-router";
import { EncounterIndexView } from "../features/encounters/EncounterIndexView";
export const Route = createFileRoute("/encounters/")({
  staticData: { section: "encounters" },
  component: Page,
});
function Page() {
  return <EncounterIndexView route={{}} />;
}
