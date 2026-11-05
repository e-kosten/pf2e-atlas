import {
  isPrimaryActivation,
  navigateDestination,
  destinationHref,
} from "../../app/navigation";
import { recordDestination } from "../../app/destinations";
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
      summary={
        (candidate || initial) && (
          <Typography.Link
            onClick={(event) => {
              if (!isPrimaryActivation(event)) return;
              event.preventDefault();
              navigateDestination(
                recordDestination({
                  recordKey:
                    candidate?.record.record_key ?? initial!.selection.record_key,
                  selection: candidate
                    ? rootNavigation(candidate.record.record_key)
                    : initial!.selection,
                }),
              );
            }}
            href={destinationHref(
              recordDestination({
                recordKey:
                  candidate?.record.record_key ?? initial!.selection.record_key,
                selection: candidate
                  ? rootNavigation(candidate.record.record_key)
                  : initial!.selection,
              }),
            )}
          >
            {candidate?.record.title ?? initialTitle ?? "Record unavailable"}
          </Typography.Link>
        )
      }
    >
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
        style={{ minWidth: 0, width: 160, maxWidth: "100%" }}
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
            <span>Link scope</span>
            <Popover
              trigger="click"
              placement="topLeft"
              arrow={false}
              open={scopeHelpOpen}
              onOpenChange={setScopeHelpOpen}
              title="Where Atlas checks for links"
              styles={{ root: { width: 320, maxWidth: "calc(100vw - 32px)" } }}
              content={
                <div>
                  <Typography.Paragraph>
                    <strong>
                      {direction === "incoming" ? "Links to" : "Linked from"}
                    </strong>{" "}
                    {direction === "incoming"
                      ? "finds records that contain a link to your selection."
                      : "finds records that your selection links to."}
                  </Typography.Paragraph>
                  <Typography.Paragraph>
                    <strong>Main entry only:</strong>{" "}
                    {direction === "incoming"
                      ? "Check links in each result’s main entry."
                      : "Check links in the selected entry itself."}
                  </Typography.Paragraph>
                  <Typography.Paragraph style={{ marginBottom: 0 }}>
                    <strong>Whole entry:</strong>{" "}
                    {direction === "incoming"
                      ? "Also check links in each result’s contents, such as its abilities, spells, and equipment."
                      : "Also check links in the selected entry’s contents, such as its abilities, spells, and equipment."}
                  </Typography.Paragraph>
                </div>
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
