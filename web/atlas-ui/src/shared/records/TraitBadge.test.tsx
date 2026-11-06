import { fireEvent, render, screen } from "@testing-library/react";
import { ReferencePreviewContext } from "./ReferencePreviewContext";
import { TraitBadge } from "./TraitBadge";

it("opens exact variant definitions through the contextual reader", () => {
  const openTrait = vi.fn();
  history.replaceState(null, "", "/search");
  render(
    <ReferencePreviewContext value={{ open: vi.fn(), openTrait }}>
      <TraitBadge
        trait={{
          kind: "trait",
          value: "deadly-d12",
          label: "Deadly d12",
          trait_key: "trait:deadly-d12",
        }}
      />
    </ReferencePreviewContext>,
  );
  const link = screen.getByRole("link", { name: "Deadly d12" });
  expect(link.getAttribute("href")).toBe("/traits/trait:deadly-d12");
  expect(link.parentElement?.closest("a")).toBeNull();
  fireEvent.click(link);
  expect(openTrait).toHaveBeenCalledWith("trait:deadly-d12", link);
  expect(location.pathname).toBe("/search");
});

it("retains plain labels for namespaces without a definition destination", () => {
  render(
    <TraitBadge trait={{ kind: "other_tag", value: "custom", label: "Custom" }} />,
  );
  expect(screen.getByText("Custom")).toBeVisible();
  expect(screen.queryByRole("link")).toBeNull();
});

it("preserves native modified clicks", () => {
  history.replaceState(null, "", "/search");
  render(
    <TraitBadge
      trait={{
        kind: "trait",
        value: "agile",
        label: "Agile",
        trait_key: "trait:agile",
      }}
    />,
  );
  fireEvent.click(screen.getByRole("link"), { ctrlKey: true });
  expect(location.pathname).toBe("/search");
});
