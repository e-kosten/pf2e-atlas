import {
  destinationHref,
  isPrimaryActivation,
  navigateDestination,
} from "../../app/navigation";
import { recordDestination } from "../../app/destinations";
import { Tooltip } from "antd";
import { BookOpen } from "lucide-react";
import { PaneIconLink } from "../../shared/ui/actions/PaneAction";
import { RecordReferenceDrawer } from "../../shared/records/RecordReferenceDrawer";
import { useRecordPreview } from "../../shared/records/useRecordPreview";
export function EncounterOriginalRecord({
  recordKey,
  label,
}: {
  recordKey: string;
  label: string;
}) {
  const preview = useRecordPreview();
  return (
    <>
      <Tooltip title={label} trigger={["hover", "focus"]}>
        <PaneIconLink
          title=""
          label={label}
          icon={<BookOpen size={16} aria-hidden="true" />}
          href={destinationHref(
            recordDestination({
              recordKey,
            }),
          )}
          onClick={(event) => {
            if (!isPrimaryActivation(event)) return;
            event.preventDefault();
            preview.open(recordKey, event.currentTarget);
          }}
        />
      </Tooltip>
      {preview.recordKey && (
        <RecordReferenceDrawer
          anchor={preview.anchor}
          detail={preview.detail}
          loading={preview.loading}
          error={preview.error}
          onRetry={preview.retry}
          onClose={preview.close}
          onReference={preview.open}
          onRankChange={preview.changeRank}
          readingHistory={preview.readingHistory}
          landing={preview.landing}
          onOpenFullPage={() => {
            navigateDestination(
              recordDestination({
                recordKey: preview.recordKey!,
                selection: preview.selection,
                landing: preview.landing,
              }),
            );
            preview.close();
          }}
        />
      )}
    </>
  );
}
