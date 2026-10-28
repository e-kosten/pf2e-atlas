import { routeSearch } from "../app/routeSearch";
import { createFileRoute } from "@tanstack/react-router";
export const Route = createFileRoute("/search/records/$recordKey")({
  ...routeSearch("search_record"),
  staticData: { section: "search" },
  component: SearchPage,
});
function SearchPage() {
  return null;
}
