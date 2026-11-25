import { AtlasModal } from "../ui/overlays/AtlasModal";
import { EditorView } from "@codemirror/view";
import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { Form } from "antd";
import { useState } from "react";
import {
  markdownSource,
  replaceMarkdown,
  selectMarkdown,
} from "../../test/markdownEditor";
import * as atlasApi from "../../api/atlasApi";
import type { SearchResultsPage } from "../../generated/atlas";
import { summaryFixture } from "../../test/fixtures";
import { MarkdownEditor } from "./MarkdownEditor";
import { markdownRecordLink } from "./MarkdownDocument";

const note = () => screen.getByRole("textbox", { name: "Note" });
const action = (name: string) => screen.getByRole("button", { name });
const mode = (name: string) => fireEvent.click(screen.getByRole("radio", { name }));

it("previews the same draft as styled Markdown and retains formatting history", () => {
  const source = `## Trigger\n\n**Resolve** ${markdownRecordLink("Harm [Rank 2]", "spells:harm")}`;
  expect(source).toContain("record:spells:harm");
  function Draft() {
    const [value, setValue] = useState("");
    return <MarkdownEditor label="Note" value={value} onChange={setValue} />;
  }
  render(
    <QueryClientProvider client={new QueryClient()}>
      <Draft />
    </QueryClientProvider>,
  );
  const textbox = note();
  replaceMarkdown(textbox, source);
  selectMarkdown(textbox, 4);
  fireEvent.click(action("Heading"));
  fireEvent.click(screen.getByRole("menuitem", { name: "Heading 3" }));
  mode("Preview");
  expect(screen.getByRole("heading", { name: "Trigger", level: 3 })).toBeVisible();
  expect(screen.getByRole("link", { name: "Harm [Rank 2]" })).toBeVisible();
  expect(screen.queryByRole("textbox", { name: "Note" })).toBeNull();
  mode("Write");
  expect(note()).toBe(textbox);
  expect(markdownSource(textbox)).toBe(source.replace("## Trigger", "### Trigger"));
  fireEvent.click(action("Undo"));
  expect(markdownSource(textbox)).toBe(source);
  fireEvent.click(action("Redo"));
  expect(markdownSource(textbox)).toBe(source.replace("## Trigger", "### Trigger"));
});

it("shows active emphasis and toggles the requested style inside nested delimiters", () => {
  const source = "***Both*** and _italic_ and plain";
  const change = vi.fn();
  render(<MarkdownEditor label="Note" value={source} onChange={change} />);

  selectMarkdown(note(), 4);
  expect(action("Bold")).toHaveAttribute("aria-pressed", "true");
  expect(action("Italic")).toHaveAttribute("aria-pressed", "true");
  fireEvent.click(action("Bold"));
  expect(change).toHaveBeenLastCalledWith("*Both* and _italic_ and plain");
  expect(action("Bold")).toHaveAttribute("aria-pressed", "false");

  const next = markdownSource(note());
  selectMarkdown(
    note(),
    next.indexOf("italic"),
    next.indexOf("italic") + "italic".length,
  );
  expect(action("Italic")).toHaveAttribute("aria-pressed", "true");
  fireEvent.click(action("Italic"));
  expect(change).toHaveBeenLastCalledWith("*Both* and italic and plain");
  selectMarkdown(
    note(),
    markdownSource(note()).indexOf("plain"),
    markdownSource(note()).length,
  );
  fireEvent.click(action("Italic"));
  expect(change).toHaveBeenLastCalledWith("*Both* and italic and *plain*");
});

it.each([
  ["Bold", "**"],
  ["Italic", "*"],
])("%s handles whitespace and protects selections across blocks", (name, marker) => {
  const change = vi.fn();
  const source = " HARM \n\nNext stage";
  render(<MarkdownEditor label="Note" value={source} onChange={change} />);
  selectMarkdown(note(), 0, source.length);
  expect(action(name)).toBeDisabled();
  selectMarkdown(note(), 0, 6);
  expect(action(name)).toBeEnabled();
  fireEvent.click(action(name));
  expect(change).toHaveBeenLastCalledWith(` ${marker}HARM${marker} \n\nNext stage`);
});

it("switches selected list types, preserving nesting, and undoes each command", () => {
  const source = "- First\n  - Nested\n- Second";
  render(<MarkdownEditor label="Note" value={source} />);
  const textbox = note();
  expect(action("Undo")).toBeDisabled();
  expect(action("Redo")).toBeDisabled();
  selectMarkdown(textbox, 0, source.length);
  fireEvent.click(action("Numbered list"));
  expect(markdownSource(textbox)).toBe("1. First\n   1. Nested\n2. Second");
  fireEvent.click(action("Bulleted list"));
  expect(markdownSource(textbox)).toBe(source);
  fireEvent.click(action("Undo"));
  expect(markdownSource(textbox)).toBe("1. First\n   1. Nested\n2. Second");
  expect(action("Redo")).toBeEnabled();
  fireEvent.click(action("Undo"));
  expect(markdownSource(textbox)).toBe(source);
  expect(action("Undo")).toBeDisabled();
});

