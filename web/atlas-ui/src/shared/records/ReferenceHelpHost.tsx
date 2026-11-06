import { createContext, useContext, type ReactNode } from "react";
import { ReferenceHelpProvider } from "./ReferenceHelp";
import { RecordReferenceDrawer } from "./RecordReferenceDrawer";
import { useRecordPreview } from "./useRecordPreview";

const PreviewSession = createContext<ReturnType<typeof useRecordPreview> | null>(null);
export const useReferenceHelpSession = () => useContext(PreviewSession);

type HelpHostProps = {
  children: ReactNode;
  generation?: string;
  recordActions?: (
    record: import("../../generated/atlas").RecordSummaryView,
  ) => ReactNode;
};

/** Hosts without a selected record compose the same contextual reading owner. */
export function ReferenceHelpHost({
  children,
  generation,
  recordActions,
}: HelpHostProps) {
  const inherited = useReferenceHelpSession();
  return inherited ? (
    <>{children}</>
  ) : (
    <LocalReferenceHelpHost generation={generation} recordActions={recordActions}>
      {children}
    </LocalReferenceHelpHost>
  );
}

function LocalReferenceHelpHost({
  children,
  generation,
  recordActions,
}: HelpHostProps) {
  const preview = useRecordPreview();
  return (
    <PreviewSession value={preview}>
      <ReferenceHelpProvider onHelp={preview.openHelp} generation={generation}>
        {children}
        {preview.destination && (
          <RecordReferenceDrawer
            actions={preview.detail && recordActions?.(preview.detail.record)}
            destination={preview.destination}
            definition={preview.definition}
            help={preview.help}
            onHelp={preview.openHelp}
            anchor={preview.anchor}
            detail={preview.detail}
            loading={preview.loading}
            error={preview.error}
            onRetry={preview.retry}
            onReload={preview.reload}
            retryLabel={preview.retryLabel}
            reloadLabel={preview.reloadLabel}
            onClose={preview.close}
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
