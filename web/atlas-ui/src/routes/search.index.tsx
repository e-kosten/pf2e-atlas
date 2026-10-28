import { routeSearch } from "../app/routeSearch";
import { createFileRoute } from "@tanstack/react-router";
export const Route = createFileRoute("/search/")({
  ...routeSearch("search"),
  staticData: { section: "search" },
  component: SearchPage,
});
function SearchPage() {
  return null;
}