it("preserves the child indent when switching wider numbered markers to bullets", () => {
  const source = "1. First\n      1. Nested\n2. Second";
  render(<MarkdownEditor label="Note" value={source} />);
  const textbox = note();
  selectMarkdown(textbox, 0, source.length);
  fireEvent.click(action("Bulleted list"));
  expect(markdownSource(textbox)).toBe("- First\n     - Nested\n- Second");
});

it("retains history through read-only transitions and resets it on external replacement", () => {
  const change = vi.fn();
  const editor = render(
    <MarkdownEditor label="Note" value="Saved" onChange={change} />,
  );
  replaceMarkdown(note(), "Edited");
  expect(change).toHaveBeenCalledExactlyOnceWith("Edited");
  editor.rerender(
    <MarkdownEditor label="Note" value="Edited" onChange={change} readOnly />,
  );
  expect(note()).toHaveAttribute("aria-readonly", "true");
  expect(note()).toHaveAttribute("tabindex", "0");
  expect(action("Undo")).toBeDisabled();
  fireEvent.keyDown(note(), { key: "z", ctrlKey: true });
  fireEvent.keyDown(note(), { key: "Enter" });
  expect(markdownSource(note())).toBe("Edited");
  editor.rerender(<MarkdownEditor label="Note" value="Edited" onChange={change} />);
  expect(action("Undo")).toBeEnabled();
  editor.rerender(
    <MarkdownEditor label="Note" value="Replacement" onChange={change} />,
  );
  expect(markdownSource(note())).toBe("Replacement");
  expect(action("Undo")).toBeDisabled();
  expect(action("Redo")).toBeDisabled();
  expect(change).toHaveBeenCalledTimes(1);
});

it("connects the source textbox to form validation and help", async () => {
  render(
    <Form initialValues={{ note: "" }}>
      <Form.Item
        name="note"
        label="Note"
        extra="Encounter reminder"
        rules={[{ required: true }]}
      >
        <MarkdownEditor label="Note" />
      </Form.Item>
      <button type="submit">Save</button>
    </Form>,
  );
  expect(note()).toHaveAccessibleDescription("Encounter reminder");
  expect(note()).toHaveAttribute("aria-required", "true");
  fireEvent.click(screen.getByText("Note", { selector: "label" }));
  expect(note()).toHaveFocus();
  fireEvent.click(action("Save"));
  await waitFor(() => expect(note()).toHaveAttribute("aria-invalid", "true"));
  await waitFor(() => expect(note()).toHaveAccessibleDescription(/note.*required/i));
});

function insertAtCaret(editor: EditorView, text: string, userEvent: string) {
  const { from, to } = editor.state.selection.main;
  act(() =>
    editor.dispatch({
      changes: { from, to, insert: text },
      selection: { anchor: from + text.length },
      userEvent,
    }),
  );
}

function inlinePage(query: string, title: string): SearchResultsPage {
  return {
    search_content: "all",
    sort: { kind: "relevance" },
    ranking: "score",
    search_profile: "bge-base",
    artifact_generation: "reference-a",
    result_detail: "records" as const,
    relationship_clauses: [],
    coverage: null,
    mode: { kind: "text_search", mode: "lexical", query },
    page: {
      number: 1,
      size: 5,
      count: 1,
      total: 1,
      has_more: false,
      next_page: null,
    },
    rows: [
      {
        record: summaryFixture("spells:exact-key", title),
        matches: [],
        relationship_evidence: [],
      },
    ],
  };
}

