import {
  Button,
  ConfigProvider,
  Dropdown,
  Form,
  Input,
  Segmented,
  Space,
  type GetRef,
} from "antd";
import { ChevronDown } from "lucide-react";
import { fromMarkdown } from "mdast-util-from-markdown";
import { useMemo, useRef, useState, type RefObject } from "react";
import { buildSearchRequest, DEFAULT_SEARCH_STATE } from "../filters/searchState";
import { SearchPickerModal } from "../ui/pickers/SearchPickerModal";
import { MarkdownDocument, markdownRecordLink } from "./MarkdownDocument";

type Selection = { start: number; end: number };
type Candidate = { key: string; title: string };

const headingItems = Array.from({ length: 6 }, (_, index) => ({
  key: String(index + 1),
  label: `Heading ${index + 1}`,
}));
const headingMarker = /^#{1,6}(?:[ \t]+|$)/;

type Emphasis = "strong" | "emphasis";

type MarkdownNode = {
  type: string;
  position?: { start: { offset?: number }; end: { offset?: number } };
  children?: MarkdownNode[];
};

function emphasisRanges(value: string) {
  const ranges: Record<Emphasis, Selection[]> = { strong: [], emphasis: [] };
  function visit({ type, position, children }: MarkdownNode) {
    if (type === "strong" || type === "emphasis") {
      const start = position?.start.offset;
      const end = position?.end.offset;
      if (start != null && end != null) ranges[type].push({ start, end });
    }
    children?.forEach(visit);
  }
  visit(fromMarkdown(value));
  return ranges;
}

