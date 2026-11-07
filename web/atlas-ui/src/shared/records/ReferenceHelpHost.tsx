import { useEffect, type ReactNode } from "react";
import { atlasRouter } from "../../app/router";
import {
  ReferenceHelpProvider,
  useReferenceHelpHandler,
  useReferenceRecordNavigation,
} from "./ReferenceHelp";
import { RecordPreviewDrawer } from "./RecordPreviewDrawer";
import { useRecordPreview } from "./useRecordPreview";

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
  const onHelp = useReferenceHelpHandler();
  const onReference = useReferenceRecordNavigation();
  return (onHelp || onReference) &&
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
    <ReferenceHelpProvider
      onHelp={preview.openHelp}
      onReference={preview.open}
      generation={generation}
    >
      {children}
      <RecordPreviewDrawer
        preview={preview}
        actions={preview.detail && recordActions?.(preview.detail.record)}
        visibleOwnerRecordKey={visibleOwnerRecordKey}
        restoreAnchorFocus={false}
      />
    </ReferenceHelpProvider>
  );
}
