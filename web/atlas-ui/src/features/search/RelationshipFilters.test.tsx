import { ConfigProvider } from "antd";
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
    remove = vi.fn();
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
it("opens the modal for the exact existing relationship and removes only its own leaf", () => {
  const { edit, remove } = setup();
  fireEvent.click(screen.getByRole("button", { name: /Links to Leather Armor/ }));
  expect(edit).toHaveBeenCalledWith(leaf);
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
