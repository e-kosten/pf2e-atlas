import { linkScopeCopy } from "./linkScopeCopy";
import { useReferencePreview } from "../records/ReferencePreviewContext";
import {
  isPrimaryActivation,
  navigateDestination,
  destinationHref,
} from "../../app/navigation";
import { recordDestination } from "../../app/destinations";
import { QuestionCircleOutlined } from "@ant-design/icons";
import { Button, Form, Popover, Radio, Select, Typography } from "antd";
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
  parent,
  parentListRef,
  onCancel,
  onConfirm,
}: LinkPickerProps) {
  const reference = useReferencePreview();
  const [candidate, setCandidate] = useState<PickerSelection | null | undefined>(
    undefined,
  );
  const [feedback, setFeedback] = useState<string | null>(null);
  const [scopeHelpOpen, setScopeHelpOpen] = useState(false);
  const scopeHelpTrigger = useRef<HTMLButtonElement>(null);
  const [form] = Form.useForm<LinkFilterForm>();
  const [direction, setDirection] = useState<QueryRelationshipDirection>(
    initial?.direction ?? "incoming",
  );
  const [includeContents, setIncludeContents] = useState(
    initial?.include_contents ?? true,
  );
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
  const selection =
    candidate === undefined
      ? initial?.selection
      : candidate
        ? rootNavigation(candidate.record.record_key)
        : undefined;
  function add(_values: LinkFilterForm) {
    if (!selection) return;
    const relationship: QueryRelationship = {
      direction,
      include_contents: includeContents,
      selection,
    };
    setFeedback(onConfirm(relationship, candidate ?? null));
  }
  const controls = (
    <>
      <div className="link-picker-context__direction">
        <Radio.Group
          aria-label="Link filter direction"
          value={direction}
          onChange={(event) => {
            setDirection(event.target.value);
            setCandidate(null);
            setFeedback(selection ? "Direction changed. Choose a record." : null);
          }}
        >
          <Radio.Button value="incoming">Links to</Radio.Button>
          <Radio.Button value="outgoing">Linked from</Radio.Button>
        </Radio.Group>
      </div>
      <div className="link-picker-context__scope">
        <Select
          className="link-picker-context__scope-select"
          aria-label="Link filter scope"
          value={includeContents ? "whole" : "main"}
          onChange={(value) => {
            setIncludeContents(value === "whole");
            setCandidate(null);
            setFeedback(selection ? "Link scope changed. Choose a record." : null);
          }}
          options={[
            { value: "main", label: "Main entry only" },
            { value: "whole", label: "Whole entry" },
          ]}
        />
        <Popover
          trigger="click"
          placement="top"
          arrow={false}
          open={scopeHelpOpen}
          onOpenChange={setScopeHelpOpen}
          title="Where Atlas checks for links"
          styles={{ root: { width: 320, maxWidth: "calc(100vw - 32px)" } }}
          content={
            <div>
              <Typography.Paragraph>
                <strong>{direction === "incoming" ? "Links to" : "Linked from"}</strong>{" "}
                {direction === "incoming"
                  ? "finds records that contain a link to your selection."
                  : "finds records that your selection links to."}
              </Typography.Paragraph>
              <Typography.Paragraph>
                <strong>Main entry only:</strong> {linkScopeCopy[direction].main}
              </Typography.Paragraph>
              <Typography.Paragraph style={{ marginBottom: 0 }}>
                <strong>Whole entry:</strong> {linkScopeCopy[direction].whole}
              </Typography.Paragraph>
            </div>
          }
        >
          <Button
            ref={scopeHelpTrigger}
            type="text"
            icon={<QuestionCircleOutlined />}
            aria-label="About link scope"
            aria-expanded={scopeHelpOpen}
            onKeyDown={scopeHelpKeyDown}
            styles={{ root: { flexShrink: 0 } }}
          />
        </Popover>
      </div>
    </>
  );
  return (
    <SearchPickerModal<LinkFilterForm>
      confirmLabel={initial ? "Apply" : "Add"}
      title={initial ? "Edit link condition" : "Add link condition"}
      open={open}
      form={form}
      onCancel={close}
      onFinish={add}
      selectedRecordKey={selection?.record_key ?? null}
      onSelectionChange={(selection) => {
        setCandidate(selection);
        setFeedback(null);
      }}
      buildRequest={(query) => ({
        ...buildSearchRequest({ ...DEFAULT_SEARCH_STATE, query }, 1),
        connection_scope: {
          parent,
          list_ref: parentListRef,
          direction,
          include_contents: includeContents,
        },
      })}
      selectionContext={`${direction}:${includeContents}`}
      searchHeader={
        <div className="link-picker-context" role="group" aria-label="Link matching">
          <div className="link-picker-context__fields">{controls}</div>
          <Typography.Text
            type="secondary"
            className="link-picker-context__description"
          >
            {direction === "incoming"
              ? "Choose an entry that matching records link to."
              : "Choose an entry that links to matching records."}
          </Typography.Text>
        </div>
      }
      emptyPrompt="Search or choose filters."
      emptyResults="No matching records."
      status={
        feedback ? (
          <Typography.Text role="status">{feedback}</Typography.Text>
        ) : undefined
      }
      summary={
        selection && (
          <Typography.Link
            onClick={(event) => {
              if (!isPrimaryActivation(event)) return;
              event.preventDefault();
              if (reference) {
                reference.open(selection.record_key, event.currentTarget, selection);
                return;
              }
              navigateDestination(
                recordDestination({
                  recordKey: selection.record_key,
                  selection: selection,
                }),
              );
            }}
            href={destinationHref(
              recordDestination({
                recordKey: selection.record_key,
                selection: selection,
              }),
            )}
          >
            {candidate?.record.title ?? initialTitle ?? "Record unavailable"}
          </Typography.Link>
        )
      }
    />
  );
}
