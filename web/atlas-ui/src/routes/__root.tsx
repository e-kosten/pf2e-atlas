import { createRootRoute } from "@tanstack/react-router";
import { AtlasLayout } from "../app/AtlasLayout";
import { InvalidUrl } from "../app/InvalidUrl";
import type { AtlasSearchParams } from "../app/urlState";
export const Route = createRootRoute({
  validateSearch: (search: AtlasSearchParams) => search,
  component: AtlasLayout,
  errorComponent: ({ error }) => (
    <AtlasLayout routeError={<InvalidUrl error={error} />} />
  ),
});
