import { Button } from "antd";
import { ExternalLink, X } from "lucide-react";
import type React from "react";

export function RecordPreviewActions({
  children,
  closeLabel = "Close reference preview",
  onClose,
  onOpenFullPage,
  openLabel = "Open reference full page",
}: {
  children?: React.ReactNode;
  closeLabel?: string;
  onClose: () => void;
  onOpenFullPage: () => void;
  openLabel?: string;
}) {
  return (
    <div className="preview-popover__actions">
      {children}
      <Button
        aria-label={openLabel}
        icon={<ExternalLink size={14} />}
        onClick={onOpenFullPage}
        size="small"
      />
      <Button
        aria-label={closeLabel}
        icon={<X size={14} />}
        onClick={onClose}
        size="small"
      />
    </div>
  );
}
