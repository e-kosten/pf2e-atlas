import { createRootRoute } from "@tanstack/react-router";
import { AtlasLayout, InvalidRoute } from "../app/AtlasLayout";
import type { AtlasSearchParams } from "../app/urlState";
export const Route = createRootRoute({
  validateSearch: (search: AtlasSearchParams) => search,
  component: AtlasLayout,
  errorComponent: ({ error }) => (
    <InvalidRoute
      message={error instanceof Error ? error.message : "Invalid navigation URL"}
    />
  ),
});
