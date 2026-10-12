import { Button } from "antd";
import { ExternalLink, X } from "lucide-react";
import { shouldHandleAtlasRouteClick } from "../../app/routes";

export function RecordPreviewActions({
  onClose,
  onOpenFullPage,
  fullPageHref,
}: {
  onClose: () => void;
  onOpenFullPage: () => void;
  fullPageHref?: string;
}) {
  return (
    <div className="encounter-actions">
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
