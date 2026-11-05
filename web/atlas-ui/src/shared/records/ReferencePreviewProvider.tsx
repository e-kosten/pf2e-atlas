import { useEffect, type ReactNode } from "react";
import { atlasRouter } from "../../app/router";
import { recordDestination } from "../../app/destinations";
import { navigateDestination } from "../../app/navigation";
import { ReferencePreviewContext } from "./ReferencePreviewContext";
import { RecordReferenceDrawer } from "./RecordReferenceDrawer";
import { useRecordPreview } from "./useRecordPreview";

/** Contextual links outside an existing reader share one reference session. */
export function ReferencePreviewProvider({ children }: { children: ReactNode }) {
  const preview = useRecordPreview();
  const { active, close } = preview;
  useEffect(() => {
    if (!active) return;
    return atlasRouter.subscribe("onBeforeNavigate", ({ fromLocation, toLocation }) => {
      if (fromLocation && fromLocation.pathname !== toLocation.pathname) close();
    });
  }, [active, close]);
  return (
    <ReferencePreviewContext value={preview}>
      {children}
      {preview.active && (
        <RecordReferenceDrawer
          trait={preview.trait}
          onTraitReference={preview.openTrait}
          anchor={preview.anchor}
          requestedNavigation={preview.selection}
          readingHistory={preview.readingHistory}
          landing={preview.landing}
          detail={preview.detail}
          loading={preview.loading}
          error={preview.error}
          onRetry={preview.retry}
          onClose={preview.close}
          onReference={preview.open}
          onRankChange={preview.changeRank}
          onOpenFullPage={() => {
            if (!preview.recordKey) return;
            navigateDestination(
              recordDestination({
                recordKey: preview.recordKey,
                selection: preview.selection,
                landing: preview.landing,
              }),
            );
            preview.close();
          }}
        />
      )}
    </ReferencePreviewContext>
  );
}