export function MarkdownEditor({
  value = "",
  onChange,
  id,
  label,
  disabled,
  readOnly,
  editorRef,
  sizing = "compact",
}: {
  value?: string;
  onChange?: (value: string) => void;
  id?: string;
  label?: string;
  disabled?: boolean;
  readOnly?: boolean;
  sizing?: "compact" | "roomy" | "fill";
  editorRef?: RefObject<GetRef<typeof Input.TextArea> | null>;
}) {
  const [currentSelection, setCurrentSelection] = useState<Selection>({
    start: value.length,
    end: value.length,
  });
  const ranges = useMemo(() => emphasisRanges(value), [value]);
  const [mode, setMode] = useState<"Write" | "Preview">("Write");
  const [insertion, setInsertion] = useState<Selection | null>(null);
  const [candidate, setCandidate] = useState<Candidate | null>(null);
  const [form] = Form.useForm();
  const localRef = useRef<GetRef<typeof Input.TextArea>>(null);
  const textArea = editorRef ?? localRef;
  const { componentDisabled } = ConfigProvider.useConfig();
  const locked = !!(disabled || componentDisabled || readOnly);

  function nativeEditor() {
    const root = textArea.current?.nativeElement;
    return root instanceof HTMLTextAreaElement ? root : root?.querySelector("textarea");
  }

  function selectRange(start: number, end = start) {
    setCurrentSelection({ start, end });
    requestAnimationFrame(() => {
      const editor = nativeEditor();
      editor?.focus({ preventScroll: true });
      editor?.setSelectionRange(start, end);
    });
  }

  function selection(): Selection {
    const editor = nativeEditor();
    return {
      start: editor?.selectionStart ?? value.length,
      end: editor?.selectionEnd ?? value.length,
    };
  }

  function emphasisState(kind: Emphasis, { start, end }: Selection) {
    if (start !== end) {
      const selected = value.slice(start, end);
      const content = selected.trim();
      if (!content) {
        return { formatted: undefined, disabled: true, start, end };
      }
      start += selected.length - selected.trimStart().length;
      end = start + content.length;
    }
    const formatted = ranges[kind].find((range) =>
      start === end
        ? range.start < start && start < range.end
        : range.start <= start && end <= range.end,
    );
    if (formatted) return { formatted, disabled: false, start, end };
    if (start === end) {
      // Empty delimiter pairs have no CommonMark node until the author types.
      const left = value.slice(0, start).match(/\*+$/)?.[0].length;
      const right = value.slice(start).match(/^\*+/)?.[0].length;
      const length = kind === "strong" ? 2 : 1;
      if (left === length && right === length) {
        return {
          formatted: { start: start - length, end: end + length },
          disabled: false,
          start,
          end,
        };
      }
    }
    return {
      formatted: undefined,
      disabled:
        value.slice(start, end).includes("\n") ||
        ranges[kind].some((range) => start < range.end && end > range.start),
      start,
      end,
    };
  }

  const boldState = emphasisState("strong", currentSelection);
  const italicState = emphasisState("emphasis", currentSelection);

  function toggleEmphasis(kind: Emphasis) {
    const original = selection();
    const { formatted, disabled, start, end } = emphasisState(kind, original);
    if (disabled) return;
    const marker = kind === "strong" ? "**" : "*";
    if (formatted) {
      const innerStart = formatted.start + marker.length;
      const innerEnd = formatted.end - marker.length;
      onChange?.(
        value.slice(0, formatted.start) +
          value.slice(innerStart, innerEnd) +
          value.slice(formatted.end),
      );
      const moveOffset = (offset: number) =>
        offset -
        Math.min(marker.length, Math.max(0, offset - formatted.start)) -
        Math.min(marker.length, Math.max(0, offset - innerEnd));
      selectRange(moveOffset(original.start), moveOffset(original.end));
      return;
    }
    onChange?.(
      value.slice(0, start) +
        marker +
        value.slice(start, end) +
        marker +
        value.slice(end),
    );
    selectRange(
      original.start === start ? start + marker.length : original.start,
      original.end === end ? end + marker.length : original.end + 2 * marker.length,
    );
  }

  function formatLine(prefix: string, marker?: RegExp) {
    const { start, end } = selection();
    const lineStart = start > 0 ? value.lastIndexOf("\n", start - 1) + 1 : 0;
    const previous = marker?.exec(value.slice(lineStart))?.[0].length ?? 0;
    onChange?.(value.slice(0, lineStart) + prefix + value.slice(lineStart + previous));
    const delta = prefix.length - previous;
    selectRange(
      Math.max(lineStart + prefix.length, start + delta),
      Math.max(lineStart + prefix.length, end + delta),
    );
  }

  function cancelPicker() {
    if (insertion) selectRange(insertion.end);
    setInsertion(null);
    setCandidate(null);
  }

  function insertRecord() {
    if (!candidate || !insertion || locked) return;
    const label = value.slice(insertion.start, insertion.end).replace(/^\[\[$/, "");
    const link = markdownRecordLink(label || candidate.title, candidate.key);
    onChange?.(value.slice(0, insertion.start) + link + value.slice(insertion.end));
    selectRange(insertion.start + link.length);
    setInsertion(null);
    setCandidate(null);
  }

  return (
    <div className="markdown-editor" data-sizing={sizing}>
      <div className="markdown-editor__toolbar">
        <Segmented
          aria-label="Document editor mode"
          options={["Write", "Preview"]}
          value={mode}
          onChange={setMode}
        />
        {mode === "Write" && (
          <Space wrap size={4}>
            <Dropdown
              trigger={["click"]}
              disabled={locked}
              menu={{
                items: headingItems,
                onClick: ({ key }) =>
                  formatLine("#".repeat(Number(key)) + " ", headingMarker),
              }}
            >
              <Button size="small" type="text" disabled={locked}>
                Heading <ChevronDown size={13} aria-hidden="true" />
              </Button>
            </Dropdown>
            <Button
              size="small"
              type={boldState.formatted ? "default" : "text"}
              aria-pressed={!!boldState.formatted}
              disabled={locked || boldState.disabled}
              onClick={() => toggleEmphasis("strong")}
            >
              Bold
            </Button>
            <Button
              size="small"
              type={italicState.formatted ? "default" : "text"}
              aria-pressed={!!italicState.formatted}
              disabled={locked || italicState.disabled}
              onClick={() => toggleEmphasis("emphasis")}
            >
              Italic
            </Button>
            <Button
              size="small"
              type="text"
              disabled={locked}
              onClick={() => formatLine("- ")}
            >
              List
            </Button>
            <Button
              size="small"
              type="text"
              disabled={locked}
              onClick={() => setInsertion(selection())}
            >
              Insert record link
            </Button>
          </Space>
        )}
      </div>
      <div className="markdown-editor__write" hidden={mode !== "Write"}>
        <Input.TextArea
          id={id}
          aria-label={label}
          ref={textArea}
          value={value}
          disabled={disabled ?? componentDisabled}
          readOnly={readOnly}
          autoSize={sizing === "compact" ? { minRows: 3, maxRows: 10 } : false}
          style={
            sizing === "roomy"
              ? {
                  height: "clamp(9rem, 40dvh, 28rem)",
                  maxHeight: "60dvh",
                  resize: "vertical",
                }
              : undefined
          }
          onSelect={({ currentTarget }) => {
            setCurrentSelection({
              start: currentTarget.selectionStart,
              end: currentTarget.selectionEnd,
            });
          }}
          onChange={(event) => {
            const next = event.currentTarget.value;
            const caret = event.currentTarget.selectionStart;
            setCurrentSelection({
              start: caret,
              end: event.currentTarget.selectionEnd,
            });
            onChange?.(next);
            if (
              !locked &&
              !insertion &&
              next.length === value.length + 1 &&
              next.slice(caret - 2, caret) === "[["
            ) {
              setInsertion({ start: caret - 2, end: caret });
            }
          }}
        />
      </div>
      {mode === "Preview" && (
        <div
          className="markdown-editor__preview"
          role={sizing === "fill" ? "region" : undefined}
          aria-label={sizing === "fill" ? "Note preview" : undefined}
          tabIndex={sizing === "fill" ? 0 : undefined}
        >
          {value.trim() ? (
            <MarkdownDocument source={value} />
          ) : (
            <p>No text to preview.</p>
          )}
        </div>
      )}
      {insertion && (
        <SearchPickerModal
          title="Insert record link"
          focusSearchOnOpen
          footerLayout="actions"
          confirmLabel="Insert link"
          open
          form={form}
          onCancel={cancelPicker}
          onFinish={insertRecord}
          selectedRecordKey={candidate?.key ?? null}
          onSelectionChange={(item) =>
            setCandidate(
              item ? { key: item.record.record_key, title: item.record.title } : null,
            )
          }
          emptyPrompt="Search for an Atlas record."
          emptyResults="No matching records."
          buildRequest={(query) =>
            buildSearchRequest({ ...DEFAULT_SEARCH_STATE, query }, 1)
          }
        />
      )}
    </div>
  );
}
