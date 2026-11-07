import { createFileRoute } from "@tanstack/react-router";
import { routeSearch } from "../app/routeSearch";
import { DefinitionView } from "../features/records/DefinitionView";

export const Route = createFileRoute("/definitions/$definitionKey")({
  ...routeSearch("definition"),
  staticData: { section: "reference" },
  component: Page,
});
function Page() {
  const { definitionKey } = Route.useParams();
  return <DefinitionView definitionKey={definitionKey} />;
}
