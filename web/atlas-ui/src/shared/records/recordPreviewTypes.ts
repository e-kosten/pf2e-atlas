import type { RecordDetailView } from "../../generated/atlas";
import type { RecordReferenceHandler } from "./PreparedContent";

export type RecordPreviewAnchor = HTMLElement;

export type RecordPreviewContentProps = {
  content?: import("react").ReactNode;
  fullPageHref?: string;
  landing?: import("../../generated/atlas").RecordNavigationView;
  onOwnerReturn?: (
    parent: string,
    landing: import("../../generated/atlas").RecordNavigationView,
    anchor: HTMLElement,
  ) => void;
  detail: RecordDetailView | undefined;
  loading: boolean;
  error?: Error | null;
  onRetry?: () => void;
  onClose: () => void;
  onOpenFullPage: () => void;
  onReference: RecordReferenceHandler;
};
