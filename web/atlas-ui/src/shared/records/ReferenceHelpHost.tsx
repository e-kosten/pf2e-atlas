import { createContext, useContext, useEffect, type ReactNode } from "react";
import { atlasRouter } from "../../app/router";
import {
  ReferenceHelpProvider,
  useReferenceHelpHandler,
  useReferenceRecordNavigation,
} from "./ReferenceHelp";
import { RecordReferenceDrawer } from "./RecordReferenceDrawer";
import { useRecordPreview } from "./useRecordPreview";

const PreviewSession = createContext<ReturnType<typeof useRecordPreview> | null>(null);
export const useReferenceHelpSession = () => useContext(PreviewSession);

type HelpHostProps = {
  children: ReactNode;
  generation?: string;
  visibleOwnerRecordKey?: string;
  recordActions?: (
    record: import("../../generated/atlas").RecordSummaryView,
  ) => ReactNode;
};

/** Hosts without a selected record compose the same contextual reading owner. */
export function ReferenceHelpHost({
  children,
  generation,
  recordActions,
  visibleOwnerRecordKey,
}: HelpHostProps) {
  const inherited = useReferenceHelpSession();
  const onHelp = useReferenceHelpHandler();
  const onReference = useReferenceRecordNavigation();
  return (inherited || onHelp || onReference) &&
    !recordActions &&
    visibleOwnerRecordKey === undefined ? (
    <ReferenceHelpProvider generation={generation}>{children}</ReferenceHelpProvider>
  ) : (
    <LocalReferenceHelpHost
      generation={generation}
      recordActions={recordActions}
      visibleOwnerRecordKey={visibleOwnerRecordKey}
    >
      {children}
    </LocalReferenceHelpHost>
  );
}

function LocalReferenceHelpHost({
  children,
  generation,
  recordActions,
  visibleOwnerRecordKey,
}: HelpHostProps) {
  const preview = useRecordPreview();
  const { destination, closeForNavigation } = preview;
  useEffect(() => {
    if (!destination) return;
    return atlasRouter.subscribe("onBeforeNavigate", ({ fromLocation, toLocation }) => {
      if (fromLocation && fromLocation.pathname !== toLocation.pathname)
        closeForNavigation();
    });
  }, [destination, closeForNavigation]);
  return (
    <PreviewSession value={preview}>
      <ReferenceHelpProvider
        onHelp={preview.openHelp}
        onReference={preview.open}
        generation={generation}
      >
        {children}
        {preview.destination && (
          <RecordReferenceDrawer
            actions={preview.detail && recordActions?.(preview.detail.record)}
            destination={preview.destination}
            definition={preview.definition}
            help={preview.help}
            onHelp={preview.openHelp}
            anchor={preview.anchor}
            visibleOwnerRecordKey={visibleOwnerRecordKey}
            detail={preview.detail}
            loading={preview.loading}
            error={preview.error}
            onRetry={preview.retry}
            onReload={preview.reload}
            retryLabel={preview.retryLabel}
            reloadLabel={preview.reloadLabel}
            onClose={preview.close}
            onNavigateAway={preview.closeForNavigation}
            restoreAnchorFocus={false}
            onReference={preview.open}
            onRankChange={preview.changeRank}
            readingHistory={preview.readingHistory}
            requestedNavigation={preview.selection}
            landing={preview.landing}
            onOpenFullPage={preview.close}
          />
        )}
      </ReferenceHelpProvider>
    </PreviewSession>
  );
}
