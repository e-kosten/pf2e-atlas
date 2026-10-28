import {
  destinationHref,
  isPrimaryActivation,
  navigateDestination,
} from "../../app/navigation";
import { recordDestination } from "../../app/destinations";
import { Button, Tooltip } from "antd";
import { BookOpen } from "lucide-react";
import { PaneIconLink } from "../../shared/ui/actions/PaneAction";
import { RecordReferenceDrawer } from "../../shared/records/RecordReferenceDrawer";
import { useRecordPreview } from "../../shared/records/useRecordPreview";
export function EncounterOriginalRecord({
  recordKey,
  label,
  navigation,
  text,
}: {
  recordKey: string;
  label: string;
  navigation?: import("../../generated/atlas").RecordNavigationView;
  text?: string;
}) {
  const preview = useRecordPreview();
  return (
    <>
      <Tooltip title={text ? null : label} trigger={["hover", "focus"]}>
        {text ? (
          <Button
            type="link"
            size="small"
            icon={<BookOpen size={16} aria-hidden="true" />}
            href={destinationHref(
              recordDestination({ recordKey, selection: navigation }),
            )}
            onClick={(event) => {
              if (!isPrimaryActivation(event)) return;
              event.preventDefault();
              preview.open(recordKey, event.currentTarget, navigation);
            }}
          >
            {text}
          </Button>
        ) : (
          <PaneIconLink
            title=""
            label={label}
            icon={<BookOpen size={16} aria-hidden="true" />}
            href={destinationHref(
              recordDestination({ recordKey, selection: navigation }),
            )}
            onClick={(event) => {
              if (!isPrimaryActivation(event)) return;
              event.preventDefault();
              preview.open(recordKey, event.currentTarget, navigation);
            }}
          />
        )}
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
          requestedNavigation={preview.selection}
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
