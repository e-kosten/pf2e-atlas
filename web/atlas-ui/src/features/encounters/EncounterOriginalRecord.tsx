import { Button, Modal } from "antd";
import { useState } from "react";
import { RecordDetailPane } from "../../shared/records/RecordDetailPane";
import { RecordPreviewPopover } from "../../shared/records/RecordPreviewPopover";
import { useRecordDetail } from "../../shared/records/useRecordDetail";
import { useRecordPreview } from "../../shared/records/useRecordPreview";
import { navigateToAtlasRoute } from "../../app/routes";

export function EncounterOriginalRecord({ recordKey }: { recordKey: string }) {
  const [open, setOpen] = useState(false);
  const detail = useRecordDetail(open ? recordKey : null);
  const preview = useRecordPreview();
  const close = () => {
    preview.close();
    setOpen(false);
  };
  return (
    <>
      <Button type="link" size="small" onClick={() => setOpen(true)}>
        Original record
      </Button>
      <Modal open={open} onCancel={close} footer={null} title="Source record">
        <p className="muted">Original reference; encounter changes are not applied.</p>
        <RecordDetailPane
          detail={detail.data}
          loading={detail.isLoading || detail.isFetching}
          error={detail.error}
          onRetry={() => {
            void detail.refetch();
          }}
          onBack={close}
          backLabel="Close source content"
          onReference={preview.open}
        />
        {preview.recordKey && (
          <RecordPreviewPopover
            context="encounter"
            visibleOwnerRecordKey={recordKey}
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
              if (preview.recordKey)
                navigateToAtlasRoute({
                  kind: "record",
                  recordKey: preview.recordKey,
                  selection: preview.selection,
                  landing: preview.landing,
                });
            }}
          />
        )}
      </Modal>
    </>
  );
}
