import { routeSearch } from "../app/routeSearch";
import { createFileRoute } from "@tanstack/react-router";
export const Route = createFileRoute("/lists/$slug/records/$recordKey")({
  ...routeSearch("list_record"),
  staticData: { section: "lists" },
});
