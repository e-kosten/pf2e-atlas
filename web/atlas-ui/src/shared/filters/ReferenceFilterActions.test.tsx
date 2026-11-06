import { fireEvent, render, screen } from "@testing-library/react";
import { ReferenceFilterActions } from "./ReferenceFilterActions";
import type { ReferenceFilterActionView } from "../../generated/atlas";
import { fieldFixture } from "../../test/fixtures";
import { navigateDestination } from "../../app/navigation";
import { decodeFilterUrl } from "./filterUrl";

vi.mock("../../app/navigation", async (original) => ({
  ...(await original<typeof import("../../app/navigation")>()),
  navigateDestination: vi.fn(),
}));
const action: ReferenceFilterActionView = {
  kind: "search",
  label: "Exact speakers",
  artifact_generation: "generation-a",
  predicate: {
    kind: "set_match",
    field: "languages",
    op: "includes_any",
    values: ["chthonian"],
  },
};
beforeEach(() => vi.clearAllMocks());
it("uses the same supplied action for local insertion and native canonical search links", () => {
  const onAction = vi.fn(() => null);
  const { rerender } = render(
    <ReferenceFilterActions
      actions={[action]}
      generation="generation-a"
      onAction={onAction}
    />,
  );
  fireEvent.click(screen.getByRole("button", { name: "Exact speakers" }));
  expect(onAction).toHaveBeenCalledWith(action);
  expect(navigateDestination).not.toHaveBeenCalled();
  rerender(<ReferenceFilterActions actions={[action]} generation="generation-a" />);
  const link = screen.getByRole("link", { name: "Exact speakers" });
  const url = new URL(link.getAttribute("href")!, window.location.origin);
  expect(url.pathname).toBe("/search");
  expect(decodeFilterUrl(url.searchParams.get("filter")!)).toMatchObject(
    action.predicate,
  );
  fireEvent.click(link, { ctrlKey: true });
  expect(navigateDestination).not.toHaveBeenCalled();
  fireEvent.click(link);
  expect(navigateDestination).toHaveBeenCalledWith(expect.any(Object), {
    dismissReadingOverlays: true,
  });
});
it("rejects stale action execution and keeps numeric configuration out of executable filter URLs", () => {
  const onAction = vi.fn(() => null);
  const { rerender } = render(
    <ReferenceFilterActions
      actions={[action]}
      generation="generation-b"
      onAction={onAction}
    />,
  );
  fireEvent.click(screen.getByRole("button", { name: "Exact speakers" }));
  expect(onAction).not.toHaveBeenCalled();
  expect(screen.getByRole("status")).toHaveTextContent("reference collection changed");
  rerender(
    <ReferenceFilterActions
      actions={[
        {
          kind: "configure_field",
          label: "Strength",
          field: fieldFixture("abilities.str.modifier", "number", "Strength"),
        },
      ]}
      generation="generation-b"
    />,
  );
  const url = new URL(
    screen.getByRole("link", { name: "Strength" }).getAttribute("href")!,
    window.location.origin,
  );
  expect(url.searchParams.get("configure_field")).toBe("abilities.str.modifier");
  expect(url.searchParams.has("filter")).toBe(false);
});
