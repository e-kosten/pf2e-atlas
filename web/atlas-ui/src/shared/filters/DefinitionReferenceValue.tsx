import { Button, Select, Space, Typography } from "antd";
import { useQuery } from "@tanstack/react-query";
import type { QueryDefinitionReference } from "../../generated/atlas";
import { ErrorNotice } from "../feedback/ErrorNotice";
import { isArtifactGenerationChanged } from "../queries/referenceQueries";
import { ReferenceHelpLink } from "../records/ReferenceHelp";
import { definitionFilterSelectionQueryOptions } from "./definitionFilterSelectionQuery";

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
  const options = useQuery(definitionFilterSelectionQueryOptions(target, generation));
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
