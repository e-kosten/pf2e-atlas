import { createFileRoute } from "@tanstack/react-router";
import { DesktopSettings } from "../app/DesktopSettings";

export const Route = createFileRoute("/settings")({
  staticData: { section: "reference" },
  component: DesktopSettings,
});
