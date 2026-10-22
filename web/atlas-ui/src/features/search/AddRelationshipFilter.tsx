import { PlusOutlined, QuestionCircleOutlined } from "@ant-design/icons";
import { Button, Form, Popover, Radio, Select, Typography, theme } from "antd";
import { useRef, useState, type KeyboardEvent } from "react";
import type {
  RelationshipConstraintView,
  RelationshipDirectionView,
} from "../../generated/atlas";
import { MAX_RELATIONSHIP_FILTERS } from "../../shared/filters/relationshipUrl";
import {
  buildSearchRequest,
  DEFAULT_SEARCH_STATE,
} from "../../shared/filters/searchState";
import { rootNavigation } from "../../shared/records/useReferenceReading";
import { SearchPickerModal } from "../../shared/ui/pickers/SearchPickerModal";
import type { SearchWorkspaceState } from "./useSearchWorkspace";
import { sameRelationshipConstraint } from "./relationshipConstraints";

type LinkFilterForm = {
  direction: RelationshipDirectionView;
  includeContents: boolean;
};

export function AddRelationshipFilter({
  workspace,
}: {
  workspace: SearchWorkspaceState;
}) {
  const [open, setOpen] = useState(false);
  const [candidate, setCandidate] = useState<string | null>(null);
  const [feedback, setFeedback] = useState<string | null>(null);
  const [scopeHelpOpen, setScopeHelpOpen] = useState(false);
  const scopeHelpTrigger = useRef<HTMLButtonElement>(null);
  const { token } = theme.useToken();
  const [form] = Form.useForm<LinkFilterForm>();
  const direction = Form.useWatch("direction", form) ?? "incoming";
  const constraints = workspace.search.relationships ?? [];
  const atLimit = constraints.length >= MAX_RELATIONSHIP_FILTERS;
  const limitMessage = "Remove a link filter before adding another.";
  function close() {
    setOpen(false);
    setCandidate(null);
    setFeedback(null);
    setScopeHelpOpen(false);
    form.resetFields();
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
    if (!candidate) return;
    const constraint: RelationshipConstraintView = {
      direction: values.direction,
      include_contents: values.includeContents,
      selection: rootNavigation(candidate),
    };
    if (
      constraints.some((existing) => sameRelationshipConstraint(existing, constraint))
    ) {
      setFeedback("This link filter is already added.");
      return;
    }
    if (atLimit) {
      setFeedback(limitMessage);
      return;
    }
    workspace.setSearch({
      ...workspace.search,
      relationships: [...constraints, constraint],
    });
    close();
  }
  return (
    <>
      <Button
        icon={<PlusOutlined aria-hidden="true" />}
        disabled={atLimit}
        onClick={() => setOpen(true)}
      >
        Add link filter
      </Button>
      {atLimit && <Typography.Text role="status">{limitMessage}</Typography.Text>}
      <SearchPickerModal<LinkFilterForm>
        title="Add link filter"
        open={open}
        form={form}
        onCancel={close}
        onFinish={add}
        selectedRecordKey={candidate}
        onSelectedRecordKeyChange={(key) => {
          setCandidate(key);
          setFeedback(null);
        }}
        buildRequest={(query) =>
          buildSearchRequest({ ...DEFAULT_SEARCH_STATE, query }, 1)
        }
        emptyPrompt="Search or choose filters."
        emptyResults="No matching records."
        okButtonProps={{ disabled: atLimit }}
        status={
          feedback ? (
            <Typography.Text role="status">{feedback}</Typography.Text>
          ) : undefined
        }
      >
        <Form.Item name="direction" label="Direction" initialValue="incoming">
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
          initialValue={true}
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
    </>
  );
}
