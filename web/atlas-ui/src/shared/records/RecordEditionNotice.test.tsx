import { fireEvent, render, screen } from "@testing-library/react";
import { detailFixture } from "../../test/fixtures";
import { remasterInformationFixture } from "../../test/remasterFixtures";
import { rootNavigation } from "./useReferenceReading";
import { RecordEditionNotice } from "./RecordEditionNotice";
import { RecordEditionBadge } from "./RecordEditionBadge";
import type { RecordDetailView, RecordSummaryView } from "../../generated/atlas";
function setup(
  edition: RecordSummaryView["edition"],
  change?: (detail: RecordDetailView) => void,
) {
  const detail = detailFixture("rules:old");
  detail.record.edition = edition;
  detail.presentation.identity.edition = edition;
  detail.remaster_relationships = {
    kind: "content",
    content: remasterInformationFixture(),
  };
  change?.(detail);
  const open = vi.fn(),
    reload = vi.fn();
  render(
    <>
      <RecordEditionBadge edition={edition} />
      <RecordEditionNotice detail={detail} onReference={open} onReload={reload} />
    </>,
  );
  return { open, reload };
}
it("renders projected notices above rules with native exact links and normal document semantics", () => {
  const { open } = setup("legacy");
  expect(screen.getByText("Legacy")).toBeVisible();
  expect(screen.getByText("Renamed to:")).toBeVisible();
  expect(screen.queryByRole("alert")).toBeNull();
  const link = screen.getByRole("link", { name: "New name" });
  expect(link).toHaveAttribute("href", "/records/rules:new");
  fireEvent.click(link, { ctrlKey: true });
  expect(open).not.toHaveBeenCalled();
  fireEvent.click(link);
  expect(open).toHaveBeenCalledWith("rules:new", link, rootNavigation("rules:new"));
});
it("renders source-backed information even when publication era is unknown", () => {
  setup(undefined);
  expect(screen.getByRole("link", { name: "New name" })).toBeVisible();
  expect(screen.queryByText("Legacy")).toBeNull();
});
it("renders an exact owned notice supplied by Rust without borrowing root classification", () => {
  setup("legacy", (detail) => {
    detail.selected.embedded_path = [
      { collection: "items", identity: { SnapshotLocal: { index: 0 } } },
    ];
    detail.remaster_relationships = { kind: "empty" };
    detail.source_remaster_context = {
      kind: "content",
      content: {
        heading: "Source entry: Old name",
        navigation: rootNavigation("rules:old"),
        relationships: { kind: "content", content: remasterInformationFixture() },
      },
    };
  });
  expect(
    screen.getByRole("region", { name: "Source-entry remaster information" }),
  ).toBeVisible();
  expect(screen.getByRole("link", { name: "New name" })).toBeVisible();
});
it("keeps legacy metadata without inventing a replacement when information is empty", () => {
  setup("legacy", (detail) => {
    detail.remaster_relationships = { kind: "empty" };
  });
  expect(screen.getByText("Legacy")).toBeVisible();
  expect(screen.queryByRole("link")).toBeNull();
});
it("keeps readable identity when information fails and supports local retry", () => {
  const { reload } = setup("legacy", (detail) => {
    detail.remaster_relationships = {
      kind: "error",
      error: { code: "record_not_found", message: "private diagnostic" },
    };
  });
  expect(screen.getByText("Legacy")).toBeVisible();
  expect(screen.queryByText("private diagnostic")).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Retry" }));
  expect(reload).toHaveBeenCalledOnce();
});
