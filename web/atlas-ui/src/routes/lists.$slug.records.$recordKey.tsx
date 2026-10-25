import { createFileRoute } from "@tanstack/react-router";
import { validateReadingSearch } from "../app/urlState";
export const Route = createFileRoute("/lists/$slug/records/$recordKey")({
  validateSearch: validateReadingSearch,
  staticData: { section: "lists" },
});
