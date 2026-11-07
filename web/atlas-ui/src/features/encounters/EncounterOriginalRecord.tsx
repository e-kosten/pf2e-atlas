import { destinationHref, isPrimaryActivation } from "../../app/navigation";
import { recordDestination } from "../../app/destinations";
import { Button } from "antd";
import { BookOpen } from "lucide-react";
import { PaneIconLink } from "../../shared/ui/actions/PaneAction";
import { RecordPreviewDrawer } from "../../shared/records/RecordPreviewDrawer";
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
          outlined
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
      <RecordPreviewDrawer preview={preview} />
    </>
  );
}
