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
} from "../runtime/derivation/assignments.js";
import type { DerivedTagSource } from "../runtime/publication/catalog.js";
import type { DerivedTagManagedCategory } from "../manifest.js";
import type { OntologyExplorerEntityRecord } from "../../app/ontology/entity-record.js";
import type { DerivedTagTranslationRecord } from "../../domain/derived-tag-types.js";
import type { DerivedTagTranslationOverride } from "../translations/tag-overrides.js";
import type {
  DerivedTagReviewAssignmentModeValue,
  DerivedTagReviewDecisionKindValue,
  DerivedTagReviewExemplarActionValue,
  DerivedTagReviewRuleDecisionValue,
  DerivedTagReviewResolutionStatusValue,
  DerivedTagReviewSourceValue,
  DerivedTagReviewStatusValue,
} from "./review-vocabulary.js";
import { DERIVED_TAG_REVIEW_VOCABULARY } from "./review-vocabulary.js";

export type { DerivedTagManagedCategory } from "../manifest.js";

export const DERIVED_TAG_WORKBENCH = {
  MODES: [
    "review_queue",
    "proposal_review",
    "legacy_seed",
    "legacy_rule",
    "exemplar_cleanup",
  ] as const,
} as const;

export type DerivedTagWorkbenchMode = (typeof DERIVED_TAG_WORKBENCH.MODES)[number];

export type DerivedTagReviewSelectionSource =
  | "authored_review_queue"
  | "authored_exemplar_review_queue"
  | "llm_assignment_review_queue"
  | "llm_exemplar_review_queue"
  | "legacy_seed"
  | "legacy_rule"
  | "exemplar_cleanup";

export type DerivedTagReviewResolutionStatus = DerivedTagReviewResolutionStatusValue;

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
  kind: Extract<DerivedTagReviewDecisionKindValue, typeof DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.DECISION_KIND.ASSIGNMENT>;
  family: string;
  tag: string;
  mode: DerivedTagReviewAssignmentModeValue;
  status: DerivedTagReviewStatusValue;
  confidence?: DerivedTagReviewConfidence;
  rationale: string;
  source?: DerivedTagReviewSourceValue;
};

export type DerivedTagReviewExemplarDecision = {
  kind: Extract<DerivedTagReviewDecisionKindValue, typeof DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.DECISION_KIND.EXEMPLAR>;
  tag: string;
  polarity: DerivedTagExemplarPolarity;
  action: DerivedTagReviewExemplarActionValue;
  status: DerivedTagReviewStatusValue;
  confidence?: DerivedTagReviewConfidence;
  rationale: string;
  source?: DerivedTagReviewSourceValue;
  currentPolarity?: DerivedTagExemplarPolarity | typeof DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.EXEMPLAR_POLARITY.NONE;
};

export type DerivedTagReviewRuleDecision = {
  kind: Extract<DerivedTagReviewDecisionKindValue, typeof DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.DECISION_KIND.RULE>;
  tag: string;
  decision: DerivedTagReviewRuleDecisionValue;
  status: DerivedTagReviewStatusValue;
  rationale: string;
  source?: DerivedTagReviewSourceValue;
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
  kind: DerivedTagReviewDecisionKind;
  category: SearchCategory;
  family?: string;
  tag: string;
  count: number;
  confidence: DerivedTagReviewConfidence | "unspecified" | "mixed";
};

export type DerivedTagAuthoredState = {
  assignments: AuthoredDerivedTagAssignment[];
  assignmentReviews: Record<DerivedTagManagedCategory, DerivedTagAssignmentReviewCategory>;
  assignmentMemory: Record<DerivedTagManagedCategory, DerivedTagAssignmentMemoryCategory>;
  exemplars: Record<DerivedTagManagedCategory, DerivedTagExemplarCategory>;
  exemplarReviews: Record<DerivedTagManagedCategory, DerivedTagExemplarReviewCategory>;
  authoredRules: Record<DerivedTagManagedCategory, AuthoredDerivedTagRule[]>;
};

export type DerivedTagReviewDecisionKind = Extract<
  DerivedTagReviewDecisionKindValue,
  typeof DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.DECISION_KIND.ASSIGNMENT | typeof DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.DECISION_KIND.EXEMPLAR
>;

export type DerivedTagTranslationReviewFilterStatus = "all" | "mapped" | "provisional" | "unmapped";

export type DerivedTagTranslationReviewRow = {
  key: `${SearchCategory}:${string}`;
  base: DerivedTagTranslationRecord;
  currentOverride: DerivedTagTranslationOverride;
  draftOverride: DerivedTagTranslationOverride;
};

export type DerivedTagTranslationReviewSessionManifest = {
  id: string;
  createdAt: string;
  rowCount: number;
};

export type DerivedTagTranslationReviewSessionState = {
  currentIndex: number;
  categoryFilter: SearchCategory | "all";
  statusFilter: DerivedTagTranslationReviewFilterStatus;
  imported: boolean;
  updatedAt: string;
};

export type DerivedTagTranslationReviewSession = {
  manifest: DerivedTagTranslationReviewSessionManifest;
  rows: DerivedTagTranslationReviewRow[];
  reviewState: DerivedTagTranslationReviewSessionState;
};
