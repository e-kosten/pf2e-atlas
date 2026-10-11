import type { RecordDetailView } from "../../generated/atlas";
import type { RecordReferenceHandler } from "./PreparedContent";

export type RecordPreviewAnchor = HTMLElement;

export type RecordPreviewContentProps = {
  detail: RecordDetailView | undefined;
  loading: boolean;
  onClose: () => void;
  onOpenFullPage: () => void;
  onReference: RecordReferenceHandler;
};
