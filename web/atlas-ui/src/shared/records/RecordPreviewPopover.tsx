import type React from "react";
import { RecordPresentation } from "./RecordPresentation";
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
  const position = recordPreviewPosition(anchor);

  return (
    <div
      aria-label="Reference preview overlay"
      className="record-preview-popover__backdrop"
      role="presentation"
      onClick={(event) => {
        if (event.target === event.currentTarget) {
          onClose();
        }
      }}
    >
      <section
        aria-label="Reference preview"
        className="record-preview-popover"
        role="dialog"
        style={position}
      >
        <header className="record-preview-popover__header">
          <span>Reference</span>
          <RecordPreviewActions onClose={onClose} onOpenFullPage={onOpenFullPage} />
        </header>
        <div className="record-preview-popover__body">
          <RecordPresentation
            detail={detail}
            loading={loading}
            onReference={onReference}
          />
        </div>
      </section>
    </div>
  );
}

function recordPreviewPosition(
  anchor: RecordPreviewAnchor | null,
): React.CSSProperties {
  const margin = 16;
  const gap = 8;
  const width = Math.min(560, Math.max(360, window.innerWidth - margin * 2));
  const maxHeight = Math.min(560, window.innerHeight - margin * 2);
  if (!anchor) {
    return {
      maxHeight,
      right: margin,
      top: margin,
      width,
    };
  }
  const fitsRight = anchor.right + gap + width <= window.innerWidth - margin;
  const fitsLeft = anchor.left - gap - width >= margin;
  const left = fitsRight
    ? anchor.right + gap
    : fitsLeft
      ? anchor.left - gap - width
      : clamp(anchor.left, margin, window.innerWidth - margin - width);
  return {
    left,
    maxHeight,
    top: clamp(anchor.top, margin, window.innerHeight - margin - maxHeight),
    width,
  };
}

function clamp(value: number, min: number, max: number) {
  return Math.min(Math.max(value, min), Math.max(min, max));
}
