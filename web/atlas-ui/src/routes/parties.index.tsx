import { createFileRoute } from "@tanstack/react-router";
import { routeSearch } from "../app/routeSearch";
import { PartiesPage } from "../features/parties/PartiesPage";

export const Route = createFileRoute("/parties/")({
  ...routeSearch("empty"),
  staticData: { section: "characters" },
  component: PartiesPage,
});
