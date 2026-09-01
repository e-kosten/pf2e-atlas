import { useState } from "react";
import type { PreviewPopoverTrigger } from "../ui/overlays/PreviewPopover";
import { PreviewPopover, usePreviewPopoverClose } from "../ui/overlays/PreviewPopover";
import { RecordDetailPane } from "./RecordDetailPane";
import { RecordPreviewActions } from "./RecordPreviewActions";
import { RecordPreviewContext } from "./RecordPreviewContext";
import { useRecordDetail } from "./useRecordDetail";

export function RecordPreviewPopover({
  children,
  onOpenFullPage,
  recordKey,
}: {
  children: PreviewPopoverTrigger;
  onOpenFullPage: (recordKey: string) => void;
  recordKey: string;
}) {
  const [open, setOpen] = useState(false);
  const [activeRecordKey, setActiveRecordKey] = useState(recordKey);
  const detail = useRecordDetail(open ? activeRecordKey : null);

  return (
    <PreviewPopover
      actions={
        <RecordPopoverActions onOpenFullPage={() => onOpenFullPage(activeRecordKey)} />
      }
      ariaLabel="Reference preview"
      content={
        <RecordPreviewContext.Provider value={null}>
          <RecordDetailPane
            detail={detail.data}
            errors={[detail.error]}
            loading={detail.isLoading || detail.isFetching}
            onReference={setActiveRecordKey}
          />
        </RecordPreviewContext.Provider>
      }
      onOpenChange={(nextOpen) => {
        setOpen(nextOpen);
        if (nextOpen) {
          setActiveRecordKey(recordKey);
        }
      }}
      title="Reference"
    >
      {children}
    </PreviewPopover>
  );
}

function RecordPopoverActions({ onOpenFullPage }: { onOpenFullPage: () => void }) {
  const close = usePreviewPopoverClose();
  return (
    <RecordPreviewActions
      onClose={close}
      onOpenFullPage={() => {
        close();
        onOpenFullPage();
      }}
    />
  );
}
