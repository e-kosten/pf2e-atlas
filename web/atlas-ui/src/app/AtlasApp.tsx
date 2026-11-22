import { RouterProvider } from "@tanstack/react-router";
import { DesktopTabs } from "./DesktopTabs";
import { windowsTabsEnabled } from "./desktopBridge";
import { atlasRouter } from "./router";

export function AtlasApp() {
  if (windowsTabsEnabled()) {
    const { pathname, search, hash } = window.location;
    return <DesktopTabs initialHref={pathname + search + hash} />;
  }
  return <RouterProvider router={atlasRouter} />;
}
