import type { EditorView, ViewUpdate } from "@codemirror/view";
import { useQuery } from "@tanstack/react-query";
import { Button, Popover } from "antd";
import { Search } from "lucide-react";
import {
  useEffect,
  useEffectEvent,
  useId,
  useLayoutEffect,
  useRef,
  useState,
  type ComponentRef,
} from "react";
import { searchPage } from "../../api/atlasApi";
import type { RecordSummaryView } from "../../generated/atlas";
import {
  isArtifactGenerationChanged,
  referenceQueryMeta,
} from "../queries/referenceQueries";
import { useReferenceQueryRetirement } from "../queries/useReferenceQueryRetirement";
import { assertKeywordSearch, assertSearchIdentity } from "../search/searchIdentity";
import { useReferenceGeneration } from "../search/useReferenceGeneration";

export type InlineRecordQuery = {
  from: number;
  to: number;
  head: number;
  text: string;
};

export function readInlineRecordQuery(
  update: ViewUpdate,
  previous: InlineRecordQuery | null,
): InlineRecordQuery | null {
  const { state, transactions } = update;
  const caret = state.selection.main;
  if (
    !caret.empty ||
    state.readOnly ||
    update.view.composing ||
    transactions.some((tr) => tr.isUserEvent("input.type.compose"))
  )
    return null;
  if (previous) {
    const from = update.changes.mapPos(previous.from, -1);
    const to = update.changes.mapPos(previous.to, 1);
    const token = state.sliceDoc(from, to);
    if (
      caret.head < from + 2 ||
      caret.head > to ||
      !token.startsWith("[[") ||
      /[\n\]]/.test(token) ||
      transactions.some((tr) => tr.isUserEvent("undo") || tr.isUserEvent("redo"))
    )
      return null;
    const text = token.slice(2);
    return from === previous.from &&
      to === previous.to &&
      caret.head === previous.head &&
      text === previous.text
      ? previous
      : { from, to, head: caret.head, text };
  }
  if (
    !update.docChanged ||
    state.doc.length !== update.startState.doc.length + 1 ||
    caret.head < 2 ||
    !transactions.some((tr) => tr.isUserEvent("input.type")) ||
    state.sliceDoc(caret.head - 2, caret.head) !== "[["
  )
    return null;
  return { from: caret.head - 2, to: caret.head, head: caret.head, text: "" };
}

