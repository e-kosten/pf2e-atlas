import { createFileRoute } from "@tanstack/react-router";
import { routeSearch } from "../app/routeSearch";
import { PartyPage } from "../features/parties/PartyPage";

export const Route = createFileRoute("/parties/$partyKey/")({
  ...routeSearch("empty"),
  staticData: { section: "encounters" },
  component: Page,
});
function Page() {
  const { partyKey } = Route.useParams();
  return <PartyPage partyKey={partyKey} />;
}
