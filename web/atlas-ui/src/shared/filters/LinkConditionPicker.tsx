import { QuestionCircleOutlined } from "@ant-design/icons";
import { Button, Form, Popover, Radio, Select, Typography, theme } from "antd";
import { useRef, useState, type KeyboardEvent } from "react";
import type {
  QueryRelationship,
  QueryRelationshipDirection,
} from "../../generated/atlas";
import { buildSearchRequest, DEFAULT_SEARCH_STATE } from "./searchState";
import { rootNavigation } from "../records/useReferenceReading";
import { SearchPickerModal } from "../ui/pickers/SearchPickerModal";
import type { PickerSelection } from "../ui/pickers/SearchPickerModal";
import type { LinkPickerProps } from "./LinkPickerContext";
import {
  atlasRoutePath,
  navigateToAtlasRoute,
  shouldHandleAtlasRouteClick,
} from "../../app/routes";

type LinkFilterForm = {
  direction: QueryRelationshipDirection;
  includeContents: boolean;
};

export function LinkConditionPicker({
  open,
  initial,
  initialTitle,
  onCancel,
  onConfirm,
}: LinkPickerProps) {
  const [candidate, setCandidate] = useState<PickerSelection | null>(null);
  const [feedback, setFeedback] = useState<string | null>(null);
  const [scopeHelpOpen, setScopeHelpOpen] = useState(false);
  const scopeHelpTrigger = useRef<HTMLButtonElement>(null);
  const { token } = theme.useToken();
  const [form] = Form.useForm<LinkFilterForm>();
  const direction = Form.useWatch("direction", form) ?? "incoming";
  function close() {
    setScopeHelpOpen(false);
    onCancel();
  }
  function scopeHelpKeyDown(event: KeyboardEvent<HTMLElement>) {
    if (event.key === "Escape" && scopeHelpOpen) {
      event.preventDefault();
      event.stopPropagation();
      setScopeHelpOpen(false);
      scopeHelpTrigger.current?.focus({ preventScroll: true });
    }
  }
  function add(values: LinkFilterForm) {
    const selection = candidate
      ? rootNavigation(candidate.record.record_key)
      : initial?.selection;
    if (!selection) return;
    const relationship: QueryRelationship = {
      direction: values.direction,
      include_contents: values.includeContents,
      selection,
    };
    setFeedback(onConfirm(relationship, candidate));
  }
  return (
    <SearchPickerModal<LinkFilterForm>
      confirmLabel={initial ? "Apply" : "Add"}
      title={initial ? "Edit link condition" : "Add link condition"}
      open={open}
      form={form}
      onCancel={close}
      onFinish={add}
      selectedRecordKey={
        candidate?.record.record_key ?? initial?.selection.record_key ?? null
      }
      onSelectionChange={(selection) => {
        setCandidate(selection);
        setFeedback(null);
      }}
      buildRequest={(query) =>
        buildSearchRequest({ ...DEFAULT_SEARCH_STATE, query }, 1)
      }
      emptyPrompt="Search or choose filters."
      emptyResults="No matching records."
      status={
        feedback ? (
          <Typography.Text role="status">{feedback}</Typography.Text>
        ) : undefined
      }
    >
      {(candidate || initial) && (
        <Typography.Link
          onClick={(event) => {
            if (!shouldHandleAtlasRouteClick(event)) return;
            event.preventDefault();
            navigateToAtlasRoute({
              kind: "record",
              recordKey: candidate?.record.record_key ?? initial!.selection.record_key,
              selection: candidate
                ? rootNavigation(candidate.record.record_key)
                : initial!.selection,
            });
          }}
          href={atlasRoutePath({
            kind: "record",
            recordKey: candidate?.record.record_key ?? initial!.selection.record_key,
            selection: candidate
              ? rootNavigation(candidate.record.record_key)
              : initial!.selection,
          })}
        >
          {candidate?.record.title ?? initialTitle ?? "Record unavailable"}
        </Typography.Link>
      )}
      <Form.Item
        name="direction"
        label="Direction"
        initialValue={initial?.direction ?? "incoming"}
      >
        <Radio.Group
          aria-label="Link filter direction"
          onChange={() => setFeedback(null)}
        >
          <Radio.Button value="incoming">Links to</Radio.Button>
          <Radio.Button value="outgoing">Linked from</Radio.Button>
        </Radio.Group>
      </Form.Item>
      <Form.Item
        name="includeContents"
        style={{ minWidth: 0, width: 280, maxWidth: "100%" }}
        getValueProps={(value: boolean) => ({ value: value ? "whole" : "main" })}
        getValueFromEvent={(value: string) => value === "whole"}
        label={
          <span
            style={{
              display: "inline-flex",
              alignItems: "center",
              gap: token.marginXXS,
              maxWidth: "100%",
            }}
          >
            <span style={{ whiteSpace: "normal" }}>
              {direction === "incoming"
                ? "Look for links in matching records"
                : "Look for links in this entry"}
            </span>
            <Popover
              trigger="click"
              placement="bottom"
              arrow={false}
              open={scopeHelpOpen}
              onOpenChange={setScopeHelpOpen}
              styles={{ root: { maxWidth: "calc(100vw - 32px)" } }}
              content={
                <Typography.Paragraph style={{ margin: 0 }}>
                  {direction === "incoming"
                    ? "Whole entry also checks each matching record’s abilities, spells, equipment, and other nested documents."
                    : "Whole entry also checks this entry’s abilities, spells, equipment, and other nested documents."}
                </Typography.Paragraph>
              }
            >
              <Button
                ref={scopeHelpTrigger}
                type="text"
                size="small"
                icon={<QuestionCircleOutlined />}
                aria-label="About link scope"
                aria-expanded={scopeHelpOpen}
                onKeyDown={scopeHelpKeyDown}
                styles={{ root: { flexShrink: 0 } }}
              />
            </Popover>
          </span>
        }
        initialValue={initial?.include_contents ?? true}
      >
        <Select
          aria-label="Link filter scope"
          onChange={() => setFeedback(null)}
          style={{ width: "100%" }}
          options={[
            { value: "main", label: "Main entry only" },
            { value: "whole", label: "Whole entry" },
          ]}
        />
      </Form.Item>
    </SearchPickerModal>
  );
}
