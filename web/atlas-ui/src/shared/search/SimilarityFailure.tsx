import { Button, Space } from "antd";
import { useQuery } from "@tanstack/react-query";
import type { ReactNode } from "react";
import { AtlasLink } from "../../app/AtlasLink";
import { recordDestination } from "../../app/destinations";
import type { SimilaritySeed } from "../../generated/atlas";
import { ErrorNotice } from "../feedback/ErrorNotice";
import { appError } from "../feedback/errorMessages";
import type { SearchFormState } from "../filters/searchState";
import { useSearchProfile } from "./SearchProfileContext";
import { similarityPartsQueryOptions } from "./similarityQueries";
import { seedSearchFields } from "./similaritySource";
import {
  hasCoreSourcePreference,
  preferredSourceSections,
  similarityFromPreferences,
  similarityPreferences,
} from "./similarityState";

export type ComparisonControl = "sections" | "filters" | "settings" | "reselect";

/** The same recovery keeps the selected source and effective criteria in both entry points. */
export function SimilarityFailure({
  error,
  seed,
  search,
  generation,
  parentTitle,
  onRecover,
  fallback,
}: {
  error: unknown;
  seed: SimilaritySeed;
  search: SearchFormState;
  generation?: string | null;
  parentTitle?: string;
  onRecover: (next: SearchFormState, control?: ComparisonControl) => void;
  fallback: ReactNode;
}) {
  const code = appError(error)?.code;
  const profile = useSearchProfile();
  const embedded = Boolean(seed.embedded_path?.length);
  const parent: SimilaritySeed = { kind: "record", record_key: seed.record_key };
  const canCompareParent =
    embedded &&
    !!parentTitle &&
    (code === "similarity_no_content" ||
      code === "similarity_method_unavailable" ||
      code === "vector_readiness_required");
  const parentMethod = similarityFromPreferences(profile.preferences.recordSimilarity);
  const needsParentPreset =
    canCompareParent &&
    parentMethod.method !== "baseline" &&
    hasCoreSourcePreference(profile.preferences);
  const parentInventory = useQuery({
    ...similarityPartsQueryOptions(parent, profile.id, generation),
    enabled: needsParentPreset && profile.id !== null && profile.storedVectors,
  });
  const parentCore =
    profile.id &&
    profile.storedVectors &&
    parentInventory.isSuccess &&
    !parentInventory.isFetching
      ? parentInventory.data?.core
      : null;
  const compareParent = canCompareParent && (
    <Button
      size="small"
      disabled={needsParentPreset && !parentCore}
      onClick={() =>
        onRecover({
          ...search,
          ...seedSearchFields(parent),
          similarity: parentMethod,
          sourceSections:
            parentMethod.method === "baseline"
              ? undefined
              : parentCore
                ? preferredSourceSections(profile.preferences, parentCore.family)
                : "all",
          similarityGeneration: undefined,
        })
      }
    >
      Compare {parentTitle}
    </Button>
  );
  let action: ReactNode;
  let description: string | undefined;
  let title = "Unable to compare this source";
  switch (code) {
    case "vector_readiness_required": {
      if (!profile.storedVectors) return <>{fallback}</>;
      const usesSections = search.similarity?.method !== "baseline";
      title = "Comparison content unavailable";
      description = usesSections
        ? "Some content has no stored vectors for this model. Choose available sections or change the model in Search settings."
        : "This model has no usable vectors for this comparison. Review the model and comparison method in Search settings.";
      action = (
        <Space wrap>
          <Button
            size="small"
            onClick={() => onRecover(search, usesSections ? "sections" : "settings")}
          >
            {usesSections ? "Choose sections" : "Search settings"}
          </Button>
          {compareParent}
        </Space>
      );
      break;
    }
    case "artifact_generation_changed":
      if (search.sourceSections !== "custom" || !search.similarityGeneration)
        return <>{fallback}</>;
      description =
        "The reference collection changed. Choose sections from the current document to update this comparison.";
      action = (
        <Button size="small" onClick={() => onRecover(search, "reselect")}>
          Choose sections
        </Button>
      );
      break;
    case "similarity_sections_stale":
      action = (
        <Button size="small" onClick={() => onRecover(search, "reselect")}>
          Choose sections
        </Button>
      );
      break;
    case "similarity_source_limit":
      action = (
        <Button size="small" onClick={() => onRecover(search, "sections")}>
          Choose sections
        </Button>
      );
      break;
    case "similarity_candidate_limit":
      action = (
        <Button size="small" onClick={() => onRecover(search, "filters")}>
          Narrow results
        </Button>
      );
      break;
    case "similarity_no_content":
      action = embedded ? (
        compareParent
      ) : (
        <Button
          size="small"
          onClick={() =>
            onRecover({
              ...search,
              similarity: { method: "baseline" },
              sourceSections: undefined,
              similarityGeneration: undefined,
            })
          }
        >
          Use name and identity
        </Button>
      );
      break;
    case "similarity_method_unavailable": {
      const similarity = search.similarity ?? parentMethod;
      const identityOnly = similarity.method === "baseline";
      action = (
        <Space wrap>
          <Button
            size="small"
            onClick={() =>
              onRecover({
                ...search,
                similarity: similarityFromPreferences({
                  ...similarityPreferences(
                    similarity,
                    profile.preferences.recordSimilarity,
                  ),
                  method: identityOnly ? "passages" : similarity.method,
                }),
                sourceSections: "all",
                similarityGeneration: undefined,
              })
            }
          >
            {identityOnly ? "Use passage matches" : "Use all sections"}
          </Button>
          {compareParent}
        </Space>
      );
      break;
    }
    case "similarity_source_stale":
      action = (
        <AtlasLink {...recordDestination({ recordKey: seed.record_key })}>
          Open parent record
        </AtlasLink>
      );
      break;
    case "record_not_found":
      break;
    default:
      return <>{fallback}</>;
  }
  return (
    <>
      <ErrorNotice
        error={error}
        title={title}
        description={description}
        action={action}
      />
      {needsParentPreset && parentInventory.error && (
        <ErrorNotice
          error={parentInventory.error}
          title="Unable to load parent comparison settings"
          action={
            <Button size="small" onClick={() => void parentInventory.refetch()}>
              Retry
            </Button>
          }
        />
      )}
    </>
  );
}
