import { fireEvent, render, screen } from "@testing-library/react";
import { TraitBadge } from "./TraitBadge";
import { currentAtlasRoute } from "../../app/routes";
import { RecordBrowseSummary } from "./RecordBrowseSummary";
import { summaryFixture } from "../../test/fixtures";

it("opens exact variant definitions without triggering record selection", () => {
  history.replaceState(null, "", "/search");
  const preview = vi.fn();
  const record = summaryFixture("weapons:dagger", "Dagger");
  record.traits = [
    {
      kind: "trait",
      value: "deadly-d12",
      label: "Deadly d12",
      trait_key: "trait:deadly-d12",
    },
  ];
  render(<RecordBrowseSummary record={record} selected={false} onPreview={preview} />);
  const link = screen.getByRole("link", { name: "Deadly d12" });
  expect(link.getAttribute("href")).toBe("/traits/trait:deadly-d12");
  expect(link.parentElement?.closest("a")).toBeNull();
  fireEvent.click(link);
  expect(preview).not.toHaveBeenCalled();
  expect(currentAtlasRoute()).toEqual({ kind: "trait", traitKey: "trait:deadly-d12" });
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
  expect(currentAtlasRoute().kind).toBe("search");
});
