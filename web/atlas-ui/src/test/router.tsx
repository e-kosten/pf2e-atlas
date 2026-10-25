import { useLayoutEffect, useState, type ReactNode } from "react";
import { RouterContextProvider } from "@tanstack/react-router";
import { atlasRouter, type AtlasRouter } from "../app/router";

/** Isolated feature tests use the application router without rendering its pages. */
export function RouterTestContext({
  children,
  router = atlasRouter,
}: {
  children: ReactNode;
  router?: AtlasRouter;
}) {
  useState(() => {
    router.updateLatestLocation();
    return null;
  });
  useLayoutEffect(() => {
    const unsubscribe = router.history.subscribe(() => {
      void router.load();
    });
    void router.load();
    return unsubscribe;
  }, [router]);
  return <RouterContextProvider router={router}>{children}</RouterContextProvider>;
}
