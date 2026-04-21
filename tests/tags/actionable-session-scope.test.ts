import { afterEach, describe, expect, it } from "vitest";

import {
  getCurrentDerivedTagAuthoredState,
  setCurrentDerivedTagAuthoredState,
} from "../../src/tags/editorial/state/authored-state.js";
import {
  getActionableSessionScopeKeys,
  matchesDerivedTagFamilyFilter,
} from "../../src/tags/editorial/sessions/actionable-session-scope.js";

const initialState = getCurrentDerivedTagAuthoredState();

afterEach(() => {
  setCurrentDerivedTagAuthoredState(initialState);
});

describe("actionable session scope keys", () => {
  it("returns only LLM proposal families and tags for proposal review", () => {
    const nextState = structuredClone(initialState);
    nextState.assignmentReviews.creature = {
      category: "creature",
      decisions: [
        {
          name: "Scout",
          recordKey: "creature:scout",
          family: "alarm",
          tag: "alarm",
          mode: "include",
          rationale: "LLM proposal.",
          source: "llm",
        },
        {
          name: "Guide",
          recordKey: "creature:guide",
          family: "alarm",
          tag: "alarm",
          mode: "include",
          rationale: "Human proposal.",
          source: "human",
        },
      ],
    };
    nextState.exemplarReviews.spell = {
      category: "spell",
      decisions: [
        {
          name: "Alarm Ward",
          recordKey: "spell:alarm-ward",
          tag: "alarm",
          proposedPolarity: "positive",
          status: "needs_review",
          rationale: "LLM exemplar proposal.",
          source: "llm",
        },
      ],
    };
    setCurrentDerivedTagAuthoredState(nextState);

    const keys = getActionableSessionScopeKeys("proposal_review", undefined);

    expect([...keys!.familyKeys]).toEqual(expect.arrayContaining(["creature:alarm", "spell:security"]));
    expect([...keys!.tagKeys]).toEqual(expect.arrayContaining(["creature:alarm", "spell:alarm"]));
  });

  it("filters exemplar cleanup keys by exemplar limit", () => {
    const nextState = structuredClone(initialState);
    nextState.exemplars.creature = {
      category: "creature",
      exemplars: [
        {
          tag: "alarm",
          positives: [
            { recordKey: "creature:one", name: "One" },
            { recordKey: "creature:two", name: "Two" },
          ],
          negatives: [{ recordKey: "creature:three", name: "Three" }],
        },
        {
          tag: "social_infiltration",
          positives: [{ recordKey: "creature:four", name: "Four" }],
          negatives: [],
        },
      ],
    };
    setCurrentDerivedTagAuthoredState(nextState);

    const keys = getActionableSessionScopeKeys("exemplar_cleanup", 2);

    expect([...keys!.tagKeys]).toContain("creature:alarm");
    expect([...keys!.tagKeys]).not.toContain("creature:social_infiltration");
  });

  it("matches tags to ontology families for family-filtered exemplar reviews", () => {
    expect(matchesDerivedTagFamilyFilter("spell", "alarm", "security")).toBe(true);
    expect(matchesDerivedTagFamilyFilter("spell", "alarm", "communication")).toBe(false);
  });
});
