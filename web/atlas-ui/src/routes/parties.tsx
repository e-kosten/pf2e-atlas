import { createFileRoute, Outlet, useParams } from "@tanstack/react-router";
import { PartiesWorkspace } from "../features/parties/PartiesPage";

export const Route = createFileRoute("/parties")({
  staticData: { section: "characters" },
  component: Workspace,
});

function Workspace() {
  const { partyKey } = useParams({ strict: false });
  return (
    <PartiesWorkspace selected={partyKey}>
      <Outlet />
    </PartiesWorkspace>
  );
}
