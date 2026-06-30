import { Button } from "antd";
import { ExternalLink, X } from "lucide-react";

export function RecordPreviewActions({
  onClose,
  onOpenFullPage,
}: {
  onClose: () => void;
  onOpenFullPage: () => void;
}) {
  return (
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
  );
}
