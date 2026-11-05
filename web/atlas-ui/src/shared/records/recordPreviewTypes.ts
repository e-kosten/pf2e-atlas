import type { RecordDetailView } from "../../generated/atlas";
import type { RecordReferenceHandler } from "./PreparedContent";

export type RecordPreviewAnchor = HTMLElement;

export type RecordPreviewContentProps = {
  trait?: {
    key: string;
    detail: import("../../generated/atlas").TraitDetailView | undefined;
    loading: boolean;
    error: Error | null;
    retry: () => void;
  };
  onTraitReference?: (traitKey: string, anchor?: HTMLElement) => void;
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
  requestedNavigation?: import("../../generated/atlas").RecordNavigationView;
  landing?: import("../../generated/atlas").RecordNavigationView;
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
