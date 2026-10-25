import { createFileRoute } from "@tanstack/react-router";
import { TraitView } from "../features/records/TraitView";
import { stringifySearchParams } from "../app/urlState";
export const Route = createFileRoute("/traits/$traitKey")({
  staticData: { section: "reference" },
  component: Page,
});
function Page() {
  const params = Route.useParams();
  const search = Route.useSearch();
  return (
    <TraitView
      key={params.traitKey}
      route={{
        traitKey: params.traitKey,
        query: stringifySearchParams(search) || undefined,
      }}
    />
  );
}
