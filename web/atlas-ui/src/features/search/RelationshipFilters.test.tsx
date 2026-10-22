import { fireEvent, render, screen } from "@testing-library/react";
import { LinkFilterControls } from "../../shared/filters/LinkFilterControls";
import { linkLeaf } from "../../shared/filters/linkPredicates";
import { rootNavigation } from "../../shared/records/useReferenceReading";
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
function setup(filter = leaf as import("../../generated/atlas").QueryPredicate) {
  const edit = vi.fn().mockReturnValue(null),
    remove = vi.fn(),
    advanced = vi.fn();
  const view = render(
    <LinkFilterControls
      filter={filter}
      fields={[]}
      title={() => "Leather Armor"}
      onAdd={vi.fn()}
      onEdit={edit}
      onRemove={remove}
      onAdvanced={advanced}
    />,
  );
  return { edit, remove, advanced, view };
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
  expect(
    await screen.findByRole("link", { name: "Open Leather Armor" }),
  ).toHaveAttribute("href", expect.stringContaining("selection="));
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
it("preserves OR and NOT grouping in a single truthful summary rather than flat chips", () => {
  const { advanced } = setup({
    kind: "any_of",
    children: [
      leaf,
      {
        kind: "not",
        predicate: {
          ...leaf,
          clause_id: "two",
          relationship: { ...leaf.relationship, direction: "outgoing" },
        },
      },
    ],
  });
  expect(screen.queryByRole("group", { name: "Relationship filter" })).toBeNull();
  expect(
    screen.getByText(/\(Links to Leather Armor.* OR NOT Linked from Leather Armor/),
  ).toBeInTheDocument();
  fireEvent.click(
    screen.getByRole("button", { name: /Grouped filters: 2 link conditions/ }),
  );
  expect(advanced).toHaveBeenCalled();
});
