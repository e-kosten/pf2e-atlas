import type {
  AuthoredDerivedTagRule,
  DerivedTagExemplarCategory,
  DerivedTagExemplarPolarity,
  DerivedTagExemplarReviewCategory,
  SearchCategory,
  SearchSubcategory,
} from "../../types.js";
import type {
  AuthoredDerivedTagAssignment,
  DerivedTagAssignmentMemoryCategory,
  DerivedTagAssignmentReviewCategory,
  DerivedTagReviewConfidence,
  DerivedTagReviewSource,
  DerivedTagReviewStatus,
} from "../runtime/assignments.js";
import type { DerivedTagSource } from "../runtime/catalog-utils.js";
import type { OntologyExplorerEntityRecord } from "../../tui/ontology-explorer/entity-record.js";

export type DerivedTagMigrationMode =
  | "review_queue"
  | "proposal_review"
  | "legacy_seed"
  | "legacy_rule"
  | "exemplar_cleanup";

export type DerivedTagManagedCategory =
  | "affliction"
  | "creature"
  | "equipment"
  | "hazard"
  | "spell";

export type DerivedTagMigrationSelectionSource =
  | "authored_review_queue"
  | "authored_exemplar_review_queue"
  | "llm_assignment_review_queue"
  | "llm_exemplar_review_queue"
  | "legacy_seed"
  | "legacy_rule"
  | "exemplar_cleanup";

export type DerivedTagMigrationResolutionStatus = "complete" | "needs_review";

export type DerivedTagMigrationSelectionReason = {
  source: DerivedTagMigrationSelectionSource;
  family?: string;
  tag?: string;
  note: string;
};

export type DerivedTagMigrationSessionRecord = {
  entityRecord: OntologyExplorerEntityRecord;
  currentSources: Record<string, DerivedTagSource>;
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
  polarity: DerivedTagExemplarPolarity;
  action: "keep" | "drop";
  status: DerivedTagReviewStatus;
  confidence?: DerivedTagReviewConfidence;
  rationale: string;
  source?: DerivedTagReviewSource;
  currentPolarity?: DerivedTagExemplarPolarity | "none";
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
  decisionKind?: DerivedTagMigrationReviewDecisionKind;
  family?: string;
  tag?: string;
  limit?: number;
  exemplarLimit?: number;
};

export type DerivedTagReviewQueueSummaryItem = {
  kind: "assignment" | "exemplar";
  category: SearchCategory;
  family?: string;
  tag: string;
  count: number;
  confidence: DerivedTagReviewConfidence | "unspecified" | "mixed";
};

export type DerivedTagMigrationAuthoredState = {
  assignments: Record<DerivedTagManagedCategory, AuthoredDerivedTagAssignment[]>;
  assignmentReviews: Record<DerivedTagManagedCategory, DerivedTagAssignmentReviewCategory>;
  assignmentMemory: Record<DerivedTagManagedCategory, DerivedTagAssignmentMemoryCategory>;
  exemplars: Record<DerivedTagManagedCategory, DerivedTagExemplarCategory>;
  exemplarReviews: Record<DerivedTagManagedCategory, DerivedTagExemplarReviewCategory>;
  authoredRules: Record<DerivedTagManagedCategory, AuthoredDerivedTagRule[]>;
};

export type DerivedTagMigrationReviewDecisionKind = "assignment" | "exemplar";
