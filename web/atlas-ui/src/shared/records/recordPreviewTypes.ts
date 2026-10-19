import type { RecordDetailView } from "../../generated/atlas";
import type { RecordReferenceHandler } from "./PreparedContent";

export type RecordPreviewAnchor = HTMLElement;

export type RecordPreviewContentProps = {
  actions?: import("react").ReactNode;
  context?: "reference" | "encounter";
  visibleOwnerRecordKey?: string;
  readingHistory?: {
    address: string;
    restoringHistory: boolean;
    canGoBack: boolean;
    onBack: () => void;
    restore: { scroll: number; focusHref?: string };
  };
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
  onReload?: () => void;
  reloadLabel?: string;
  retryLabel?: string;
  onClose: () => void;
  onOpenFullPage: () => void;
  onReference: RecordReferenceHandler;
  onRankChange?: (rank: number | undefined) => void;
};
