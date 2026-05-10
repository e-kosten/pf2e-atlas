import type { DerivedTagRuntimeArtifacts } from "../../runtime/derivation/runtime-builder.js";
import {
  buildDerivedTagRuntimeArtifactsFromAuthoredState,
  deriveRecordTagDerivationFromRuntime,
  type DerivedTagDerivation,
} from "../../runtime/derivation/api.js";
import type { DerivedTagContext } from "../../runtime/matcher/engine.js";
import {
  getCurrentDerivedTagFamilyTranslationDefaultsRevision,
  getCurrentDerivedTagTranslationOverridesRevision,
} from "../../translations/state.js";
import {
  getCurrentDerivedTagAuthoredState,
  getCurrentDerivedTagAuthoredStateRevision,
} from "./authored-state.js";

type DerivedTagWorkingRuntime = DerivedTagRuntimeArtifacts;

let workingRuntimeCache: {
  authoredRevision: number;
  familyDefaultsRevision: number;
  translationRevision: number;
  runtime: DerivedTagWorkingRuntime;
} | null = null;

function buildCurrentDerivedTagWorkingRuntime(): DerivedTagWorkingRuntime {
  const state = getCurrentDerivedTagAuthoredState();
  const runtime = buildDerivedTagRuntimeArtifactsFromAuthoredState({
    explicitAssignments: state.assignments,
  });

  return runtime;
}

export function getCurrentDerivedTagWorkingRuntime(): DerivedTagWorkingRuntime {
  const authoredRevision = getCurrentDerivedTagAuthoredStateRevision();
  const familyDefaultsRevision = getCurrentDerivedTagFamilyTranslationDefaultsRevision();
  const translationRevision = getCurrentDerivedTagTranslationOverridesRevision();
  if (
    !workingRuntimeCache ||
    workingRuntimeCache.authoredRevision !== authoredRevision ||
    workingRuntimeCache.familyDefaultsRevision !== familyDefaultsRevision ||
    workingRuntimeCache.translationRevision !== translationRevision
  ) {
    workingRuntimeCache = {
      authoredRevision,
      familyDefaultsRevision,
      translationRevision,
      runtime: buildCurrentDerivedTagWorkingRuntime(),
    };
  }
  return workingRuntimeCache.runtime;
}

export function deriveCurrentRecordTagDerivation(input: DerivedTagContext): DerivedTagDerivation {
  return deriveRecordTagDerivationFromRuntime(getCurrentDerivedTagWorkingRuntime(), input);
}
