import { useQuery } from "@tanstack/react-query";
import { Button } from "antd";
import { ExternalLink, X } from "lucide-react";
import type React from "react";
import { useCallback, useState } from "react";
import { getRecordDetail } from "../../api/atlasApi";
import { RecordPresentation } from "./RecordPresentation";

export type RecordPreviewAnchor = {
  top: number;
  right: number;
  bottom: number;
  left: number;
  width: number;
  height: number;
};

export function useRecordPreview() {
  const [recordKey, setRecordKey] = useState<string | null>(null);
  const [anchor, setAnchor] = useState<RecordPreviewAnchor | null>(null);
  const detail = useQuery({
    queryKey: ["record-preview-popover", recordKey],
    enabled: recordKey !== null,
    queryFn: () => getRecordDetail(recordKey!),
  });

  const close = useCallback(() => {
    setRecordKey(null);
    setAnchor(null);
  }, []);

  const open = useCallback((nextRecordKey: string, anchorRect?: DOMRect) => {
    setRecordKey(nextRecordKey);
    if (anchorRect) {
      setAnchor(recordPreviewAnchorFromRect(anchorRect));
    }
  }, []);

  return {
    anchor,
    close,
    detail: detail.data,
    error: detail.error,
    loading: detail.isLoading || detail.isFetching,
    open,
    recordKey,
  };
}

export function RecordPreviewPopover({
  anchor,
  detail,
  loading,
  onClose,
  onOpenFullPage,
  onReference,
}: {
  anchor: RecordPreviewAnchor | null;
  detail: Awaited<ReturnType<typeof getRecordDetail>> | undefined;
  loading: boolean;
  onClose: () => void;
  onOpenFullPage: () => void;
  onReference: (recordKey: string, anchorRect?: DOMRect) => void;
}) {
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
          <div className="encounter-actions">
            <Button
              aria-label="Open reference full page"
              icon={<ExternalLink size={14} />}
              onClick={onOpenFullPage}
              size="small"
            />
            <Button
              aria-label="Close reference preview"
              icon={<X size={14} />}
              onClick={onClose}
              size="small"
            />
          </div>
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

function recordPreviewAnchorFromRect(rect: DOMRect): RecordPreviewAnchor {
  return {
    top: rect.top,
    right: rect.right,
    bottom: rect.bottom,
    left: rect.left,
    width: rect.width,
    height: rect.height,
  };
}

function clamp(value: number, min: number, max: number) {
  return Math.min(Math.max(value, min), Math.max(min, max));
}
