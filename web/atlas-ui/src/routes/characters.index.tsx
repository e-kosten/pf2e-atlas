import { createFileRoute } from "@tanstack/react-router";
import { routeSearch } from "../app/routeSearch";
import { CharactersPage } from "../features/parties/CharactersPage";

export const Route = createFileRoute("/characters/")({
  ...routeSearch("empty"),
  staticData: { section: "encounters" },
  component: CharactersPage,
});