it("inserts only the current inline result between surrounding prose and restores the typed range on Undo", async () => {
  const pending = new Map<string, (page: SearchResultsPage) => void>();
  const search = vi.spyOn(atlasApi, "searchPage").mockImplementation(
    (request) =>
      new Promise((resolve) => {
        if (request.criteria.kind === "text_search")
          pending.set(request.criteria.query, resolve);
      }),
  );
  const client = new QueryClient();
  function Draft() {
    const [source, setSource] = useState("Before  after");
    return <MarkdownEditor label="Note" value={source} onChange={setSource} />;
  }
  const mounted = render(
    <QueryClientProvider client={client}>
      <Draft />
    </QueryClientProvider>,
  );
  const textbox = note();
  const editor = EditorView.findFromDOM(textbox)!;
  try {
    selectMarkdown(textbox, 7);
    insertAtCaret(editor, "[", "input.type");
    insertAtCaret(editor, "[", "input.type");
    fireEvent.focusOut(textbox, { relatedTarget: null });
    expect(textbox).not.toHaveAttribute("aria-controls");
    fireEvent.focusIn(textbox);
    expect(textbox).toHaveAttribute("aria-controls");
    insertAtCaret(editor, "fi", "input.type");
    await waitFor(() => expect(pending.has("fi")).toBe(true));
    const oldSignal = search.mock.calls[0][1]!;
    insertAtCaret(editor, "re ", "input.type");
    await waitFor(() => expect(pending.has("fire ")).toBe(true));
    expect(oldSignal.aborted).toBe(true);
    expect(search.mock.lastCall?.[0]).toMatchObject({
      criteria: {
        mode: "lexical",
        query: "fire ",
        lexical: { fields: "record_names", match: "last_token_prefix" },
      },
      result_detail: "records",
      page: { number: 1, size: 5 },
    });
    await act(async () => pending.get("fire ")!(inlinePage("fire ", "Fire [Ward]")));
    await waitFor(() =>
      expect(textbox).toHaveAttribute(
        "aria-activedescendant",
        expect.stringMatching(/-0$/),
      ),
    );
    await act(async () => pending.get("fi")!(inlinePage("fi", "Wrong result")));
    fireEvent.keyDown(textbox, { key: "Enter" });
    expect(markdownSource(textbox)).toBe(
      "Before " + markdownRecordLink("Fire [Ward]", "spells:exact-key") + " after",
    );
    fireEvent.click(action("Undo"));
    expect(markdownSource(textbox)).toBe("Before [[fire  after");
    expect(textbox).not.toHaveAttribute("aria-controls");
    selectMarkdown(textbox, 7, 14);
    insertAtCaret(editor, "[[", "input.paste");
    expect(textbox).not.toHaveAttribute("aria-controls");
  } finally {
    mounted.unmount();
    client.clear();
    search.mockRestore();
  }
});

it("keeps an earlier cached suggestion unavailable after the collection changes", async () => {
  const search = vi
    .spyOn(atlasApi, "searchPage")
    .mockResolvedValueOnce(inlinePage("fi", "Fire Ward"))
    .mockResolvedValueOnce({
      ...inlinePage("fir", "New Fire Ward"),
      artifact_generation: "reference-b",
    });
  const client = new QueryClient();
  const mounted = render(
    <QueryClientProvider client={client}>
      <MarkdownEditor label="Note" value="" />
    </QueryClientProvider>,
  );
  try {
    const textbox = note();
    const editor = EditorView.findFromDOM(textbox)!;
    insertAtCaret(editor, "[", "input.type");
    insertAtCaret(editor, "[", "input.type");
    insertAtCaret(editor, "fi", "input.type");
    await waitFor(() => expect(textbox).toHaveAttribute("aria-activedescendant"));
    insertAtCaret(editor, "r", "input.type");
    await waitFor(() =>
      expect(
        client
          .getQueryCache()
          .findAll()
          .some((query) => query.state.error instanceof atlasApi.AtlasApiError),
      ).toBe(true),
    );
    vi.useFakeTimers();
    try {
      act(() =>
        editor.dispatch({
          changes: { from: 4, to: 5 },
          selection: { anchor: 4 },
          userEvent: "delete.backward",
        }),
      );
      await act(async () => {
        await vi.advanceTimersByTimeAsync(300);
      });
      expect(markdownSource(textbox)).toBe("[[fi");
      expect(textbox).toHaveAttribute("aria-controls");
      expect(textbox).not.toHaveAttribute("aria-activedescendant");
      expect(search).toHaveBeenCalledTimes(2);
    } finally {
      vi.useRealTimers();
    }
    fireEvent.focusOut(textbox, { relatedTarget: null });
    expect(textbox).not.toHaveAttribute("aria-controls");
  } finally {
    mounted.unmount();
    client.clear();
    search.mockRestore();
  }
});

it("expands each bound field in the same form and preserves its edit session", () => {
  const save = vi.fn();
  render(
    <AtlasModal open title="Settings">
      <Form onFinish={save}>
        <Form.Item name="first" label="First">
          <MarkdownEditor label="First" />
        </Form.Item>
        <section>
          <Form.Item name="second" label="Second">
            <MarkdownEditor label="Second" />
          </Form.Item>
        </section>
      </Form>
    </AtlasModal>,
  );
  for (const label of ["First", "Second"]) {
    const box = screen.getByRole("textbox", { name: label });
    replaceMarkdown(box, "# Ambush");
    const expand = within(box.closest(".markdown-editor") as HTMLElement).getByRole(
      "button",
      { name: "Expand editor" },
    );
    fireEvent.click(expand);
    const dialog = screen.getByRole("dialog", { name: label });
    expect(within(dialog).getByRole("textbox", { name: label })).toBe(box);
    fireEvent.submit(box.closest("form")!);
    const collapse = within(dialog).getByRole("button", { name: "Collapse editor" });
    expect(collapse).toHaveFocus();
    fireEvent.click(collapse);
    expect(expand).toHaveFocus();
    expect(markdownSource(box)).toBe("# Ambush");
    expect(save).not.toHaveBeenCalled();
  }
});
