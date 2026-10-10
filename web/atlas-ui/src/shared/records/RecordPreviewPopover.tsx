import { Popover } from "antd";
import { useEffect } from "react";
import { RecordDetailPane } from "./RecordDetailPane";
import { RecordPreviewActions } from "./RecordPreviewActions";
import type {
  RecordPreviewAnchor,
  RecordPreviewContentProps,
} from "./recordPreviewTypes";
type RecordPreviewPopoverProps = RecordPreviewContentProps & {
  anchor: RecordPreviewAnchor | null;
};
export function RecordPreviewPopover({
  anchor,
  detail,
  loading,
  onClose,
  onOpenFullPage,
  onReference,
}: RecordPreviewPopoverProps) {
  useEffect(() => {
    const close = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    document.addEventListener("keydown", close);
    return () => document.removeEventListener("keydown", close);
  }, [onClose]);
  return (
    <Popover
      open
      placement="rightTop"
      trigger="click"
      onOpenChange={(open) => {
        if (!open) onClose();
      }}
      title={
        <div className="record-preview-popover__header">
          <span>Reference</span>
          <RecordPreviewActions onClose={onClose} onOpenFullPage={onOpenFullPage} />
        </div>
      }
      content={
        <section
          aria-label="Reference preview"
          className="record-preview-popover__body"
          role="dialog"
        >
          <RecordDetailPane
            detail={detail}
            loading={loading}
            onReference={onReference}
          />
        </section>
      }
    >
      <span
        aria-hidden
        className="record-preview-anchor"
        style={{
          position: "fixed",
          left: anchor?.left || 0,
          top: anchor?.top || 0,
          width: anchor?.width || 1,
          height: anchor?.height || 1,
          pointerEvents: "none",
        }}
      />
    </Popover>
  );
}
