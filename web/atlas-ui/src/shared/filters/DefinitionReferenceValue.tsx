import { Button, Select, Space, Typography } from "antd";
import { useQuery } from "@tanstack/react-query";
import { AtlasApiError, getDefinitionFilterOptions } from "../../api/atlasApi";
import type { QueryDefinitionReference } from "../../generated/atlas";
import { ErrorNotice } from "../feedback/ErrorNotice";
import {
  isArtifactGenerationChanged,
  referenceQueryMeta,
} from "../queries/referenceQueries";
import { sameReferenceCondition } from "./linkPredicates";
import { ReferenceHelpLink } from "../records/ReferenceHelp";

/** Read an already-selected exact reference; new selection belongs to its picker. */
export function DefinitionReferenceValue({
  reference,
  generation,
  disabled,
  onRefresh,
  onChange,
}: {
  reference: QueryDefinitionReference | undefined;
  generation: string | undefined;
  disabled?: boolean;
  onRefresh?: () => Promise<void>;
  onChange: (reference: QueryDefinitionReference) => void;
}) {
  const target = reference?.target;
  const options = useQuery({
    queryKey: ["definition-filter-selection", generation, target],
    enabled: !!target && !!generation,
    meta: referenceQueryMeta,
    staleTime: Infinity,
    queryFn: async ({ signal }) => {
      const response = await getDefinitionFilterOptions(
        {
          text: null,
          offset: 0,
          limit: 1,
          selected: target ? [target] : [],
          expected_artifact_generation: generation ?? null,
        },
        signal,
      );
      signal.throwIfAborted();
      if (response.artifact_generation !== generation)
        throw new AtlasApiError(409, "Reference collection changed", {
          code: "artifact_generation_changed",
          message: "Reference collection changed",
          recoverable_action: "refresh_results",
        });
      if (
        target &&
        response.selected.some(
          (entry) =>
            !sameReferenceCondition(
              { kind: "definition", reference: { target, include_contents: true } },
              {
                kind: "definition",
                reference: { target: entry.target, include_contents: true },
              },
            ),
        )
      )
        throw new Error("Unable to verify selected definition");
      return response;
    },
  });
  const changed = isArtifactGenerationChanged(options.error);
  const selected = changed ? undefined : options.data?.selected[0];
  return (
    <Space className="filter-rule-value" direction="vertical" align="start">
      <Typography.Text>
        <ReferenceHelpLink help={selected?.help} generation={generation}>
          {selected?.label ??
            (options.isPending && target
              ? "Loading definition…"
              : "Definition unavailable")}
        </ReferenceHelpLink>
      </Typography.Text>
      {options.error && (
        <ErrorNotice
          error={options.error}
          title="Definition unavailable"
          compact
          action={
            <Button
              disabled={changed && !onRefresh}
              onClick={() => (changed ? void onRefresh?.() : void options.refetch())}
            >
              {changed ? "Refresh filters" : "Retry"}
            </Button>
          }
        />
      )}
      {reference && (
        <Select
          aria-label="Definition reference scope"
          disabled={disabled}
          value={reference.include_contents ? "whole" : "main"}
          options={[
            { value: "whole", label: "Whole entry" },
            { value: "main", label: "Main entry only" },
          ]}
          onChange={(value) =>
            onChange({ ...reference, include_contents: value === "whole" })
          }
        />
      )}
    </Space>
  );
}
