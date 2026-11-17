import {
  defaultKeymap,
  history,
  historyKeymap,
  insertNewline,
  redo,
  redoDepth,
  undo,
  undoDepth,
} from "@codemirror/commands";
import {
  insertNewlineContinueMarkupCommand,
  markdown,
} from "@codemirror/lang-markdown";
import { Compartment, EditorState, type Extension } from "@codemirror/state";
import { EditorView, keymap, type ViewUpdate } from "@codemirror/view";
import { Button, ConfigProvider, Dropdown, Form, Segmented, Tooltip } from "antd";
import {
  Bold,
  ChevronDown,
  Heading,
  Italic,
  Link2,
  List,
  ListOrdered,
  Redo2,
  Undo2,
} from "lucide-react";
import {
  useEffectEvent,
  useImperativeHandle,
  useLayoutEffect,
  useRef,
  useState,
  type AriaAttributes,
  type ReactNode,
  type RefObject,
} from "react";
import { buildSearchRequest, DEFAULT_SEARCH_STATE } from "../filters/searchState";
import { SearchPickerModal } from "../ui/pickers/SearchPickerModal";
import { MarkdownDocument, markdownRecordLink } from "./MarkdownDocument";
import {
  emphasisState,
  insertRecordLink,
  setHeading,
  toggleEmphasis,
  toggleList,
} from "./markdownCommands";

type Range = { from: number; to: number };
type Candidate = { key: string; title: string };
type FieldAttributes = Pick<
  AriaAttributes,
  "aria-describedby" | "aria-invalid" | "aria-required"
>;

export type MarkdownEditorHandle = { focus(options?: FocusOptions): void };

const headingItems = Array.from({ length: 6 }, (_, index) => ({
  key: String(index + 1),
  label: `Heading ${index + 1}`,
}));
const field = new Compartment();
const continueList = insertNewlineContinueMarkupCommand({ nonTightLists: false });
const editorExtensions = [
  history(),
  markdown({ addKeymap: false, completeHTMLTags: false, pasteURLAsLink: false }),
  keymap.of([
    {
      key: "Enter",
      run: (view) =>
        !view.state.readOnly && (continueList(view) || insertNewline(view)),
    },
    { key: "Mod-b", run: (view) => toggleEmphasis(view, "strong") },
    { key: "Mod-i", run: (view) => toggleEmphasis(view, "emphasis") },
    ...historyKeymap,
    ...defaultKeymap,
  ]),
  EditorView.lineWrapping,
];

function readControls(state: EditorState) {
  const bold = emphasisState(state, "strong");
  const italic = emphasisState(state, "emphasis");
  return {
    undo: undoDepth(state) > 0,
    redo: redoDepth(state) > 0,
    bold: !!bold.formatted,
    italic: !!italic.formatted,
    boldDisabled: bold.disabled,
    italicDisabled: italic.disabled,
  };
}

function sameControls(
  a: ReturnType<typeof readControls>,
  b: ReturnType<typeof readControls>,
) {
  return (
    a.undo === b.undo &&
    a.redo === b.redo &&
    a.bold === b.bold &&
    a.italic === b.italic &&
    a.boldDisabled === b.boldDisabled &&
    a.italicDisabled === b.italicDisabled
  );
}

function FormatButton({
  label,
  icon,
  disabled,
  pressed,
  shortcut,
  onClick,
}: {
  label: string;
  icon: ReactNode;
  disabled: boolean;
  pressed?: boolean;
  shortcut?: string;
  onClick: () => void;
}) {
  return (
    <Tooltip title={label} trigger={["hover", "focus"]}>
      <Button
        size="small"
        type="text"
        icon={icon}
        aria-label={label}
        aria-pressed={pressed}
        aria-keyshortcuts={shortcut}
        disabled={disabled}
        onClick={onClick}
      />
    </Tooltip>
  );
}

