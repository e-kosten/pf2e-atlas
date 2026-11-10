import { routeSearch } from "../app/routeSearch";
import { createFileRoute } from "@tanstack/react-router";
export const Route = createFileRoute("/lists/$slug/edit")({
  ...routeSearch("empty"),
  staticData: { section: "lists" },
});
