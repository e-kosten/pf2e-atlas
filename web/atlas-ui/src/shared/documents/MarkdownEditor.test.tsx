import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { Form } from "antd";
import { useState } from "react";
import {
  markdownSource,
  replaceMarkdown,
  selectMarkdown,
} from "../../test/markdownEditor";
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
