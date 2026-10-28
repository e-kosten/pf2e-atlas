import { fireEvent, render, screen } from "@testing-library/react";
import { expect, it, vi } from "vitest";
import type { EmbeddedRecordLinkView } from "../../generated/atlas";
import { JournalContents } from "./JournalContents";
import { RecordPresentation } from "./RecordPresentation";
import { knownFact, summaryFixture, unavailableFact } from "../../test/fixtures";
import type { RecordPresentationView } from "../../generated/atlas";

function journal(recordKey: string, count: number): RecordPresentationView {
  return {
    identity: summaryFixture(recordKey, recordKey),
    classification: "Journal",
    content: [],
    embedded: [],
    body: {
      kind: "journal",
      value: {
        pages: knownFact(
          Array.from({ length: count }, (_, index) => ({
            title: `Page ${index}`,
            family: "JournalEntryPage",
            usage: null,
            action_cost: unavailableFact("not_applicable"),
            navigation: {
              record_key: recordKey,
              embedded_path: [
                { collection: "/pages", identity: { Stable: `page-${index}` } },
              ],
              field: null,
              passage: null,
              source_fingerprint: null,
            },
          })),
        ),
      },
    },
  };
}

it.each([1, 22])(
  "scopes the page filter to its journal while retaining it on same-document refresh (next pages=%s)",
  (count) => {
    const props = { onReference: vi.fn() };
    const { rerender } = render(
      <RecordPresentation {...props} presentation={journal("journals:first", 22)} />,
    );
    const input = screen.getByRole("textbox", { name: "Find a page" });
    input.focus();
    fireEvent.change(input, { target: { value: "Page 21" } });
    rerender(
      <RecordPresentation {...props} presentation={journal("journals:first", 22)} />,
    );
    expect(screen.getByRole("textbox", { name: "Find a page" })).toBe(input);
    expect(input).toHaveValue("Page 21");
    expect(input).toHaveFocus();
    expect(screen.getAllByRole("link")).toHaveLength(1);
    rerender(
      <RecordPresentation
        {...props}
        presentation={journal("journals:second", count)}
      />,
    );
    expect(screen.getAllByRole("link")).toHaveLength(count);
    expect(screen.getByRole("link", { name: "Page 0" }).getAttribute("href")).toContain(
      "/records/journals:second?",
    );
    if (count > 20)
      expect(screen.getByRole("textbox", { name: "Find a page" })).toHaveValue("");
    else expect(screen.queryByRole("textbox", { name: "Find a page" })).toBeNull();
  },
);

it("keeps an active filter clearable when the same journal's page count drops", () => {
  const first = journal("journals:first", 22);
  const next = journal("journals:first", 1);
  if (first.body.kind !== "journal" || next.body.kind !== "journal")
    throw new Error("journal fixture");
  const props = { onReference: vi.fn() };
  const { rerender } = render(
    <JournalContents {...props} pages={first.body.value.pages} />,
  );
  const input = screen.getByRole("textbox", { name: "Find a page" });
  fireEvent.change(input, { target: { value: "Page 21" } });
  rerender(<JournalContents {...props} pages={next.body.value.pages} />);
  expect(screen.getByRole("textbox", { name: "Find a page" })).toBe(input);
  expect(screen.getByText("No matching pages.")).toBeVisible();
  fireEvent.change(input, { target: { value: "" } });
  expect(screen.getByRole("link", { name: "Page 0" })).toBeVisible();
});

it("keeps page filtering local, focused and source-ordered with native destinations", () => {
  const pages = Array.from({ length: 22 }, (_, index): EmbeddedRecordLinkView => ({
    title: `Page ${index}`,
    family: "JournalEntryPage",
    usage: null,
    action_cost: unavailableFact("not_applicable"),
    navigation: {
      record_key: "journals:root",
      embedded_path: [{ collection: "/pages", identity: { Stable: `page-${index}` } }],
      field: null,
      passage: null,
      source_fingerprint: null,
    },
  }));
  const open = vi.fn();
  render(<JournalContents pages={knownFact(pages)} onReference={open} />);
  const input = screen.getByRole("textbox", { name: "Find a page" });
  input.focus();
  fireEvent.change(input, { target: { value: "Page 1" } });
  expect(input).toHaveFocus();
  expect(screen.getAllByRole("link").map((link) => link.textContent)).toEqual([
    "Page 1",
    "Page 10",
    "Page 11",
    "Page 12",
    "Page 13",
    "Page 14",
    "Page 15",
    "Page 16",
    "Page 17",
    "Page 18",
    "Page 19",
  ]);
  const link = screen.getByRole("link", { name: "Page 10" });
  expect(link.getAttribute("href")).toContain("/records/journals:root?at=1.");
  fireEvent.click(link, { ctrlKey: true });
  expect(open).not.toHaveBeenCalled();
  fireEvent.click(link);
  expect(open).toHaveBeenCalledWith("journals:root", link, pages[10].navigation);
});
