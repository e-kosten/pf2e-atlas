import { createFileRoute, useMatch } from "@tanstack/react-router";
import { listDestination } from "../app/destinations";
import { ListDetailView } from "../features/lists/ListDetailView";
import {
  stringifySearchParams,
  selectedNavigation,
  readingView,
} from "../app/urlState";
import { failedFilterTransport } from "../app/routeSearch";

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
  const resetting = failedFilterTransport(selected ? "list_record" : "list", search);
  const {
    rank: _rank,
    at: _at,
    focus: _focus,
    record_view: _view,
    record_tab: _tab,
    ...criteria
  } = search;
  const navigate = Route.useNavigate();
  return (
    <ListDetailView
      key={slug}
      onCloseDirectEditor={
        editor
          ? (canonicalSlug) =>
              void navigate({
                ...listDestination({ slug: canonicalSlug, selectedRecordKey: null }),
                replace: true,
                resetScroll: false,
              })
          : undefined
      }
      route={{
        slug,
        query: stringifySearchParams(criteria) || undefined,
        selectedRecordKey: !resetting ? (selected?.params.recordKey ?? null) : null,
        selection:
          selected && !resetting
            ? selectedNavigation(search, selected.params.recordKey)
            : undefined,
        landing:
          selected && !resetting
            ? selectedNavigation(search, selected.params.recordKey, "focus")
            : undefined,
        ...(!resetting ? readingView(search) : {}),
      }}
    />
  );
}
