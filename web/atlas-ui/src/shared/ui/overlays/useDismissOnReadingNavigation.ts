import { useEffect, useEffectEvent } from "react";
import { atlasRouter } from "../../../app/router";

/** An explicit reading departure closes overlays, including for the same href. */
export function useDismissOnReadingNavigation(open: boolean, dismiss: () => void) {
  const onNavigate = useEffectEvent(dismiss);
  useEffect(() => {
    if (!open) return;
    return atlasRouter.subscribe("onBeforeNavigate", ({ fromLocation, toLocation }) => {
      const intent = toLocation.state.atlasReadingDismissal;
      if (intent && intent !== fromLocation?.state.atlasReadingDismissal) onNavigate();
    });
  }, [open]);
}
