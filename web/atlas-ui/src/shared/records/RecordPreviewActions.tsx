import { isPrimaryActivation } from "../../app/navigation";
import { Button, Tooltip } from "antd";
import { ArrowLeft, ExternalLink, X } from "lucide-react";
import type { RecordPreviewContentProps } from "./recordPreviewTypes";
export function RecordPreviewActions({
  actions,
  secondaryActions,
  onClose,
  onOpenFullPage,
  fullPageHref,
  readingHistory,
}: {
  actions?: React.ReactNode;
  secondaryActions?: React.ReactNode;
  onClose: () => void;
  onOpenFullPage: () => void;
  fullPageHref?: string;
  readingHistory?: RecordPreviewContentProps["readingHistory"];
}) {
  return (
    <div className="record-preview-toolbar">
      {actions}
      <div className="encounter-actions record-preview-toolbar__navigation">
        {readingHistory?.canGoBack && (
          <Button
            size="middle"
            icon={<ArrowLeft size={14} />}
            onClick={readingHistory.onBack}
          >
            Back
          </Button>
        )}
        {secondaryActions}
        <Tooltip title="Open full page" trigger={["hover", "focus"]}>
          <Button
            aria-label="Open reference full page"
            icon={<ExternalLink size={14} />}
            href={fullPageHref}
            onClick={(event) => {
              if (fullPageHref && !isPrimaryActivation(event)) return;
              event.preventDefault();
              onOpenFullPage();
            }}
            size="middle"
          />
        </Tooltip>
        <Tooltip title="Close reference" trigger={["hover", "focus"]}>
          <Button
            aria-label="Close reference preview"
            icon={<X size={14} />}
            onClick={onClose}
            size="middle"
          />
        </Tooltip>
      </div>
    </div>
  );
}
