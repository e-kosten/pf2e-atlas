import type {
  AuthoredDerivedTagRule,
  DerivedTagExemplarCategory,
  DerivedTagExemplarPolarity,
  DerivedTagExemplarReviewCategory,
  SearchCategory,
  SearchSubcategory,
} from "../../domain/derived-tag-types.js";
import type {
  AuthoredDerivedTagAssignment,
  DerivedTagAssignmentMemoryCategory,
  DerivedTagAssignmentReviewCategory,
  DerivedTagReviewConfidence,
  DerivedTagReviewSource,
  DerivedTagReviewStatus,
} from "../runtime/derivation/assignments.js";
import type { DerivedTagSource } from "../runtime/publication/catalog.js";
import type { DerivedTagManagedCategory } from "../manifest.js";
import type { OntologyExplorerEntityRecord } from "../../app/ontology/entity-record.js";

export type { DerivedTagManagedCategory } from "../manifest.js";

export type DerivedTagWorkbenchMode =
  | "review_queue"
  | "proposal_review"
  | "legacy_seed"
  | "legacy_rule"
  | "exemplar_cleanup";

export type DerivedTagReviewSelectionSource =
  | "authored_review_queue"
  | "authored_exemplar_review_queue"
  | "llm_assignment_review_queue"
  | "llm_exemplar_review_queue"
  | "legacy_seed"
  | "legacy_rule"
  | "exemplar_cleanup";

export type DerivedTagReviewResolutionStatus = "complete" | "needs_review";

export type DerivedTagReviewSelectionReason = {
  source: DerivedTagReviewSelectionSource;
  family?: string;
  tag?: string;
  note: string;
};

export type DerivedTagReviewSessionRecord = {
  entityRecord: OntologyExplorerEntityRecord;
  currentSources: Record<string, DerivedTagSource>;
  selectionReasons: DerivedTagReviewSelectionReason[];
};

export type DerivedTagReviewAssignmentDecision = {
  kind: "assignment";
  family: string;
  tag: string;
  mode: "include" | "exclude";
  status: DerivedTagReviewStatus;
  confidence?: DerivedTagReviewConfidence;
  rationale: string;
  source?: DerivedTagReviewSource;
};

export type DerivedTagReviewExemplarDecision = {
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

export type DerivedTagReviewRuleDecision = {
  kind: "rule";
  tag: string;
  decision: "recreate_authored" | "assignment_takeover" | "retain_legacy";
  status: DerivedTagReviewStatus;
  rationale: string;
  source?: DerivedTagReviewSource;
  authoredRules?: AuthoredDerivedTagRule[];
};

export type DerivedTagReviewDecision =
  | DerivedTagReviewAssignmentDecision
  | DerivedTagReviewExemplarDecision
  | DerivedTagReviewRuleDecision;

export type DerivedTagReviewRecordDecision = {
  recordKey: string;
  name: string;
  category: SearchCategory;
  resolutionStatus: DerivedTagReviewResolutionStatus;
  ontologyNotes?: string[];
  decisions: DerivedTagReviewDecision[];
};

export type DerivedTagReviewSessionManifest = {
  id: string;
  mode: DerivedTagWorkbenchMode;
  category?: SearchCategory;
  subcategory?: SearchSubcategory;
  family?: string;
  tag?: string;
  createdAt: string;
  recordCount: number;
};

export type DerivedTagReviewSessionState = {
  currentIndex: number;
  unresolvedOnly: boolean;
  updatedAt: string;
};

export type DerivedTagReviewSession = {
  manifest: DerivedTagReviewSessionManifest;
  records: DerivedTagReviewSessionRecord[];
  decisions: DerivedTagReviewRecordDecision[];
  reviewState: DerivedTagReviewSessionState;
};

export type DerivedTagReviewSessionCreateOptions = {
  mode: DerivedTagWorkbenchMode;
  category?: SearchCategory;
  subcategory?: SearchSubcategory;
  decisionKind?: DerivedTagReviewDecisionKind;
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

export type DerivedTagAuthoredState = {
  assignments: Record<DerivedTagManagedCategory, AuthoredDerivedTagAssignment[]>;
  assignmentReviews: Record<DerivedTagManagedCategory, DerivedTagAssignmentReviewCategory>;
  assignmentMemory: Record<DerivedTagManagedCategory, DerivedTagAssignmentMemoryCategory>;
  exemplars: Record<DerivedTagManagedCategory, DerivedTagExemplarCategory>;
  exemplarReviews: Record<DerivedTagManagedCategory, DerivedTagExemplarReviewCategory>;
  authoredRules: Record<DerivedTagManagedCategory, AuthoredDerivedTagRule[]>;
};

export type DerivedTagReviewDecisionKind = "assignment" | "exemplar";
