import { fireEvent, render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useState } from "react";
import { MarkdownEditor } from "./MarkdownEditor";
import { markdownRecordLink } from "./MarkdownDocument";

it("previews the same draft as styled Markdown and returns to the saved source", () => {
  const source = `## Trigger\n\n**Resolve** ${markdownRecordLink("Harm [Rank 2]", "spells:harm")}`;
  expect(source).toContain("atlas-record:spells:harm");
  function Draft() {
    const [value, setValue] = useState("");
    return <MarkdownEditor label="Note" value={value} onChange={setValue} />;
  }
  render(
    <QueryClientProvider client={new QueryClient()}>
      <Draft />
    </QueryClientProvider>,
  );
  const note = screen.getByRole<HTMLTextAreaElement>("textbox", { name: "Note" });
  fireEvent.change(note, { target: { value: source } });
  note.setSelectionRange(4, 4);
  fireEvent.click(screen.getByRole("button", { name: "Heading" }));
  fireEvent.click(screen.getByRole("menuitem", { name: "Heading 3" }));
  fireEvent.click(screen.getByText("Preview"));
  expect(screen.getByRole("heading", { name: "Trigger", level: 3 })).toBeVisible();
  expect(screen.getByRole("link", { name: "Harm [Rank 2]" })).toBeVisible();
  fireEvent.click(screen.getByText("Write"));
  expect(note).toHaveValue(source.replace("## Trigger", "### Trigger"));
});

it("disables matching emphasis at the selection and inserts italic on plain text", () => {
  const source = "**Bold** and _italic_ and plain";
  const change = vi.fn();
  render(<MarkdownEditor label="Note" value={source} onChange={change} />);
  const note = screen.getByRole<HTMLTextAreaElement>("textbox", { name: "Note" });
  const bold = screen.getByRole("button", { name: "Bold" });
  const italic = screen.getByRole("button", { name: "Italic" });
  note.focus();

  function select(start: number, end = start) {
    note.setSelectionRange(start, end);
    fireEvent.select(note);
  }

  select(3);
  expect(bold).toBeDisabled();
  expect(italic).toBeEnabled();
  select(source.indexOf("italic"), source.indexOf("italic") + "italic".length);
  expect(italic).toBeDisabled();
  expect(bold).toBeEnabled();
  select(source.indexOf("plain"), source.length);
  expect(italic).toBeEnabled();
  fireEvent.click(italic);
  expect(change).toHaveBeenCalledWith("**Bold** and _italic_ and *plain*");
});
