import { Button } from "antd";
import { ArrowLeft, ExternalLink, X } from "lucide-react";
import type { RecordPreviewContentProps } from "./recordPreviewTypes";
import { shouldHandleAtlasRouteClick } from "../../app/routes";

export function RecordPreviewActions({
  actions,
  onClose,
  onOpenFullPage,
  fullPageHref,
  readingHistory,
  showClose = true,
}: {
  actions?: React.ReactNode;
  onClose: () => void;
  onOpenFullPage: () => void;
  fullPageHref?: string;
  readingHistory?: RecordPreviewContentProps["readingHistory"];
  showClose?: boolean;
}) {
  return (
    <div className="record-preview-toolbar">
      {actions}
      <div className="encounter-actions record-preview-toolbar__navigation">
        {readingHistory?.canGoBack && (
          <Button
            size={actions ? "middle" : "small"}
            icon={<ArrowLeft size={14} />}
            onClick={readingHistory.onBack}
          >
            Back
          </Button>
        )}
        <Button
          aria-label="Open reference full page"
          icon={<ExternalLink size={14} />}
          href={fullPageHref}
          onClick={(event) => {
            if (fullPageHref && !shouldHandleAtlasRouteClick(event)) return;
            event.preventDefault();
            onOpenFullPage();
          }}
          size={actions ? "middle" : "small"}
        />
        {showClose && (
          <Button
            aria-label="Close reference preview"
            icon={<X size={14} />}
            onClick={onClose}
            size={actions ? "middle" : "small"}
          />
        )}
      </div>
    </div>
  );
}
