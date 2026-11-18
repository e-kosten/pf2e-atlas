import { createFileRoute } from "@tanstack/react-router";
import { routeSearch } from "../app/routeSearch";
import { CharacterPage } from "../features/parties/CharacterPage";

export const Route = createFileRoute("/characters/$characterKey/")({
  ...routeSearch("empty"),
  staticData: { section: "encounters" },
  component: Page,
});
function Page() {
  const { characterKey } = Route.useParams();
  return <CharacterPage characterKey={characterKey} />;
}
