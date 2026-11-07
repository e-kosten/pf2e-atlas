import { RecordReferenceDrawer } from "./RecordReferenceDrawer";
import type { RecordPreviewContentProps } from "./recordPreviewTypes";
import type { useRecordPreview } from "./useRecordPreview";

export function RecordPreviewDrawer({
  preview,
  ...options
}: Pick<
  RecordPreviewContentProps,
  "actions" | "context" | "restoreAnchorFocus" | "visibleOwnerRecordKey"
> & {
  preview: ReturnType<typeof useRecordPreview>;
}) {
  if (!preview.destination) return null;
  return (
    <RecordReferenceDrawer
      {...options}
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
      onNavigateAway={preview.closeForNavigation}
      onReference={preview.open}
      onRankChange={preview.changeRank}
      readingHistory={preview.readingHistory}
      requestedNavigation={preview.selection}
      landing={preview.landing}
    />
  );
}
