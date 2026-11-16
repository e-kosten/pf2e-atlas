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
import { useRef, useState, type RefObject } from "react";
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

export function MarkdownEditor({
  value = "",
  onChange,
  id,
  label,
  disabled,
  readOnly,
  editorRef,
}: {
  value?: string;
  onChange?: (value: string) => void;
  id?: string;
  label?: string;
  disabled?: boolean;
  readOnly?: boolean;
  editorRef?: RefObject<GetRef<typeof Input.TextArea> | null>;
}) {
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

  function format(before: string, after = "") {
    const { start, end } = selection();
    onChange?.(
      value.slice(0, start) +
        before +
        value.slice(start, end) +
        after +
        value.slice(end),
    );
    selectRange(start + before.length, end + before.length);
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
    <div className="markdown-editor">
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
              type="text"
              disabled={locked}
              onClick={() => format("**", "**")}
            >
              Bold
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
      <div hidden={mode !== "Write"}>
        <Input.TextArea
          id={id}
          aria-label={label}
          ref={textArea}
          value={value}
          disabled={disabled ?? componentDisabled}
          readOnly={readOnly}
          autoSize={{ minRows: 3, maxRows: 10 }}
          onChange={(event) => {
            const next = event.currentTarget.value;
            const caret = event.currentTarget.selectionStart;
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
        <div className="markdown-editor__preview">
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
