const explanations = {
  refreshed:
    "The encounter has refreshed. Close this preview and reopen the activity to use its current context.",
  participantChanged:
    "The selected participant has changed. Reopen the activity for this participant.",
  conditionRemoved: "This condition is no longer on the participant.",
  referenceUnavailable: "Reference content is unavailable for this participant.",
  generationChanged:
    "The reference collection has changed. Reload the encounter and reopen the activity.",
  participantReferenceRemoved: "This participant's reference is no longer available.",
};
/** Finite, UI-owned context changes; never constructed from transport diagnostics. */
export class RecordContextError extends Error {
  readonly explanation: string;
  constructor(reason: keyof typeof explanations) {
    super(explanations[reason]);
    this.name = "RecordContextError";
    this.explanation = explanations[reason];
  }
}