export function MarkdownEditor({
  value = "",
  onChange,
  id,
  label,
  disabled,
  readOnly,
  editorRef,
  roomy = false,
  "aria-describedby": describedBy,
  "aria-invalid": invalid,
  "aria-required": required,
}: FieldAttributes & {
  value?: string;
  onChange?: (value: string) => void;
  id?: string;
  label: string;
  disabled?: boolean;
  readOnly?: boolean;
  roomy?: boolean;
  editorRef?: RefObject<MarkdownEditorHandle | null>;
}) {
  const [mode, setMode] = useState<"Write" | "Preview">("Write");
  const [pickerOpen, setPickerOpen] = useState(false);
  const [candidate, setCandidate] = useState<Candidate | null>(null);
  const [form] = Form.useForm();
  const source = useRef<HTMLDivElement>(null);
  const view = useRef<EditorView | null>(null);
  const insertion = useRef<Range | null>(null);
  const restoreFocus = useRef<number | undefined>(undefined);
  const [controls, setControls] = useState(() => readControls(EditorState.create()));
  const { componentDisabled } = ConfigProvider.useConfig();
  const inactive = !!(disabled || componentDisabled);
  const locked = inactive || !!readOnly;
  const writing = mode === "Write";

  function openPicker(range: Range) {
    insertion.current = range;
    setPickerOpen(true);
  }

  const publishControls = useEffectEvent((state: EditorState) => {
    const next = readControls(state);
    setControls((current) => (sameControls(current, next) ? current : next));
  });

  const onEditorUpdate = useEffectEvent((update: ViewUpdate) => {
    if (update.docChanged || update.selectionSet || update.transactions.length) {
      publishControls(update.state);
    }

    if (!update.docChanged) return;
    if (insertion.current) {
      const { from, to } = insertion.current;
      insertion.current = {
        from: update.changes.mapPos(from, -1),
        to: update.changes.mapPos(to, 1),
      };
    }
    onChange?.(update.state.sliceDoc());

    const caret = update.state.selection.main;
    if (
      !locked &&
      !insertion.current &&
      caret.empty &&
      caret.head >= 2 &&
      update.state.doc.length === update.startState.doc.length + 1 &&
      !update.view.composing &&
      update.transactions.some((transaction) =>
        transaction.isUserEvent("input.type"),
      ) &&
      update.state.doc.sliceString(caret.head - 2, caret.head) === "[["
    ) {
      openPicker({ from: caret.head - 2, to: caret.head });
    }
  });

  const fieldExtensions = useEffectEvent((): Extension => {
    const attributes: Record<string, string> = { "aria-label": label };
    if (id) attributes.id = id;
    if (describedBy) attributes["aria-describedby"] = describedBy;
    if (invalid !== undefined) attributes["aria-invalid"] = String(invalid);
    if (required !== undefined) attributes["aria-required"] = String(required);
    if (inactive) {
      attributes["aria-disabled"] = "true";
      attributes.tabindex = "-1";
    } else if (readOnly) {
      attributes.tabindex = "0";
    }
    return [
      EditorView.contentAttributes.of(attributes),
      EditorView.editable.of(!locked),
      EditorState.readOnly.of(locked),
    ];
  });

  const createState = useEffectEvent(() =>
    EditorState.create({
      doc: value,
      selection: { anchor: value.replace(/\r\n?/g, "\n").length },
      extensions: [
        editorExtensions,
        field.of(fieldExtensions()),
        EditorView.updateListener.of(onEditorUpdate),
      ],
    }),
  );

  useLayoutEffect(() => {
    const editor = new EditorView({ parent: source.current!, state: createState() });
    view.current = editor;
    return () => {
      if (restoreFocus.current !== undefined)
        cancelAnimationFrame(restoreFocus.current);
      view.current = null;
      editor.destroy();
    };
  }, []);

  useLayoutEffect(() => {
    const editor = view.current;
    if (!editor || editor.state.doc.eq(editor.state.toText(value))) return;
    // An authoritative replacement starts a new source session and history.
    insertion.current = null;
    setPickerOpen(false);
    setCandidate(null);
    editor.setState(createState());
    publishControls(editor.state);
  }, [value]);

  useLayoutEffect(() => {
    view.current?.dispatch({ effects: field.reconfigure(fieldExtensions()) });
  }, [id, label, describedBy, invalid, required, inactive, readOnly, locked]);

  useLayoutEffect(() => {
    const editor = view.current;
    if (!editor || !id || !writing || inactive) return;
    // A contenteditable div cannot be the native control of a label element.
    const form = editor.contentDOM.closest("form");
    const label = Array.from(form?.querySelectorAll("label") ?? []).find(
      (element) => element.htmlFor === id,
    );
    if (!label) return;
    const focus = () => editor.focus();
    label.addEventListener("click", focus);
    return () => label.removeEventListener("click", focus);
  }, [id, writing, inactive]);

  useImperativeHandle(
    editorRef,
    () => ({
      focus: (options) => {
        if (writing && !inactive) view.current?.contentDOM.focus(options);
      },
    }),
    [inactive, writing],
  );

  function closePicker() {
    insertion.current = null;
    setPickerOpen(false);
    setCandidate(null);
    restoreFocus.current = requestAnimationFrame(() => {
      restoreFocus.current = undefined;
      if (!inactive) view.current?.focus();
    });
  }

  function insertRecord() {
    const editor = view.current;
    const range = insertion.current;
    if (!candidate || !range || !editor || locked) return;
    const text = editor.state.doc
      .sliceString(range.from, range.to)
      .replace(/^\[\[$/, "");
    insertion.current = null;
    insertRecordLink(
      editor,
      range,
      markdownRecordLink(text || candidate.title, candidate.key),
    );
    closePicker();
  }

  function run(command: (editor: EditorView) => void) {
    const editor = view.current;
    if (!editor || locked) return;
    command(editor);
    editor.focus();
  }

  return (
    <div
      className="markdown-editor"
      data-roomy={roomy || undefined}
      data-invalid={invalid || undefined}
    >
      <div className="markdown-editor__toolbar">
        <div className="markdown-editor__mode">
          <ConfigProvider
            theme={{
              components: {
                Segmented: {
                  itemSelectedBg: "var(--panel-elevated)",
                  itemSelectedColor: "var(--text)",
                  trackBg: "var(--panel)",
                },
              },
            }}
          >
            <Segmented
              size="small"
              aria-label="Document editor mode"
              options={["Write", "Preview"]}
              value={mode}
              onChange={setMode}
            />
          </ConfigProvider>
          <div
            className="markdown-editor__group markdown-editor__history"
            role="group"
            aria-label="Edit history"
            inert={!writing}
            aria-hidden={!writing || undefined}
          >
            <FormatButton
              label="Undo"
              icon={<Undo2 size={16} />}
              disabled={locked || !controls.undo}
              onClick={() => run(undo)}
            />
            <FormatButton
              label="Redo"
              icon={<Redo2 size={16} />}
              disabled={locked || !controls.redo}
              onClick={() => run(redo)}
            />
          </div>
        </div>
        <div
          className="markdown-editor__actions"
          role="group"
          aria-label="Markdown formatting"
          inert={!writing}
          aria-hidden={!writing || undefined}
        >
          <div className="markdown-editor__group">
            <Dropdown
              trigger={["click"]}
              disabled={locked}
              menu={{
                items: headingItems,
                onClick: ({ key }) => run((editor) => setHeading(editor, Number(key))),
              }}
            >
              <Button
                size="small"
                type="text"
                aria-label="Heading"
                aria-haspopup="menu"
                title="Heading"
                disabled={locked}
              >
                <Heading size={16} aria-hidden="true" />{" "}
                <ChevronDown size={12} aria-hidden="true" />
              </Button>
            </Dropdown>
            <FormatButton
              label="Bold"
              icon={<Bold size={16} />}
              pressed={controls.bold}
              shortcut="Control+B Meta+B"
              disabled={locked || controls.boldDisabled}
              onClick={() => run((editor) => toggleEmphasis(editor, "strong"))}
            />
            <FormatButton
              label="Italic"
              icon={<Italic size={16} />}
              pressed={controls.italic}
              shortcut="Control+I Meta+I"
              disabled={locked || controls.italicDisabled}
              onClick={() => run((editor) => toggleEmphasis(editor, "emphasis"))}
            />
          </div>
          <div className="markdown-editor__group">
            <FormatButton
              label="Bulleted list"
              icon={<List size={16} />}
              disabled={locked}
              onClick={() => run((editor) => toggleList(editor, false))}
            />
            <FormatButton
              label="Numbered list"
              icon={<ListOrdered size={16} />}
              disabled={locked}
              onClick={() => run((editor) => toggleList(editor, true))}
            />
          </div>
          <div className="markdown-editor__group">
            <Button
              size="small"
              type="text"
              icon={<Link2 size={16} />}
              disabled={locked}
              onClick={() => {
                const selected = view.current?.state.selection.main;
                if (selected) openPicker({ from: selected.from, to: selected.to });
              }}
            >
              Record link
            </Button>
          </div>
        </div>
      </div>
      <div className="markdown-editor__body">
        <div
          className="markdown-editor__source"
          ref={source}
          inert={!writing}
          aria-hidden={!writing || undefined}
        />
        {!writing && (
          <div
            className="markdown-editor__preview"
            role="region"
            aria-label="Markdown preview"
            // eslint-disable-next-line jsx-a11y/no-noninteractive-tabindex -- The bounded preview needs focus for keyboard scrolling in WebKit.
            tabIndex={0}
          >
            {value.trim() ? (
              <MarkdownDocument source={value} />
            ) : (
              <p>No text to preview.</p>
            )}
          </div>
        )}
      </div>
      {pickerOpen && (
        <SearchPickerModal
          title="Insert record link"
          focusSearchOnOpen
          footerLayout="actions"
          confirmLabel="Insert link"
          open
          form={form}
          onCancel={closePicker}
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
