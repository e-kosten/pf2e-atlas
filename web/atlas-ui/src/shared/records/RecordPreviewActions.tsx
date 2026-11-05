import { isPrimaryActivation } from "../../app/navigation";
import { Button, Tooltip } from "antd";
import { ArrowLeft, ExternalLink, X } from "lucide-react";
import type { RecordPreviewContentProps } from "./recordPreviewTypes";
export function RecordPreviewActions({
  actions,
  secondaryActions,
  contextActions,
  onClose,
  onOpenFullPage,
  fullPageHref,
  readingHistory,
}: {
  actions?: React.ReactNode;
  secondaryActions?: React.ReactNode;
  contextActions?: React.ReactNode;
  onClose: () => void;
  onOpenFullPage: () => void;
  fullPageHref?: string;
  readingHistory?: RecordPreviewContentProps["readingHistory"];
}) {
  return (
    <div className="record-preview-toolbar">
      {contextActions}
      <div className="record-preview-toolbar__navigation">
        {readingHistory?.canGoBack && (
          <Button
            size="middle"
            icon={<ArrowLeft size={14} />}
            onClick={readingHistory.onBack}
          >
            Back
          </Button>
        )}
        {actions}
        {secondaryActions}
        <Tooltip title="Open full page" trigger={["hover", "focus"]}>
          <Button
            aria-label="Open reference full page"
            icon={<ExternalLink size={16} aria-hidden="true" />}
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
            icon={<X size={16} aria-hidden="true" />}
            onClick={onClose}
            size="middle"
          />
        </Tooltip>
      </div>
    </div>
  );
}
