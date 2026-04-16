import type { AuthoredDerivedTagRule, DerivedTagExemplarCategory, SearchCategory, SearchSubcategory } from "../../types.js";
import type {
  AuthoredDerivedTagAssignment,
  DerivedTagReviewConfidence,
  DerivedTagReviewSource,
  DerivedTagReviewStatus,
} from "../runtime/assignments.js";
import type { DerivedTagSource } from "../runtime/catalog-utils.js";

export type DerivedTagMigrationMode =
  | "review_queue"
  | "legacy_seed"
  | "legacy_rule"
  | "exemplar_cleanup"
  | "new_tagging";

export type DerivedTagManagedCategory =
  | "affliction"
  | "creature"
  | "equipment"
  | "hazard"
  | "spell";

export type DerivedTagMigrationSelectionSource =
  | "authored_review_queue"
  | "legacy_seed"
  | "legacy_rule"
  | "exemplar_cleanup"
  | "untagged";

export type DerivedTagMigrationResolutionStatus = "complete" | "needs_review";

export type DerivedTagMigrationSelectionReason = {
  source: DerivedTagMigrationSelectionSource;
  family?: string;
  tag?: string;
  note: string;
};

export type DerivedTagMigrationSessionRecord = {
  recordKey: string;
  name: string;
  category: SearchCategory;
  subcategory: SearchSubcategory | null;
  packName: string;
  level: number | null;
  traits: string[];
  families: string[];
  currentDerivedTags: string[];
  currentSources: Record<string, DerivedTagSource>;
  descriptionText: string | null;
  blurbText: string | null;
  selectionReasons: DerivedTagMigrationSelectionReason[];
};

export type DerivedTagMigrationAssignmentDecision = {
  kind: "assignment";
  family: string;
  tag: string;
  mode: "include" | "exclude";
  status: DerivedTagReviewStatus;
  confidence?: DerivedTagReviewConfidence;
  rationale: string;
  source?: DerivedTagReviewSource;
};

export type DerivedTagMigrationExemplarDecision = {
  kind: "exemplar";
  tag: string;
  polarity: "positive" | "negative";
  action: "keep" | "drop";
  status: DerivedTagReviewStatus;
  rationale: string;
  source?: DerivedTagReviewSource;
};

export type DerivedTagMigrationRuleDecision = {
  kind: "rule";
  tag: string;
  decision: "recreate_authored" | "assignment_takeover" | "retain_legacy";
  status: DerivedTagReviewStatus;
  rationale: string;
  source?: DerivedTagReviewSource;
  authoredRules?: AuthoredDerivedTagRule[];
};

export type DerivedTagMigrationDecision =
  | DerivedTagMigrationAssignmentDecision
  | DerivedTagMigrationExemplarDecision
  | DerivedTagMigrationRuleDecision;

export type DerivedTagMigrationRecordDecision = {
  recordKey: string;
  name: string;
  category: SearchCategory;
  resolutionStatus: DerivedTagMigrationResolutionStatus;
  ontologyNotes?: string[];
  decisions: DerivedTagMigrationDecision[];
};

export type DerivedTagMigrationSessionManifest = {
  id: string;
  mode: DerivedTagMigrationMode;
  category?: SearchCategory;
  subcategory?: SearchSubcategory;
  family?: string;
  tag?: string;
  createdAt: string;
  recordCount: number;
};

export type DerivedTagMigrationSessionReviewState = {
  currentIndex: number;
  unresolvedOnly: boolean;
  updatedAt: string;
};

export type DerivedTagMigrationSession = {
  manifest: DerivedTagMigrationSessionManifest;
  records: DerivedTagMigrationSessionRecord[];
  decisions: DerivedTagMigrationRecordDecision[];
  reviewState: DerivedTagMigrationSessionReviewState;
};

export type DerivedTagMigrationSessionCreateOptions = {
  mode: DerivedTagMigrationMode;
  category?: SearchCategory;
  subcategory?: SearchSubcategory;
  family?: string;
  tag?: string;
  limit?: number;
  exemplarLimit?: number;
};

export type DerivedTagReviewQueueSummaryItem = {
  category: SearchCategory;
  family: string;
  tag: string;
  count: number;
  confidence: DerivedTagReviewConfidence | "unspecified";
};

export type DerivedTagMigrationAuthoredState = {
  assignments: Record<DerivedTagManagedCategory, AuthoredDerivedTagAssignment[]>;
  exemplars: Record<DerivedTagManagedCategory, DerivedTagExemplarCategory>;
  authoredRules: Record<DerivedTagManagedCategory, AuthoredDerivedTagRule[]>;
};
