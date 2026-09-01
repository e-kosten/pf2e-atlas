import { createContext, useCallback, useContext, useEffect, useState } from "react";
import type React from "react";
import { RecordDetailPane } from "./RecordDetailPane";
import { RecordPreviewActions } from "./RecordPreviewActions";
import type {
  RecordPreviewAnchor,
  RecordPreviewContentProps,
} from "./recordPreviewTypes";

type RecordPreviewPopoverProps = RecordPreviewContentProps & {
  anchor: RecordPreviewAnchor | null;
  children?: React.ReactNode;
  label?: string;
  title?: React.ReactNode;
  triggerElement?: HTMLElement | null;
};

type ImmediateRecordPreview = {
  anchor: DOMRect;
  content: React.ReactNode;
  label: string;
  recordKey: string;
  title: React.ReactNode;
  triggerElement: HTMLElement;
};

type RecordPreviewHost = {
  open: (preview: ImmediateRecordPreview) => void;
};

const RecordPreviewHostContext = createContext<RecordPreviewHost | null>(null);

export function useRecordPreviewHost() {
  return useContext(RecordPreviewHostContext);
}

export function RecordPreviewPopover({
  anchor,
  children,
  detail,
  label = "Reference preview",
  loading,
  onClose,
  onOpenFullPage,
  onReference,
  title = "Reference",
  triggerElement,
}: RecordPreviewPopoverProps) {
  const [immediate, setImmediate] = useState<ImmediateRecordPreview | null>(null);
  const position = recordPreviewPosition(immediate?.anchor ?? anchor);
  const activeTrigger = immediate?.triggerElement ?? triggerElement;
  const closeAndRestoreFocus = useCallback(() => {
    activeTrigger?.focus();
    onClose();
  }, [activeTrigger, onClose]);
  const openImmediate = useCallback(
    (preview: ImmediateRecordPreview) => {
      setImmediate(preview);
      onReference(preview.recordKey, preview.anchor, preview.triggerElement);
    },
    [onReference],
  );
  useEffect(() => {
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        closeAndRestoreFocus();
      }
    };
    document.addEventListener("keydown", closeOnEscape);
    return () => document.removeEventListener("keydown", closeOnEscape);
  }, [closeAndRestoreFocus]);

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
        aria-label={immediate?.label ?? label}
        className="record-preview-popover"
        role="dialog"
        style={position}
      >
        <header className="record-preview-popover__header">
          <span>{immediate?.title ?? title}</span>
          <RecordPreviewActions
            onClose={closeAndRestoreFocus}
            onOpenFullPage={onOpenFullPage}
          />
        </header>
        <div className="record-preview-popover__body">
          <RecordPreviewHostContext.Provider value={{ open: openImmediate }}>
            {immediate?.content ?? children ?? (
              <RecordDetailPane
                detail={detail}
                errors={[]}
                loading={loading}
                onReference={onReference}
              />
            )}
          </RecordPreviewHostContext.Provider>
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
