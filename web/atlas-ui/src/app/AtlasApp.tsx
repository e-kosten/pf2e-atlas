import { RouterProvider } from "@tanstack/react-router";
import { atlasRouter } from "./router";
export function AtlasApp() {
  return <RouterProvider router={atlasRouter} />;
}
