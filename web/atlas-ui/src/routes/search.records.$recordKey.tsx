import { createFileRoute } from "@tanstack/react-router";
import { validateReadingSearch } from "../app/urlState";
export const Route = createFileRoute("/search/records/$recordKey")({
  validateSearch: validateReadingSearch,
  staticData: { section: "search" },
  component: SearchPage,
});
function SearchPage() {
  return null;
}
