import { Button } from "antd";
import { RecordReferenceDrawer } from "../../shared/records/RecordReferenceDrawer";
import { useRecordPreview } from "../../shared/records/useRecordPreview";
import {
  atlasRoutePath,
  navigateToAtlasRoute,
  shouldHandleAtlasRouteClick,
} from "../../app/routes";

export function EncounterOriginalRecord({ recordKey }: { recordKey: string }) {
  const preview = useRecordPreview();
  return (
    <>
      <Button
        type="link"
        size="small"
        href={atlasRoutePath({ kind: "record", recordKey })}
        onClick={(event) => {
          if (!shouldHandleAtlasRouteClick(event)) return;
          event.preventDefault();
          preview.open(recordKey, event.currentTarget);
        }}
      >
        Original record
      </Button>
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
            navigateToAtlasRoute({
              kind: "record",
              recordKey: preview.recordKey!,
              selection: preview.selection,
              landing: preview.landing,
            });
            preview.close();
          }}
        />
      )}
    </>
  );
}
