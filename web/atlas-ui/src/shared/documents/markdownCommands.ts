import { isolateHistory } from "@codemirror/commands";
import {
  countColumn,
  EditorSelection,
  type ChangeSpec,
  type EditorState,
  type Text,
} from "@codemirror/state";
import { type EditorView } from "@codemirror/view";
import { fromMarkdown } from "mdast-util-from-markdown";

type Emphasis = "strong" | "emphasis";
type Range = { from: number; to: number };
type MarkdownNode = {
  type: string;
  position?: { start: { offset?: number }; end: { offset?: number } };
  children?: MarkdownNode[];
};

const parsedEmphasis = new WeakMap<Text, Record<Emphasis, Range[]>>();

function emphasisRanges(doc: Text) {
  const cached = parsedEmphasis.get(doc);
  if (cached) return cached;

  const ranges: Record<Emphasis, Range[]> = { strong: [], emphasis: [] };
  function visit({ type, position, children }: MarkdownNode) {
    if (type === "strong" || type === "emphasis") {
      const from = position?.start.offset;
      const to = position?.end.offset;
      if (from != null && to != null) ranges[type].push({ from, to });
    }
    children?.forEach(visit);
  }
  visit(fromMarkdown(doc.toString()));
  parsedEmphasis.set(doc, ranges);
  return ranges;
}

export function emphasisState(state: EditorState, kind: Emphasis) {
  let { from, to } = state.selection.main;
  if (from !== to) {
    const selected = state.doc.sliceString(from, to);
    const content = selected.trim();
    if (!content) return { formatted: undefined, disabled: true, from, to };
    from += selected.length - selected.trimStart().length;
    to = from + content.length;
  }

  const ranges = emphasisRanges(state.doc)[kind];
  const formatted = ranges.find((range) =>
    from === to
      ? range.from < from && from < range.to
      : range.from <= from && to <= range.to,
  );
  if (formatted) return { formatted, disabled: false, from, to };

  if (from === to) {
    // Empty delimiter pairs have no CommonMark node until the author types.
    const line = state.doc.lineAt(from);
    const left = line.text.slice(0, from - line.from).match(/\*+$/)?.[0].length;
    const right = line.text.slice(from - line.from).match(/^\*+/)?.[0].length;
    const length = kind === "strong" ? 2 : 1;
    if (left === length && right === length) {
      return {
        formatted: { from: from - length, to: to + length },
        disabled: false,
        from,
        to,
      };
    }
  }

  return {
    formatted: undefined,
    disabled:
      state.doc.lineAt(from).number !== state.doc.lineAt(to).number ||
      ranges.some((range) => from < range.to && to > range.from),
    from,
    to,
  };
}

function format(view: EditorView, specs: ChangeSpec, selection?: EditorSelection) {
  const changes = view.state.changes(specs);
  const current = view.state.selection.main;
  view.dispatch({
    changes,
    selection:
      selection ??
      EditorSelection.single(
        changes.mapPos(current.anchor, 1),
        changes.mapPos(current.head, 1),
      ),
    annotations: isolateHistory.of("full"),
    userEvent: "input.format",
    scrollIntoView: true,
  });
}

export function toggleEmphasis(view: EditorView, kind: Emphasis) {
  if (view.state.readOnly) return false;
  const { formatted, disabled, from, to } = emphasisState(view.state, kind);
  if (disabled) return true;

  const marker = kind === "strong" ? "**" : "*";
  if (formatted) {
    format(view, [
      { from: formatted.from, to: formatted.from + marker.length },
      { from: formatted.to - marker.length, to: formatted.to },
    ]);
    return true;
  }

  const { anchor, head } = view.state.selection.main;
  const offset = (pos: number) =>
    pos < from ? pos : pos <= to ? pos + marker.length : pos + 2 * marker.length;
  format(
    view,
    { from, to, insert: marker + view.state.doc.sliceString(from, to) + marker },
    EditorSelection.single(offset(anchor), offset(head)),
  );
  return true;
}

export function setHeading(view: EditorView, level: number) {
  if (view.state.readOnly) return;
  const line = view.state.doc.lineAt(view.state.selection.main.from);
  const current = /^ {0,3}#{1,6}(?:[ \t]+|$)/.exec(line.text)?.[0] ?? "";
  const marker = "#".repeat(level) + " ";
  if (current !== marker) {
    format(view, { from: line.from, to: line.from + current.length, insert: marker });
  }
}

const listLine = /^([ \t]*)(?:(\d{1,9}[.)]|[-+*])(?:[ \t]+|$))?/;

export function toggleList(view: EditorView, ordered: boolean) {
  if (view.state.readOnly) return;
  const { doc, selection } = view.state;
  const { from, to } = selection.main;
  const first = doc.lineAt(from);
  const last = doc.lineAt(from === to ? to : to - 1);
  const items = [];
  for (let number = first.number; number <= last.number; number += 1) {
    const line = doc.line(number);
    if (first.number !== last.number && !line.text.trim()) continue;
    const [prefix, indent, marker] = listLine.exec(line.text)!;
    items.push({ line, prefix, indent, marker });
  }

  if (!items.length) return;
  const remove = items.every(({ marker }) => marker && ordered === /^\d/.test(marker));
  const levels = new Map<number, { number: number; before: number; after: number }>();
  const changes: ChangeSpec[] = [];
  const tabSize = view.state.tabSize;
  for (const { line, prefix, indent, marker } of items) {
    const column = countColumn(indent, tabSize);
    let adjustedColumn = column;
    for (const [depth, parent] of levels) {
      if (depth > column) levels.delete(depth);
      if (depth < column && parent.before <= column) {
        adjustedColumn = column + parent.after - parent.before;
      }
    }

    const number = (levels.get(column)?.number ?? 0) + 1;
    const listMarker = ordered ? `${number}. ` : "- ";
    // Keep indent relative to the parent content when its marker width changes.
    const nextIndent = adjustedColumn === column ? indent : " ".repeat(adjustedColumn);
    const nextPrefix = remove ? indent : nextIndent + listMarker;
    levels.set(column, {
      number,
      before: marker ? countColumn(prefix, tabSize) : column + 1,
      after: adjustedColumn + listMarker.length,
    });
    if (prefix !== nextPrefix) {
      changes.push({
        from: line.from,
        to: line.from + prefix.length,
        insert: nextPrefix,
      });
    }
  }

  if (changes.length) format(view, changes);
}

export function insertRecordLink(view: EditorView, range: Range, link: string) {
  if (view.state.readOnly) return;
  format(
    view,
    { ...range, insert: link },
    EditorSelection.single(range.from + link.length),
  );
}
