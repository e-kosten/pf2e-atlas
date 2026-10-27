import { createFileRoute } from "@tanstack/react-router";
import { LinkNotFound } from "../app/InvalidUrl";
export const Route = createFileRoute("/$")({
  staticData: { section: "reference" },
  component: LinkNotFound,
});
