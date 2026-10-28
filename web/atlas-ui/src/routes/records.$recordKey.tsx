import { routeSearch } from "../app/routeSearch";
import { createFileRoute, useLocation } from "@tanstack/react-router";
import { RecordView } from "../features/records/RecordViews";
import { documentSection, selectedNavigation, readingView } from "../app/urlState";
export const Route = createFileRoute("/records/$recordKey")({
  ...routeSearch("record"),
  staticData: { section: "reference" },
  component: Page,
});
function Page() {
  const params = Route.useParams();
  const search = Route.useSearch();
  const section = useLocation({ select: (location) => documentSection(location.href) });
  return (
    <RecordView
      key={params.recordKey}
      route={{
        recordKey: params.recordKey,
        selection: selectedNavigation(search, params.recordKey),
        landing: selectedNavigation(search, params.recordKey, "focus"),
        ...readingView(search),
        section: section || undefined,
      }}
    />
  );
}
