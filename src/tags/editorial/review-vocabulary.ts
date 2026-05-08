export const DERIVED_TAG_REVIEW_VOCABULARY = {
  REVIEW: {
    STATUS: {
      AUTO_APPLIED: "auto_applied" as const,
      NEEDS_REVIEW: "needs_review" as const,
      APPROVED: "approved" as const,
      REJECTED: "rejected" as const,
    },
    RESOLUTION_STATUS: {
      COMPLETE: "complete" as const,
      NEEDS_REVIEW: "needs_review" as const,
    },
    DECISION_KIND: {
      ASSIGNMENT: "assignment" as const,
      EXEMPLAR: "exemplar" as const,
      RULE: "rule" as const,
    },
    ASSIGNMENT_MODE: {
      INCLUDE: "include" as const,
      EXCLUDE: "exclude" as const,
    },
    EXEMPLAR_ACTION: {
      KEEP: "keep" as const,
      DROP: "drop" as const,
    },
    EXEMPLAR_POLARITY: {
      POSITIVE: "positive" as const,
      NEGATIVE: "negative" as const,
      NONE: "none" as const,
    },
    RULE_DECISION: {
      RECREATE_AUTHORED: "recreate_authored" as const,
      ASSIGNMENT_TAKEOVER: "assignment_takeover" as const,
      RETAIN_LEGACY: "retain_legacy" as const,
    },
    CONFIDENCE: {
      HIGH: "high" as const,
      MEDIUM: "medium" as const,
      LOW: "low" as const,
    },
    SOURCE: {
      HUMAN: "human" as const,
      LLM: "llm" as const,
    },
    UI_ACTION: {
      APPROVE: "approve" as const,
      REJECT: "reject" as const,
      NEEDS_REVIEW: "needs_review" as const,
      TOGGLE_UNRESOLVED: "toggle_unresolved" as const,
      IMPORT: "import" as const,
      QUIT: "quit" as const,
    },
    STATUSES: ["auto_applied", "needs_review", "approved", "rejected"] as const,
    RESOLUTION_STATUSES: ["complete", "needs_review"] as const,
    DECISION_KINDS: ["assignment", "exemplar", "rule"] as const,
    ASSIGNMENT_MODES: ["include", "exclude"] as const,
    EXEMPLAR_ACTIONS: ["keep", "drop"] as const,
    EXEMPLAR_POLARITIES: ["positive", "negative", "none"] as const,
    RULE_DECISIONS: ["recreate_authored", "assignment_takeover", "retain_legacy"] as const,
    CONFIDENCES: ["high", "medium", "low"] as const,
    SOURCES: ["human", "llm"] as const,
  },
} as const;

export type DerivedTagReviewStatusValue = (typeof DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.STATUSES)[number];
export type DerivedTagReviewResolutionStatusValue = (typeof DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.RESOLUTION_STATUSES)[number];
export type DerivedTagReviewDecisionKindValue = (typeof DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.DECISION_KINDS)[number];
export type DerivedTagReviewAssignmentModeValue = (typeof DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.ASSIGNMENT_MODES)[number];
export type DerivedTagReviewExemplarActionValue = (typeof DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.EXEMPLAR_ACTIONS)[number];
export type DerivedTagReviewExemplarPolarityValue = (typeof DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.EXEMPLAR_POLARITIES)[number];
export type DerivedTagReviewRuleDecisionValue = (typeof DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.RULE_DECISIONS)[number];
export type DerivedTagReviewConfidenceValue = (typeof DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.CONFIDENCES)[number];
export type DerivedTagReviewSourceValue = (typeof DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.SOURCES)[number];
