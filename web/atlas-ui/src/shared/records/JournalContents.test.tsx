import { fireEvent, render, screen, within } from "@testing-library/react";
import { expect, it, vi } from "vitest";
import { decodeReadingAddress } from "../../app/readingUrl";
import { JournalContents, type JournalContentsEntry } from "./JournalContents";
import { RecordPresentation } from "./RecordPresentation";
import { knownFact, summaryFixture, unavailableFact } from "../../test/fixtures";
import type { RecordPresentationView } from "../../generated/atlas";

function entry(recordKey: string, index: number): JournalContentsEntry {
  return {
    parent_index: null,
    is_heading_only: false,
    link: {
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
    },
  };
}

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
          Array.from({ length: count }, (_, index) => entry(recordKey, index)),
        ),
        inline_page: null,
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
  const pages = Array.from({ length: 22 }, (_, index) => entry("journals:root", index));
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
  expect(open).toHaveBeenCalledWith("journals:root", link, pages[10].link.navigation);
});

it("opens the exact page when a linked section and its child share a title", () => {
  const parent = entry("journals:root", 0);
  const child = entry("journals:root", 1);
  parent.link.title = "Rules";
  child.link.title = "Rules";
  child.parent_index = 0;
  const open = vi.fn();
  render(<JournalContents pages={knownFact([parent, child])} onReference={open} />);

  const links = screen.getAllByRole("link", { name: "Rules" });
  expect(links).toHaveLength(2);
  expect(within(screen.getByRole("heading", { name: "Rules" })).getByRole("link")).toBe(
    links[0],
  );
  for (const [index, item] of [parent, child].entries()) {
    fireEvent.click(links[index]);
    expect(open).toHaveBeenLastCalledWith(
      "journals:root",
      links[index],
      item.link.navigation,
    );
  }
});

it("keeps headings, section pages and matching descendants together while filtering", () => {
  const pages = Array.from({ length: 24 }, (_, index) => entry("journals:root", index));
  const [common, dwarf, elf, rules, travel] = pages;
  common.link.title = "Common";
  common.is_heading_only = true;
  dwarf.link.title = "Dwarf";
  dwarf.parent_index = 0;
  elf.link.title = "Elf";
  elf.parent_index = 0;
  rules.link.title = "Playing the Game";
  travel.link.title = "Travel Speed";
  travel.parent_index = 3;
  render(<JournalContents pages={knownFact(pages)} onReference={vi.fn()} />);
  expect(screen.queryByRole("link", { name: "Common" })).toBeNull();
  expect(
    within(screen.getByRole("heading", { name: "Playing the Game" })).getByRole("link"),
  ).toBeVisible();
  const input = screen.getByRole("textbox", { name: "Find a page" });
  fireEvent.change(input, { target: { value: "dwarf" } });
  expect(screen.getByRole("heading", { name: "Common" })).toBeVisible();
  expect(screen.getAllByRole("link").map((link) => link.textContent)).toEqual([
    "Dwarf",
  ]);
  expect(screen.getByRole("status")).toHaveTextContent("1 of 23 pages");
  fireEvent.change(input, { target: { value: "common" } });
  expect(screen.getAllByRole("link").map((link) => link.textContent)).toEqual([
    "Dwarf",
    "Elf",
  ]);
  fireEvent.change(input, { target: { value: "Travel Speed" } });
  expect(screen.getAllByRole("link").map((link) => link.textContent)).toEqual([
    "Playing the Game",
    "Travel Speed",
  ]);
});

it("reads a sole page under the journal identity with exact child outline destinations", () => {
  const presentation = journal("journals:root", 1);
  if (presentation.body.kind !== "journal") throw new Error("Expected journal");
  const journalBody = presentation.body.value;
  const child = journalBody.pages.value![0].link;
  child.navigation.embedded_path = [
    { collection: "/pages", identity: { SnapshotLocal: { index: 0 } } },
  ];
  child.navigation.source_fingerprint = "checked-source";
  const content = {
    locator: {
      record: child.navigation.record_key,
      embedded_path: child.navigation.embedded_path,
      field: "/text/content",
    },
    source_fingerprint: child.navigation.source_fingerprint,
    role: "description",
    body: {
      kind: "html" as const,
      html: "<h1>First outcome</h1><p>Rules text.</p><h1>Second outcome</h1><h1>Third outcome</h1>",
      inclusions: [],
      controls: [],
    },
  };
  journalBody.inline_page = {
    page: { show_body_heading: knownFact(true), media_notice: null },
    relationships: [],
    relationships_truncated: false,
    content: [content],
  };
  const read = () => (
    <RecordPresentation presentation={presentation} onReference={vi.fn()} />
  );
  const { rerender } = render(read());
  expect(screen.getByRole("heading", { name: "journals:root" })).toBeVisible();
  expect(screen.getByRole("heading", { name: "Page 0" })).toBeVisible();
  expect(screen.getByText("Rules text.")).toBeVisible();
  expect(screen.queryByRole("heading", { name: "Contents" })).toBeNull();
  const section = new URL(
    screen.getByRole<HTMLAnchorElement>("link", { name: "First outcome" }).href,
  );
  expect(
    decodeReadingAddress(
      section.searchParams.get("at")!,
      child.navigation.record_key,
      false,
    ),
  ).toEqual(child.navigation);
  expect(section.hash.slice(1)).toBe(
    screen.getByRole("heading", { name: "First outcome" }).id,
  );
  journalBody.inline_page.page.show_body_heading = knownFact(false);
  rerender(read());
  expect(screen.queryByRole("heading", { name: "Page 0" })).toBeNull();
  child.title = presentation.identity.title;
  journalBody.inline_page = {
    page: {
      show_body_heading: knownFact(true),
      media_notice: "Media preview unavailable (image).",
    },
    relationships: [],
    relationships_truncated: false,
    content: [],
  };
  rerender(read());
  expect(screen.getAllByRole("heading", { name: "journals:root" })).toHaveLength(1);
  expect(screen.getByText("Media preview unavailable (image).")).toBeVisible();
});
