import { routeSearch } from "../app/routeSearch";
import { createFileRoute } from "@tanstack/react-router";
import { EncounterDetailView } from "../features/encounters/EncounterDetailView";
import { encounterDestination } from "../app/destinations";
export const Route = createFileRoute("/encounters/$slug/edit")({
  ...routeSearch("empty"),
  staticData: { section: "encounters" },
  component: Page,
});
function Page() {
  const params = Route.useParams();
  const navigate = Route.useNavigate();
  return (
    <EncounterDetailView
      route={{ slug: params.slug }}
      onCloseDirectEditor={(slug) =>
        void navigate({
          ...encounterDestination({ slug }),
          replace: true,
          resetScroll: false,
        })
      }
    />
  );
}
