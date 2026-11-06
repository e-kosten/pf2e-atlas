import { ConfigProvider } from "antd";
import { parseSearchParams, selectedNavigation } from "../../app/urlState";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { LinkFilterControls } from "../../shared/filters/LinkFilterControls";
import { linkLeaf } from "../../shared/filters/linkPredicates";
import { rootNavigation } from "../../shared/records/useReferenceReading";
const reference = vi.hoisted(() => ({ open: vi.fn() }));
vi.mock("../../shared/records/ReferenceHelp", () => ({
  useReferenceRecordNavigation: () => reference.open,
}));
const leaf = linkLeaf(
  {
    direction: "incoming",
    include_contents: true,
    selection: {
      ...rootNavigation("equipment:secret-id"),
      embedded_path: [
        { collection: "items", identity: { SnapshotLocal: { index: 0 } } },
      ],
      source_fingerprint: "checked-source",
    },
  },
  "one",
);
function setup(
  filter = leaf as import("../../generated/atlas").QueryPredicate,
  openReference = vi.fn(),
) {
  const edit = vi.fn().mockReturnValue(null),
    remove = vi.fn();
  reference.open = openReference;
  const view = render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <LinkFilterControls
        filter={filter}
        title={() => "Leather Armor"}
        onEdit={edit}
        onRemove={remove}
      />
    </ConfigProvider>,
  );
  return { edit, remove, view };
}
it("shows human link labels and exact scope without IDs", () => {
  const { view } = setup();
  expect(
    screen.getByRole("button", { name: /Links to Leather Armor; Whole entry/ }),
  ).toBeInTheDocument();
  expect(view.container).not.toHaveTextContent("secret-id");
});
it("changes scope while preserving the exact embedded selection", async () => {
  const { edit } = setup();
  fireEvent.click(screen.getByRole("button", { name: /Links to Leather Armor/ }));
  fireEvent.click(await screen.findByRole("radio", { name: "Main entry only" }));
  expect(edit).toHaveBeenCalledWith(leaf, {
    ...leaf.relationship,
    include_contents: false,
  });
});
it("changes direction while preserving exact scope and target", async () => {
  const { edit } = setup();
  fireEvent.click(screen.getByRole("button", { name: /Links to Leather Armor/ }));
  fireEvent.click(await screen.findByRole("radio", { name: "Linked from" }));
  expect(edit).toHaveBeenCalledWith(leaf, {
    ...leaf.relationship,
    direction: "outgoing",
  });
});
it("offers the exact native target and reuses the picker for changing it", async () => {
  const { edit } = setup();
  fireEvent.click(screen.getByRole("button", { name: /Links to Leather Armor/ }));
  const target = new URL(
    (await screen.findByRole("link", { name: "Open Leather Armor" })).getAttribute(
      "href",
    )!,
    "http://atlas.local",
  );
  expect(
    selectedNavigation(parseSearchParams(target.search), "equipment:secret-id"),
  ).toEqual(leaf.relationship.selection);
  fireEvent.click(screen.getByRole("button", { name: "Change record" }));
  expect(edit).toHaveBeenCalledWith(leaf);
});
it("keeps duplicate feedback local and preserves the chip", async () => {
  const { edit } = setup();
  edit.mockReturnValue("This link condition is already added.");
  const chip = screen.getByRole("button", { name: /Links to Leather Armor/ });
  fireEvent.click(chip);
  fireEvent.click(await screen.findByRole("radio", { name: "Linked from" }));
  expect(
    await screen.findByText("This link condition is already added."),
  ).toBeInTheDocument();
  expect(chip).toHaveAttribute("aria-expanded", "true");
});
it("removes only the selected leaf and restores focus on popover Escape", async () => {
  const { remove } = setup();
  const chip = screen.getByRole("button", { name: /Links to Leather Armor/ });
  fireEvent.click(chip);
  fireEvent.keyDown(await screen.findByRole("radio", { name: "Whole entry" }), {
    key: "Escape",
  });
  expect(chip).toHaveFocus();
  fireEvent.click(screen.getByRole("button", { name: "Remove relationship filter" }));
  expect(remove).toHaveBeenCalledWith(leaf);
});
it("keeps grouped OR and NOT links out of independent chips", () => {
  setup({
    kind: "any_of",
    children: [leaf, { kind: "not", predicate: { ...leaf, clause_id: "two" } }],
  });
  expect(screen.queryByRole("group", { name: "Relationship filter" })).toBeNull();
  expect(screen.queryByText(/Grouped filters/)).toBeNull();
});

it("closes the filter popover before opening its reference and returns to the persistent chip", async () => {
  const openReference = vi.fn();
  setup(leaf, openReference);
  const chip = screen.getByRole("button", { name: /Links to Leather Armor/ });
  fireEvent.click(chip);
  const link = await screen.findByRole("link", { name: "Open Leather Armor" });
  fireEvent.click(link);
  expect(chip).toHaveAttribute("aria-expanded", "false");
  expect(openReference).toHaveBeenCalledWith(
    leaf.relationship.selection.record_key,
    chip,
    leaf.relationship.selection,
  );
  await waitFor(() => expect(link).not.toBeVisible());
});
it("preserves modified reference links without closing the filter popover", async () => {
  const openReference = vi.fn();
  setup(leaf, openReference);
  const chip = screen.getByRole("button", { name: /Links to Leather Armor/ });
  fireEvent.click(chip);
  const link = await screen.findByRole("link", { name: "Open Leather Armor" });
  fireEvent.click(link, { ctrlKey: true });
  expect(openReference).not.toHaveBeenCalled();
  expect(chip).toHaveAttribute("aria-expanded", "true");
});
