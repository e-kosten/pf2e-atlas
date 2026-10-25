import { createFileRoute } from "@tanstack/react-router";
import { ListIndexView } from "../features/lists/ListIndexView";
export const Route = createFileRoute("/lists/")({
  staticData: { section: "lists" },
  component: Page,
});
function Page() {
  return <ListIndexView route={{}} />;
}
