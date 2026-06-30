import type { RecordDetailView } from "../../generated/atlas";

export type RecordPreviewAnchor = {
  top: number;
  right: number;
  bottom: number;
  left: number;
  width: number;
  height: number;
};

export type RecordPreviewContentProps = {
  detail: RecordDetailView | undefined;
  loading: boolean;
  onClose: () => void;
  onOpenFullPage: () => void;
  onReference: (recordKey: string, anchorRect?: DOMRect) => void;
};
