import { fireEvent, render, screen } from "@testing-library/react";
import { detailFixture, summaryFixture } from "../../test/fixtures";
import { RecordPresentation } from "./RecordPresentation";

it("keeps all recipient destinations inline with native links and drawer activation", () => {
  const presentation = detailFixture("backgrounds:hunter", "Hunter").presentation;
  presentation.grants = {
    entries: Array.from({ length: 6 }, (_, index) => ({
      title: `Benefit ${index}`,
      record: summaryFixture(`feats:benefit${index}`, `Benefit ${index}`),
      navigation: {
        record_key: `feats:benefit${index}`,
        embedded_path: [],
        field: null,
        passage: null,
        source_fingerprint: null,
      },
    })),
    unavailable_count: 0,
    truncated: false,
  };
  const onReference = vi.fn();
  render(<RecordPresentation presentation={presentation} onReference={onReference} />);
  expect(screen.getAllByRole("listitem")).toHaveLength(6);
  const last = screen.getByRole("link", { name: /Benefit 5/ });
  expect(last).toHaveAttribute(
    "href",
    expect.stringContaining("/records/feats:benefit5?selection="),
  );
  fireEvent.click(last, { ctrlKey: true });
  expect(onReference).not.toHaveBeenCalled();
  fireEvent.click(last);
  expect(onReference).toHaveBeenCalledWith(
    "feats:benefit5",
    last,
    presentation.grants.entries[5].navigation,
  );
  expect(screen.queryByRole("button", { name: /search|matches/i })).toBeNull();
});

it("distinguishes unavailable grants from an empty section", () => {
  const presentation = detailFixture("effects:source", "Source effect").presentation;
  presentation.grants = { entries: [], unavailable_count: 1, truncated: false };
  const { rerender } = render(
    <RecordPresentation presentation={presentation} onReference={vi.fn()} />,
  );
  expect(screen.getByText("One granted reference is unavailable.")).toBeVisible();
  delete presentation.grants;
  rerender(<RecordPresentation presentation={presentation} onReference={vi.fn()} />);
  expect(screen.queryByRole("region", { name: "Grants" })).toBeNull();
});
