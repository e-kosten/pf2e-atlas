import { EditorView } from "@codemirror/view";
import { act } from "@testing-library/react";

function sourceEditor(textbox: HTMLElement) {
  return EditorView.findFromDOM(textbox.parentElement!)!;
}

export function markdownSource(textbox: HTMLElement) {
  return sourceEditor(textbox).state.sliceDoc();
}

// Use source transactions in jsdom. Real contenteditable input is covered in Playwright.
export function replaceMarkdown(textbox: HTMLElement, insert: string) {
  const editor = sourceEditor(textbox);
  act(() =>
    editor.dispatch({
      changes: { from: 0, to: editor.state.doc.length, insert },
      selection: { anchor: editor.state.toText(insert).length },
      userEvent: "input",
    }),
  );
}

export function selectMarkdown(textbox: HTMLElement, anchor: number, head = anchor) {
  act(() => sourceEditor(textbox).dispatch({ selection: { anchor, head } }));
}