export function InlineRecordPicker({
  editor,
  query,
  onChoose,
  onSearch,
  onDismiss,
}: {
  editor: EditorView;
  query: InlineRecordQuery;
  onChoose: (record: RecordSummaryView) => void;
  onSearch: () => void;
  onDismiss: () => void;
}) {
  const listId = useId();
  const owner = editor.contentDOM.ownerDocument;
  const container =
    editor.dom.closest<HTMLElement>('[role="dialog"], [role="tabpanel"]') ?? owner.body;
  const popover = useRef<ComponentRef<typeof Popover>>(null);
  const anchor = useRef<HTMLSpanElement>(null);
  const content = useRef<HTMLDivElement>(null);
  const [position, setPosition] = useState<{ left: number; top: number } | null>(null);
  useLayoutEffect(() => {
    const element = anchor.current;
    const parent = element?.offsetParent;
    if (!element || !(parent instanceof HTMLElement)) return;
    let active = true;
    const measure = {
      key: element,
      read(view: EditorView) {
        const start = view.coordsAtPos(view.state.selection.main.head);
        if (!start) return null;
        const frame = parent.getBoundingClientRect();
        return {
          left: start.left - frame.left - parent.clientLeft + parent.scrollLeft,
          top: start.bottom - frame.top - parent.clientTop + parent.scrollTop,
        };
      },
      write(next: typeof position) {
        if (active)
          setPosition((current) =>
            current?.left === next?.left && current?.top === next?.top ? current : next,
          );
      },
    };
    const measurePosition = () => editor.requestMeasure(measure);
    measurePosition();
    const observer = new ResizeObserver(measurePosition);
    observer.observe(editor.dom);
    owner.addEventListener("scroll", measurePosition, true);
    const viewport = owner.defaultView?.visualViewport;
    viewport?.addEventListener("resize", measurePosition);
    viewport?.addEventListener("scroll", measurePosition);
    return () => {
      active = false;
      observer.disconnect();
      owner.removeEventListener("scroll", measurePosition, true);
      viewport?.removeEventListener("resize", measurePosition);
      viewport?.removeEventListener("scroll", measurePosition);
    };
  }, [editor, owner, query]);

  useLayoutEffect(() => {
    popover.current?.forceAlign();
  }, [position]);

  const [debounced, setDebounced] = useState("");
  useEffect(() => {
    const timer = setTimeout(() => setDebounced(query.text), 140);
    return () => clearTimeout(timer);
  }, [query.text]);
  const eligible = Array.from(query.text.trim()).length >= 2;
  const ready = eligible && debounced === query.text;
  const retrieval = useReferenceGeneration(query.text, ready);
  const keyForGeneration = (generation: string | null) => [
    "inline-record-links",
    generation ?? listId,
    query.text,
  ];
  const [changedGeneration, setChangedGeneration] = useState(false);
  const resultKey = keyForGeneration(retrieval.generation);
  const result = useQuery({
    queryKey: resultKey,
    meta: referenceQueryMeta,
    enabled: ready && !changedGeneration,
    staleTime: 20_000,
    retry: false,
    queryFn: async ({ signal }) => {
      const ticket = retrieval.capture(signal);
      const page = await searchPage(
        {
          criteria: {
            kind: "text_search",
            mode: "lexical",
            query: query.text,
            filter: null,
            lexical: { fields: "record_names", match: "last_token_prefix" },
          },
          page: { number: 1, size: 5 },
          result_detail: "records",
          expected_artifact_generation: retrieval.generation,
        },
        signal,
      );
      assertKeywordSearch(page);
      assertSearchIdentity(page, undefined, retrieval.generation);
      retrieval.adopt(ticket, page.artifact_generation, {
        queryKey: keyForGeneration(page.artifact_generation),
        data: page,
      });
      return page;
    },
  });
  useReferenceQueryRetirement(resultKey);
  if (!changedGeneration && isArtifactGenerationChanged(result.error))
    setChangedGeneration(true);
  const rows =
    ready &&
    !changedGeneration &&
    !result.isFetching &&
    result.isSuccess &&
    result.data.artifact_generation === retrieval.generation
      ? result.data.rows
      : [];
  const [choice, setChoice] = useState<{ text: string; key: string | null }>({
    text: "",
    key: null,
  });
  const selected =
    choice.text === query.text && choice.key === "full"
      ? rows.length
      : Math.max(
          rows.length ? 0 : -1,
          rows.findIndex(
            ({ record }) =>
              choice.text === query.text && record.record_key === choice.key,
          ),
        );
  const at = (index: number) =>
    setChoice({
      text: query.text,
      key: index === rows.length ? "full" : rows[index].record.record_key,
    });
  const activate = (index: number) => {
    if (index === rows.length) onSearch();
    else if (rows[index]) onChoose(rows[index].record);
  };
  const dismissOutside = useEffectEvent((event: Event) => {
    if (
      event.target instanceof Node &&
      !editor.contentDOM.contains(event.target) &&
      !content.current?.contains(event.target)
    )
      onDismiss();
  });
  const handleKey = useEffectEvent((event: KeyboardEvent) => {
    if (
      event.isComposing ||
      event.keyCode === 229 ||
      !(event.target instanceof Node) ||
      (!editor.contentDOM.contains(event.target) &&
        !content.current?.contains(event.target))
    )
      return;
    if (event.key === "Escape") {
      onDismiss();
      editor.focus();
    } else if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      if (event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return;
      const count = rows.length + 1;
      at(
        event.key === "ArrowDown"
          ? (selected + 1) % count
          : selected < 0
            ? count - 1
            : (selected - 1 + count) % count,
      );
    } else if (
      event.key === "Enter" &&
      selected >= 0 &&
      !event.shiftKey &&
      !event.ctrlKey &&
      !event.metaKey &&
      !event.altKey
    ) {
      activate(selected);
    } else return;
    event.preventDefault();
    event.stopPropagation();
  });
  const dismiss = useEffectEvent(onDismiss);
  useEffect(() => {
    let popupPointer = false;
    const pointerDown = (event: PointerEvent) => {
      popupPointer =
        event.target instanceof Node && !!content.current?.contains(event.target);
      dismissOutside(event);
    };
    const clearPointer = () => {
      popupPointer = false;
    };
    const focusOut = (event: FocusEvent) => {
      // Tapping an option may blur the editable before the synthesized click.
      if (!popupPointer && !event.relatedTarget) dismiss();
    };
    owner.addEventListener("pointerdown", pointerDown, true);
    owner.addEventListener("pointercancel", clearPointer, true);
    owner.addEventListener("focusin", dismissOutside, true);
    owner.addEventListener("keydown", handleKey, true);
    editor.contentDOM.addEventListener("focusout", focusOut);
    owner.defaultView?.addEventListener("blur", dismiss);
    return () => {
      owner.removeEventListener("pointerdown", pointerDown, true);
      owner.removeEventListener("pointercancel", clearPointer, true);
      owner.removeEventListener("focusin", dismissOutside, true);
      owner.removeEventListener("keydown", handleKey, true);
      editor.contentDOM.removeEventListener("focusout", focusOut);
      owner.defaultView?.removeEventListener("blur", dismiss);
    };
  }, [owner, editor]);
  useLayoutEffect(() => {
    const textbox = editor.contentDOM;
    textbox.setAttribute("aria-autocomplete", "list");
    textbox.setAttribute("aria-controls", listId);
    if (selected >= 0) {
      const id = listId + "-" + selected;
      textbox.setAttribute("aria-activedescendant", id);
      const item = owner.getElementById(id);
      const scroller = item?.closest(".markdown-record-options");
      if (item && scroller) {
        const rect = item.getBoundingClientRect();
        const bounds = scroller.getBoundingClientRect();
        if (rect.top < bounds.top) scroller.scrollTop += rect.top - bounds.top;
        else if (rect.bottom > bounds.bottom)
          scroller.scrollTop += rect.bottom - bounds.bottom;
      }
    }
    return () => {
      textbox.removeAttribute("aria-autocomplete");
      textbox.removeAttribute("aria-controls");
      textbox.removeAttribute("aria-activedescendant");
    };
  }, [editor, listId, owner, selected, position]);

  let status: string;
  if (changedGeneration)
    status = "The reference collection changed. Open full search to continue.";
  else if (!eligible) status = "Type at least two characters.";
  else if (ready && result.isError) status = "Matching records could not load.";
  else if (!ready || result.isFetching) status = "Searching…";
  else
    status = rows.length ? `${rows.length} matching records` : "No matching records.";
  return (
    <Popover
      ref={popover}
      open={position !== null}
      trigger={[]}
      placement="bottomLeft"
      arrow={false}
      getPopupContainer={() => container}
      styles={{ container: { padding: 0, overflow: "hidden" } }}
      content={
        <div ref={content} className="markdown-record-picker">
          <div
            role="status"
            className={rows.length ? "sr-only" : "markdown-record-status"}
          >
            {status}
          </div>
          <div role="listbox" id={listId} aria-label="Record link suggestions">
            <div className="markdown-record-options">
              {rows.map(({ record }, index) => (
                <Button
                  key={record.record_key}
                  type="text"
                  role="option"
                  tabIndex={-1}
                  id={listId + "-" + index}
                  aria-selected={selected === index}
                  onMouseDown={(event) => event.preventDefault()}
                  onClick={() => activate(index)}
                >
                  <span className="markdown-record-label">
                    <strong>{record.title}</strong>
                    <span>
                      {[
                        record.kind_label,
                        record.source_title,
                        record.publication ?? record.pack,
                      ]
                        .filter(Boolean)
                        .join(" · ")}
                    </span>
                  </span>
                </Button>
              ))}
            </div>
            <Button
              type="text"
              role="option"
              tabIndex={-1}
              icon={<Search size={14} />}
              className="markdown-record-full"
              id={listId + "-" + rows.length}
              aria-selected={selected === rows.length}
              onMouseDown={(event) => event.preventDefault()}
              onClick={onSearch}
            >
              Open full search
            </Button>
          </div>
        </div>
      }
    >
      <span
        ref={anchor}
        aria-hidden="true"
        className="markdown-record-anchor"
        style={position ?? undefined}
      />
    </Popover>
  );
}
