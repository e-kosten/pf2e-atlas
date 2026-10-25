import { createFileRoute } from "@tanstack/react-router";
import { ListEditView } from "../features/lists/ListEditView";
export const Route = createFileRoute("/lists/$slug/edit")({
  staticData: { section: "lists" },
  component: Page,
});
function Page() {
  const params = Route.useParams();
  return <ListEditView route={{ slug: params.slug }} />;
}
