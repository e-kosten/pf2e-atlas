import { useEffect, useState } from "react";
import { Button, Space, Spin, Typography } from "antd";
import type { DefinitionFilterOptionView } from "../../generated/atlas";
import { ChoicePicker, ChoicePickerAction } from "../ui/pickers/ChoicePicker";
import { ReferenceHelpLink } from "../records/ReferenceHelp";
import { ErrorNotice } from "../feedback/ErrorNotice";
import { isArtifactGenerationChanged } from "../queries/referenceQueries";
import {
  ReferenceFilterActions,
  type ReferenceFilterActionHandler,
} from "./ReferenceFilterActions";
import {
  DEFINITION_PAGE_SIZE,
  definitionTargetIdentity,
  useDefinitionFilterOptions,
} from "./useDefinitionFilterOptions";

/** Discover all eligible definitions, then choose one supplied relationship. */
export function DefinitionFilterPicker({
  generation,
  onAction,
  onRefresh,
}: {
  generation?: string;
  onAction: ReferenceFilterActionHandler;
  onRefresh?: () => Promise<void>;
}) {
  const [open, setOpen] = useState(false);
  const [text, setText] = useState("");
  const [search, setSearch] = useState("");
  const [offset, setOffset] = useState(0);
  const [selection, setSelection] = useState<DefinitionFilterOptionView | null>(null);
  useEffect(() => {
    const timeout = window.setTimeout(() => setSearch(text), 250);
    return () => window.clearTimeout(timeout);
  }, [text]);
  const query = useDefinitionFilterOptions({
    generation,
    text: search,
    offset,
    selected: selection ? [selection.target] : [],
    enabled: open,
  });
  const changed = isArtifactGenerationChanged(query.error);
  const data = changed ? undefined : query.data;
  const selected = data?.selected.find(
    (option) =>
      selection &&
      definitionTargetIdentity(option.target) ===
        definitionTargetIdentity(selection.target),
  );
  const options = [...(data?.options ?? []), ...(selected ? [selected] : [])];
  const byId = new Map(
    options.map((option) => [definitionTargetIdentity(option.target), option]),
  );
  const choices = (data?.options ?? []).map((option) => ({
    value: definitionTargetIdentity(option.target),
    label: option.label,
  }));
  const active =
    selected ??
    (selection && data?.artifact_generation === generation ? selection : null);
  const pages =
    ((data?.total ?? 0n) + BigInt(DEFINITION_PAGE_SIZE) - 1n) /
    BigInt(DEFINITION_PAGE_SIZE);
  return (
    <ChoicePicker
      id="definition-filter"
      label="Definition"
      choices={choices}
      selected={selection ? [definitionTargetIdentity(selection.target)] : []}
      stateOf={(value) =>
        selection && definitionTargetIdentity(selection.target) === value
          ? "include"
          : "neutral"
      }
      onChange={(value, state) => {
        if (state === "neutral") setSelection(null);
        else {
          const option = byId.get(value);
          if (option) setSelection(option);
        }
      }}
      labelForValue={(value) =>
        byId.get(value)?.label ??
        (selection && definitionTargetIdentity(selection.target) === value
          ? selection.label
          : "Definition unavailable")
      }
      valueEditing={{ triggerText: "Find a definition" }}
      filterChoices={false}
      open={open}
      onOpenChange={(next) => {
        setOpen(next);
        if (!next) setOffset(0);
      }}
      text={text}
      onTextChange={(next) => {
        setText(next);
        setOffset(0);
      }}
      loading={query.isFetching}
      pageKey={JSON.stringify([generation, search, data?.offset])}
      optionAction={(option) => {
        const help = byId.get(option.value)?.help;
        return help?.destination ? (
          <ChoicePickerAction>
            {(handoff) => (
              <ReferenceHelpLink help={help} generation={generation} handoff={handoff}>
                Read<span className="sr-only"> {option.label}</span>
              </ReferenceHelpLink>
            )}
          </ChoicePickerAction>
        ) : null;
      }}
    >
      {query.error && (
        <ErrorNotice
          error={query.error}
          title="Definitions could not be loaded"
          compact
          action={
            <Button
              disabled={changed && !onRefresh}
              onClick={() => (changed ? void onRefresh?.() : void query.refetch())}
            >
              {changed ? "Refresh filters" : "Retry"}
            </Button>
          }
        />
      )}
      {data && (
        <div className="common-choice-footer">
          <div className="common-choice-progress">
            <Typography.Text type="secondary" role="status">
              {new Intl.NumberFormat().format(data.total)} definitions
              {pages > 1n
                ? ` · Page ${data.offset / DEFINITION_PAGE_SIZE + 1} of ${pages}`
                : ""}
            </Typography.Text>
            <Spin spinning={query.isFetching} size="small" delay={500} />
          </div>
          <Space>
            <Button
              size="small"
              aria-label="Previous definitions"
              disabled={offset === 0 || query.isFetching}
              onClick={() => setOffset(Math.max(0, offset - DEFINITION_PAGE_SIZE))}
            >
              Previous
            </Button>
            <Button
              size="small"
              aria-label="More definitions"
              disabled={
                BigInt(offset + DEFINITION_PAGE_SIZE) >= data.total || query.isFetching
              }
              onClick={() => setOffset(offset + DEFINITION_PAGE_SIZE)}
            >
              Next
            </Button>
          </Space>
        </div>
      )}
      {active && (
        <section aria-label={`Filters for ${active.label}`}>
          <Typography.Text strong>{active.label}</Typography.Text>
          <ReferenceFilterActions
            actions={active.help.related_filters}
            generation={generation}
            onAction={(action) => {
              if (query.error || !selected || data?.artifact_generation !== generation)
                return "Refresh definitions before adding this filter.";
              return onAction(action);
            }}
          />
        </section>
      )}
    </ChoicePicker>
  );
}
