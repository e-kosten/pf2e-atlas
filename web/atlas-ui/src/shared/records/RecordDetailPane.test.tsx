import {
  detailFixture,
  actorFixture,
  numberFact,
  summaryFixture,
} from "../../test/fixtures";
import { fireEvent, render, screen } from "@testing-library/react";
import { AtlasApiError } from "../../api/atlasApi";
import type { RecordDetailView } from "../../generated/atlas";
import { RecordDetailPane } from "./RecordDetailPane";

describe("RecordDetailPane", () => {
  it("wraps record presentation with a detail panel", () => {
    const { container } = render(
      <RecordDetailPane
        detail={recordDetailFixture()}
        loading={false}
        onReference={vi.fn()}
      />,
    );

    expect(container.querySelector(".detail-panel")).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Dirge of Doom" })).toBeInTheDocument();
  });

  it("renders a custom empty state when there is no selection", () => {
    render(
      <RecordDetailPane
        detail={undefined}
        emptyMessage="This saved record is unresolved."
        loading={false}
        onReference={vi.fn()}
      />,
    );

    expect(screen.getByText("This saved record is unresolved.")).toBeInTheDocument();
  });

  it("keeps valid content while refreshing and exposes a local retry after failure", () => {
    const onRetry = vi.fn();
    const props = { detail: recordDetailFixture(), onReference: vi.fn(), onRetry };
    const view = render(<RecordDetailPane {...props} loading />);
    expect(screen.getByRole("heading", { name: "Dirge of Doom" })).toBeVisible();
    expect(screen.getByLabelText("Updating record")).toBeInTheDocument();
    view.rerender(
      <RecordDetailPane
        {...props}
        loading={false}
        error={new Error("Service unavailable")}
      />,
    );
    expect(screen.getByRole("heading", { name: "Dirge of Doom" })).toBeVisible();
    expect(screen.getByRole("alert")).toHaveTextContent(
      "Previously loaded content remains below.",
    );
    fireEvent.click(screen.getByRole("button", { name: "Retry" }));
    expect(onRetry).toHaveBeenCalledTimes(1);
  });

  it("distinguishes typed not-found from no selection and preserves workflow recovery", () => {
    const onBack = vi.fn();
    const error = new AtlasApiError(404, "Internal record key was not found", {
      code: "record_not_found",
      message: "Internal record key was not found",
    });
    render(
      <RecordDetailPane
        detail={undefined}
        loading={false}
        error={error}
        onBack={onBack}
        backLabel="Back to results"
        onReference={vi.fn()}
      />,
    );
    expect(screen.getByRole("alert")).toHaveTextContent("Record unavailable");
    expect(screen.queryByText("Select a record")).toBeNull();
    expect(screen.queryByText("Internal record key was not found")).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Back to results" }));
    expect(onBack).toHaveBeenCalledTimes(1);
  });

  it("keeps invalid-address details local with a native search recovery link", () => {
    const error = new AtlasApiError(
      400,
      "Reopen this selected passage from its parent record.",
      {
        code: "invalid_request",
        message: "Reopen this selected passage from its parent record.",
      },
    );
    render(
      <RecordDetailPane
        detail={undefined}
        loading={false}
        error={error}
        onReference={vi.fn()}
      />,
    );
    expect(screen.getByRole("alert")).toHaveTextContent(error.message);
    expect(screen.getByRole("link", { name: "Back to search" })).toHaveAttribute(
      "href",
      "/search",
    );
  });

  it("renders the semantic family facts once", () => {
    render(
      <RecordDetailPane
        detail={{
          ...recordDetailFixture(),
          presentation: {
            classification: "Creature 3",
            identity: {
              ...summaryFixture("actors:testCreature", "Test Creature"),
              kind: "creature",
              kind_label: "Creature",
              level_label: "3",
              traits: [{ kind: "trait", label: "hag", value: "hag" }],
            },
            body: {
              kind: "creature",
              value: { ...actorFixture(), armor_class: numberFact(25) },
            },
            content: [],
            embedded: [],
          },
        }}
        loading={false}
        onReference={vi.fn()}
      />,
    );

    expect(screen.getAllByRole("heading", { name: "Test Creature" })).toHaveLength(1);
    expect(screen.getByText("AC")).toBeInTheDocument();
    expect(screen.getByText("25")).toBeInTheDocument();
    expect(screen.queryByText("Source presentation")).toBeNull();
  });
});

function recordDetailFixture(): RecordDetailView {
  return detailFixture("spell:dirge", "Dirge of Doom");
}
