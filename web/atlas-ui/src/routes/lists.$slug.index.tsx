import { createFileRoute } from "@tanstack/react-router";
export const Route = createFileRoute("/lists/$slug/")({
  staticData: { section: "lists" },
});
