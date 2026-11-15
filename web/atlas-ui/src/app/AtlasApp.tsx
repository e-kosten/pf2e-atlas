import { RouterProvider } from "@tanstack/react-router";
import { DesktopTabs, windowsTabsEnabled } from "./DesktopTabs";
import { atlasRouter } from "./router";

export function AtlasApp() {
  if (windowsTabsEnabled()) {
    const { pathname, search, hash } = window.location;
    return <DesktopTabs initialHref={pathname + search + hash} />;
  }
  return <RouterProvider router={atlasRouter} />;
}
