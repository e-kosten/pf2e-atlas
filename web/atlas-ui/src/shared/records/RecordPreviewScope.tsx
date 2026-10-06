import { useCallback } from "react";
import type React from "react";
import type { PreviewPopoverTrigger } from "../ui/overlays/PreviewPopover";
import { RecordPreviewContext } from "./RecordPreviewContext";
import { RecordPreviewPopover } from "./RecordPreviewPopover";

export function RecordPreviewScope({
  children,
  onOpenFullPage,
}: {
  children: React.ReactNode;
  onOpenFullPage: (recordKey: string) => void;
}) {
  const renderPreview = useCallback(
    (recordKey: string, trigger: PreviewPopoverTrigger) => (
      <RecordPreviewPopover
        key={recordKey}
        onOpenFullPage={onOpenFullPage}
        recordKey={recordKey}
      >
        {trigger}
      </RecordPreviewPopover>
    ),
    [onOpenFullPage],
  );

  return (
    <RecordPreviewContext.Provider value={renderPreview}>
      {children}
    </RecordPreviewContext.Provider>
  );
}
