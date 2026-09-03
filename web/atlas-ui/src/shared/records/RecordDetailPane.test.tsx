import { render, screen } from "@testing-library/react";
import { recordDetailFixture } from "../../test/recordFixtures";
import { RecordDetailPane } from "./RecordDetailPane";

describe("RecordDetailPane", () => {
  it("renders the typed creature surface inside the detail panel", () => {
    const { container } = render(
      <RecordDetailPane
        detail={recordDetailFixture({ title: "Dirge of Doom" })}
        loading={false}
        onReference={vi.fn()}
      />,
    );

    expect(container.querySelector(".detail-panel")).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Dirge of Doom" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Overview" })).toBeInTheDocument();
  });

  it("renders an accessible loading state", () => {
    render(
      <RecordDetailPane
        detail={undefined}
        loading
        loadingMessage="Loading creature"
        onReference={vi.fn()}
      />,
    );

    expect(
      screen.getByLabelText("Loading creature").closest(".detail-panel"),
    ).toHaveAttribute("aria-busy", "true");
  });

  it("renders custom empty and error states", () => {
    render(
      <RecordDetailPane
        detail={undefined}
        emptyMessage="This saved record is unresolved."
        errors={[new Error("Unable to load detail")]}
        loading={false}
        onReference={vi.fn()}
      />,
    );

    expect(screen.getByText("This saved record is unresolved.")).toBeInTheDocument();
    expect(screen.getByText("Unable to load detail")).toBeInTheDocument();
  });

  it("keeps stale content visible while announcing refresh", () => {
    render(
      <RecordDetailPane
        detail={recordDetailFixture()}
        loading={false}
        onReference={vi.fn()}
        stale
      />,
    );

    expect(screen.getByRole("heading", { name: "Goblin Warrior" })).toBeInTheDocument();
    expect(screen.getByText("Refreshing this record…")).toBeInTheDocument();
  });
});
