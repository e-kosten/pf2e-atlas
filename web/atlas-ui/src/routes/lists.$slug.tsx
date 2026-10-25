import { createFileRoute, Outlet, useMatch } from "@tanstack/react-router";
import { ListDetailView } from "../features/lists/ListDetailView";
import { stringifySearchParams, selectedNavigation } from "../app/urlState";

export const Route = createFileRoute("/lists/$slug")({
  staticData: { section: "lists" },
  component: ListWorkspace,
});

function ListWorkspace() {
  const { slug } = Route.useParams();
  const search = Route.useSearch();
  const selected = useMatch({
    from: "/lists/$slug/records/$recordKey",
    shouldThrow: false,
  });
  const editor = useMatch({ from: "/lists/$slug/edit", shouldThrow: false });
  const { landing: _landing, selection: _selection, ...criteria } = search;
  if (editor) return <Outlet />;
  return (
    <ListDetailView
      key={slug}
      route={{
        slug,
        query: stringifySearchParams(criteria) || undefined,
        selectedRecordKey: selected?.params.recordKey ?? null,
        landing: selected ? selectedNavigation(search, "landing") : undefined,
      }}
    />
  );
}
