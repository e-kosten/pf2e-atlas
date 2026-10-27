import { createFileRoute, Outlet, useMatch } from "@tanstack/react-router";
import { ListDetailView } from "../features/lists/ListDetailView";
import {
  stringifySearchParams,
  selectedNavigation,
  readingView,
} from "../app/urlState";

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
  const {
    rank: _rank,
    at: _at,
    focus: _focus,
    record_view: _view,
    record_tab: _tab,
    ...criteria
  } = search;
  if (editor) return <Outlet />;
  return (
    <ListDetailView
      key={slug}
      route={{
        slug,
        query: stringifySearchParams(criteria) || undefined,
        selectedRecordKey: selected?.params.recordKey ?? null,
        selection: selected
          ? selectedNavigation(search, selected.params.recordKey)
          : undefined,
        landing: selected
          ? selectedNavigation(search, selected.params.recordKey, "focus")
          : undefined,
        ...readingView(search),
      }}
    />
  );
}
