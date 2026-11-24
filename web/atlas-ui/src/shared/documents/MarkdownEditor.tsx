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
import { useDesktopTab } from "../../app/DesktopTabContext";
import { SearchPickerModal } from "../ui/pickers/SearchPickerModal";
import {
  InlineRecordPicker,
  readInlineRecordQuery,
  type InlineRecordQuery,
} from "./InlineRecordPicker";
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
    // Atlas overlays own Escape and preserve the selection when collapsing notes.
    ...defaultKeymap.filter(({ key }) => key !== "Escape"),
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
  sizing = "compact",
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
  sizing?: "compact" | "roomy" | "fill";
  editorRef?: RefObject<MarkdownEditorHandle | null>;
}) {
  const [mode, setMode] = useState<"Write" | "Preview">("Write");
  const tabActive = useDesktopTab()?.active ?? true;
  const [headingOpen, setHeadingOpen] = useState(false);
  if (!tabActive && headingOpen) setHeadingOpen(false);
  const [picker, setPicker] = useState<{ initialQuery?: string } | null>(null);
  const [inline, setInline] = useState<{
    editor: EditorView;
    query: InlineRecordQuery;
  } | null>(null);
  const inlineQuery = useRef<InlineRecordQuery | null>(null);
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

  function dismissInline() {
    inlineQuery.current = null;
    setInline(null);
  }

  function openPicker(range: Range, initialQuery?: string) {
    dismissInline();
    insertion.current = range;
    setPicker({ initialQuery });
  }

  const publishControls = useEffectEvent((state: EditorState) => {
    const next = readControls(state);
    setControls((current) => (sameControls(current, next) ? current : next));
  });

  const onEditorUpdate = useEffectEvent((update: ViewUpdate) => {
    if (update.docChanged || update.selectionSet || update.transactions.length) {
      publishControls(update.state);
    }

    if (update.docChanged || update.selectionSet) {
      const next =
        writing && tabActive && !locked && !insertion.current
          ? readInlineRecordQuery(update, inlineQuery.current)
          : null;
      if (next !== inlineQuery.current) {
        inlineQuery.current = next;
        setInline(next ? { editor: update.view, query: next } : null);
      }
    }
    if (update.docChanged) {
      if (insertion.current) {
        const { from, to } = insertion.current;
        insertion.current = {
          from: update.changes.mapPos(from, -1),
          to: update.changes.mapPos(to, 1),
        };
      }
      onChange?.(update.state.sliceDoc());
    }
  });

  function resumeInline() {
    const editor = view.current;
    const query = inlineQuery.current;
    if (!editor || !query || inline || !writing || locked || !tabActive) return;
    const caret = editor.state.selection.main;
    if (caret.empty && caret.head >= query.from + 2 && caret.head <= query.to)
      setInline({ editor, query });
  }

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
    dismissInline();
    insertion.current = null;
    setPicker(null);
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

  if (inline && (!writing || locked || !tabActive)) setInline(null);
  useLayoutEffect(() => {
    if (!writing || locked || !tabActive) inlineQuery.current = null;
  }, [writing, locked, tabActive]);

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
    const editor = view.current;
    const state = editor?.state;
    insertion.current = null;
    setPicker(null);
    setCandidate(null);
    restoreFocus.current = requestAnimationFrame(() => {
      restoreFocus.current = undefined;
      if (
        inactive ||
        !editor ||
        view.current !== editor ||
        editor.contentDOM.closest("[inert]")
      )
        return;
      editor.focus();
      if (!state || editor.state.doc !== state.doc) return;
      const { anchor, head, empty } = state.selection.main;
      const selection = editor.contentDOM.ownerDocument.getSelection();
      if (empty || !selection?.isCollapsed) return;
      // WebKit can collapse the DOM selection when focus returns from the picker.
      const start = editor.domAtPos(anchor);
      const end = editor.domAtPos(head);
      if (
        editor.posAtDOM(start.node, start.offset) === anchor &&
        editor.posAtDOM(end.node, end.offset) === head
      ) {
        selection.setBaseAndExtent(start.node, start.offset, end.node, end.offset);
      }
    });
  }

  function insertRecord() {
    const editor = view.current;
    const range = insertion.current;
    if (!candidate || !range || !editor || locked) return;
    const selected = editor.state.sliceDoc(range.from, range.to);
    if (picker?.initialQuery !== undefined && selected !== "[[" + picker.initialQuery) {
      closePicker();
      return;
    }
    const text = picker?.initialQuery === undefined ? selected : "";
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
      data-sizing={sizing}
      data-invalid={invalid || undefined}
    >
      <ConfigProvider
        theme={{
          token: { motion: false },
          components: {
            Segmented: {
              itemSelectedBg: "var(--panel-elevated)",
              itemSelectedColor: "var(--text)",
              trackBg: "var(--panel)",
            },
          },
        }}
      >
        <div className="markdown-editor__toolbar">
          <div className="markdown-editor__mode">
            <Segmented
              size="small"
              aria-label="Document editor mode"
              options={["Write", "Preview"]}
              value={mode}
              onChange={setMode}
            />
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
                open={tabActive && headingOpen}
                onOpenChange={setHeadingOpen}
                trigger={["click"]}
                disabled={locked}
                menu={{
                  items: headingItems,
                  onClick: ({ key }) => {
                    setHeadingOpen(false);
                    run((editor) => setHeading(editor, Number(key)));
                  },
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
              <FormatButton
                label="Record link"
                icon={<Link2 size={16} />}
                disabled={locked}
                onClick={() => {
                  const selected = view.current?.state.selection.main;
                  if (selected) openPicker({ from: selected.from, to: selected.to });
                }}
              />
            </div>
          </div>
        </div>
      </ConfigProvider>
      <div className="markdown-editor__body">
        <div
          className="markdown-editor__source"
          ref={source}
          onFocus={resumeInline}
          onPointerUp={resumeInline}
          inert={!writing}
          aria-hidden={!writing || undefined}
        />
        {!writing && (
          <div
            className="markdown-editor__preview"
            role="region"
            aria-label={sizing === "fill" ? "Note preview" : "Markdown preview"}
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
      {inline && writing && !locked && tabActive && (
        <InlineRecordPicker
          editor={inline.editor}
          query={inline.query}
          onDismiss={dismissInline}
          onBlur={() => setInline(null)}
          onSearch={() => {
            if (inlineQuery.current === inline.query)
              openPicker(inline.query, inline.query.text);
          }}
          onChoose={(record) => {
            if (inlineQuery.current !== inline.query) return;
            dismissInline();
            insertRecordLink(
              inline.editor,
              inline.query,
              markdownRecordLink(record.title, record.record_key),
            );
            inline.editor.focus();
          }}
        />
      )}
      {picker && (
        <SearchPickerModal
          initialQuery={picker.initialQuery}
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
        />
      )}
    </div>
  );
}
