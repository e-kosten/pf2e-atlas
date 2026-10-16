import { Button } from "antd";
import { ArrowLeft, ExternalLink, X } from "lucide-react";
import type { RecordPreviewContentProps } from "./recordPreviewTypes";
import { shouldHandleAtlasRouteClick } from "../../app/routes";

export function RecordPreviewActions({
  onClose,
  onOpenFullPage,
  fullPageHref,
  readingHistory,
}: {
  onClose: () => void;
  onOpenFullPage: () => void;
  fullPageHref?: string;
  readingHistory?: RecordPreviewContentProps["readingHistory"];
}) {
  return (
    <div className="encounter-actions">
      {readingHistory?.canGoBack && (
        <Button
          size="small"
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
        size="small"
      />
      <Button
        aria-label="Close reference preview"
        icon={<X size={14} />}
        onClick={onClose}
        size="small"
      />
    </div>
  );
}
