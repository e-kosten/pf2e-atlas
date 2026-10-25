import { createFileRoute } from "@tanstack/react-router";
export const Route = createFileRoute("/search/")({
  staticData: { section: "search" },
  component: SearchPage,
});
function SearchPage() {
  return null;
}
