import { render, screen } from "@testing-library/react";
import { summaryFixture } from "../../test/fixtures";
import { RecordBrowseSummary } from "./RecordBrowseSummary";

it("renders the server's fractional unit-price label verbatim", () => {
  const record = {
    ...summaryFixture("items:armor", "Armor"),
    unit_price_label: "0.005 gp each",
  };
  const { rerender } = render(
    <RecordBrowseSummary record={record} selected={false} onPreview={vi.fn()} />,
  );
  expect(screen.getByText("0.005 gp each", { exact: true })).toBeVisible();
  rerender(
    <RecordBrowseSummary
      record={{ ...record, unit_price_label: undefined }}
      selected={false}
      onPreview={vi.fn()}
    />,
  );
  expect(screen.queryByText(/gp each/)).toBeNull();
});
