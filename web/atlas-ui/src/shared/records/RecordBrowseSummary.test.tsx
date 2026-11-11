import { fireEvent, render, screen, within } from "@testing-library/react";
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

it("uses the selected-text role instead of the primary action color", () => {
  render(
    <RecordBrowseSummary
      record={summaryFixture("actors:selected", "Selected creature")}
      selected
      onPreview={vi.fn()}
    />,
  );

  expect(screen.getByRole("link", { name: /Selected creature/ })).toHaveStyle({
    color: "var(--selection-text)",
  });
});

it("shows compact scenario context on ungrouped rows and scopes grouped rows", () => {
  const record = {
    ...summaryFixture("actors:eltha", "Eltha"),
    kind_label: "Creature",
    level_label: "Level 11",
    publication: "Cavern of Sundered Songs",
    creature_display: {
      qualifiers: [],
      scenario_context: "Party levels 9–10 (Advanced) · 11–12",
      uses: [
        { id: "party-9-10", label: "Party levels 9–10 (Advanced)" },
        { id: "party-11-12", label: "Party levels 11–12" },
      ],
    },
  };
  const props = { record, selected: false, onPreview: vi.fn() };
  const view = render(<RecordBrowseSummary {...props} />);
  const link = screen.getByRole("link", {
    name: /Eltha.*Creature 11.*Party levels 9–10 \(Advanced\) · 11–12/,
  });
  expect(link).toHaveAttribute("href", "/records/actors:eltha");
  expect(
    within(link).getByText(record.creature_display.scenario_context),
  ).toBeVisible();
  view.rerender(<RecordBrowseSummary {...props} qualifiers={["Cavern · Advanced"]} />);
  expect(screen.getByText("Cavern · Advanced")).toBeVisible();
  expect(screen.queryByText(record.creature_display.scenario_context)).toBeNull();
  view.rerender(
    <RecordBrowseSummary
      {...props}
      record={{
        ...record,
        creature_display: {
          ...record.creature_display,
          scenario_context: "Cavern of Sundered Songs · Party levels 11–12",
          uses: [
            { id: "adventure", label: record.publication },
            record.creature_display.uses[1],
          ],
        },
      }}
    />,
  );
  expect(screen.getAllByText(/Cavern of Sundered Songs/)).toHaveLength(1);
  view.rerender(
    <RecordBrowseSummary
      {...props}
      record={{
        ...record,
        creature_display: {
          qualifiers: [],
          uses: [{ id: "adventure", label: record.publication }],
        },
      }}
    />,
  );
  expect(screen.getByText(record.publication)).toBeVisible();
});

it("selects the record from its plain trait badges and retains native modified links", () => {
  const onPreview = vi.fn();
  const record = {
    ...summaryFixture("weapons:dagger", "Dagger"),
    traits: [
      {
        kind: "trait",
        value: "deadly-d12",
        label: "Deadly d12",
        trait_key: "trait:deadly-d12",
      },
    ],
  };
  render(
    <RecordBrowseSummary record={record} selected={false} onPreview={onPreview} />,
  );
  const link = screen.getByRole("link", { name: /Dagger/ });
  const badge = within(link).getByText("Deadly d12");
  expect(screen.getAllByRole("link")).toHaveLength(1);
  expect(link).toHaveAttribute("href", "/records/weapons:dagger");
  fireEvent.click(badge);
  expect(onPreview).toHaveBeenCalledExactlyOnceWith(link);
  fireEvent.click(badge, { metaKey: true });
  expect(onPreview).toHaveBeenCalledTimes(1);
});
