import { routeSearch } from "../app/routeSearch";
import { createFileRoute } from "@tanstack/react-router";
export const Route = createFileRoute("/lists/$slug/")({
  ...routeSearch("list"),
  staticData: { section: "lists" },
});
