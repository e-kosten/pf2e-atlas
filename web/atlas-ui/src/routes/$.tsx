import { createFileRoute } from "@tanstack/react-router";
export const Route = createFileRoute("/$")({
  staticData: { section: "search" },
  component: SearchPage,
});
function SearchPage() {
  return null;
}
