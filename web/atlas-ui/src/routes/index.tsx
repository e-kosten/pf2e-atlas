import { createFileRoute, redirect } from "@tanstack/react-router";
import { canonicalRouteSearch } from "../app/routeSearch";
import { stringifySearchParams } from "../app/urlState";
export const Route = createFileRoute("/")({
  staticData: { section: "search" },
  beforeLoad: ({ location }) => {
    const search = canonicalRouteSearch("search", location.search);
    const hash = location.href.includes("#")
      ? location.href.slice(location.href.indexOf("#"))
      : "";
    throw redirect({
      href: "/search" + stringifySearchParams(search) + hash,
      replace: true,
      state: true,
      resetScroll: false,
    });
  },
  component: SearchPage,
});
function SearchPage() {
  return null;
}
