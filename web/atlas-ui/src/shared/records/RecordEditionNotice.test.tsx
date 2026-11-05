import { fireEvent, render, screen } from "@testing-library/react";
import { detailFixture, summaryFixture } from "../../test/fixtures";
import { RecordEditionNotice } from "./RecordEditionNotice";
import { RecordEditionBadge } from "./RecordEditionBadge";
import type { RecordDetailView, RecordSummaryView } from "../../generated/atlas";
function setup(
  edition: RecordSummaryView["edition"],
  change?: (detail: RecordDetailView) => void,
) {
  const key = edition === "remaster" ? "rules:new" : "rules:old";
  const detail = detailFixture(key);
  detail.record.edition = edition;
  detail.presentation.identity.edition = edition;
  detail.counterparts = {
    kind: "content",
    content: {
      seed: summaryFixture(key),
      links: [
        {
          legacy_record: summaryFixture("rules:old", "Old name"),
          remaster_record: summaryFixture("rules:new", "New name"),
          provenance: {
            source: {
              record: "journals:changes",
              embedded_path: [],
              field: "/text/content",
            },
            attribution: "Remaster changes",
            navigation: null,
            unavailable_reason: null,
          },
        },
      ],
    },
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
it.each(["legacy", "remaster"] as const)(
  "shows %s status and an inline native counterpart link that previews locally",
  (edition) => {
    const { open } = setup(edition);
    expect(
      screen.getByText(edition === "legacy" ? "Legacy" : "Remaster"),
    ).toBeVisible();
    const title = edition === "legacy" ? "New name" : "Old name";
    const key = edition === "legacy" ? "rules:new" : "rules:old";
    const link = screen.getByRole("link", { name: title });
    expect(link).toHaveAttribute("href", `/records/${key}`);
    fireEvent.click(link, { ctrlKey: true });
    expect(open).not.toHaveBeenCalled();
    fireEvent.click(link);
    expect(open).toHaveBeenCalledWith(key, link);
  },
);
it("shows verified counterparts even when publication metadata is unavailable", () => {
  setup(undefined);
  expect(screen.getByRole("link", { name: "New name" })).toBeVisible();
  expect(screen.queryByText("Legacy")).toBeNull();
  expect(screen.queryByText("Remaster")).toBeNull();
});
it("does not lend root edition mappings to an embedded document", () => {
  setup("legacy", (detail) => {
    detail.selected.embedded_path = [
      { collection: "items", identity: { SnapshotLocal: { index: 0 } } },
    ];
  });
  expect(screen.queryByRole("link")).toBeNull();
});
it("keeps legacy status without claiming there is no replacement when no pair is indexed", () => {
  setup("legacy", (detail) => {
    detail.counterparts = { kind: "empty" };
  });
  expect(screen.getByText("Legacy")).toBeVisible();
  expect(screen.queryByRole("link")).toBeNull();
});
it.each(["record_not_found", "artifact_generation_changed"] as const)(
  "keeps edition status readable with %s and offers record reload",
  (code) => {
    const { reload } = setup("legacy", (detail) => {
      detail.counterparts = {
        kind: "error",
        error: { code, message: "private diagnostic" },
      };
    });
    expect(screen.getByText("Legacy")).toBeVisible();
    expect(screen.getByRole("status")).toHaveTextContent("Edition links unavailable");
    expect(screen.queryByText("private diagnostic")).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Reload record" }));
    expect(reload).toHaveBeenCalledOnce();
  },
);
