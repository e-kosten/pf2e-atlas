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
