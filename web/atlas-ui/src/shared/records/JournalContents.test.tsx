import { fireEvent, render, screen } from "@testing-library/react";
import { expect, it, vi } from "vitest";
import type { EmbeddedRecordLinkView } from "../../generated/atlas";
import { JournalContents } from "./JournalContents";
import { knownFact, unavailableFact } from "../../test/fixtures";

it("keeps page filtering local, focused and source-ordered with native destinations", () => {
  const pages = Array.from(
    { length: 22 },
    (_, index): EmbeddedRecordLinkView => ({
      title: `Page ${index}`,
      family: "JournalEntryPage",
      usage: null,
      action_cost: unavailableFact("not_applicable"),
      navigation: {
        record_key: "journals:root",
        embedded_path: [
          { collection: "/pages", identity: { Stable: `page-${index}` } },
        ],
        field: null,
        passage: null,
        source_fingerprint: null,
      },
    }),
  );
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
  expect(link.getAttribute("href")).toContain("/records/journals:root?selection=");
  fireEvent.click(link, { ctrlKey: true });
  expect(open).not.toHaveBeenCalled();
  fireEvent.click(link);
  expect(open).toHaveBeenCalledWith("journals:root", link, pages[10].navigation);
});
