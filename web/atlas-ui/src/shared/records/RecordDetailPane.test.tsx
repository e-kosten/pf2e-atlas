import {
  detailFixture,
  actorFixture,
  numberFact,
  summaryFixture,
} from "../../test/fixtures";
import { fireEvent, render, screen } from "@testing-library/react";
import { AtlasApiError } from "../../api/atlasApi";
import type { RecordDetailView } from "../../generated/atlas";
import { remasterInformationFixture } from "../../test/remasterFixtures";
import { RouterTestContext } from "../../test/router";
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

  it("keeps routine remaster context in a non-exploring reader by default", () => {
    const detail = recordDetailFixture();
    detail.remaster_relationships = {
      kind: "content",
      content: remasterInformationFixture(),
    };
    render(
      <RecordDetailPane
        detail={detail}
        loading={false}
        showRelated={false}
        onReference={vi.fn()}
      />,
    );
    expect(screen.getByRole("heading", { name: "Dirge of Doom" })).toBeVisible();
    expect(screen.getByText("Renamed to:")).toBeVisible();
    expect(screen.getByRole("link", { name: "New name" })).toHaveAttribute(
      "href",
      "/records/rules:new",
    );
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
    expect(screen.getByRole("alert")).toHaveTextContent("Unable to load this record");
    expect(screen.queryByText(error.message)).toBeNull();
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

  it("shows the friendly scenario heading and puts the reviewed uses after Family", () => {
    const detail = detailFixture("actors:eltha", "Advanced Eltha (9-10)/Eltha (11-12)");
    const identity = {
      ...detail.record,
      kind: "creature",
      kind_label: "Creature",
      publication: "Cavern of Sundered Songs",
      creature_display: {
        title: "Eltha",
        qualifiers: [],
        uses: [
          { id: "party-9-10", label: "Party levels 9–10 (Advanced)" },
          { id: "party-11-12", label: "Party levels 11–12" },
        ],
      },
    };
    detail.record = identity;
    detail.creature_family = {
      id: "eltha",
      name: "Eltha",
      versions: [identity],
      named_creatures: [],
      named_bases: {},
    };
    detail.presentation = {
      classification: "Creature 11",
      identity,
      body: { kind: "creature", value: actorFixture() },
      content: [],
      embedded: [],
    };
    render(
      <RecordDetailPane
        detail={detail}
        loading={false}
        onReference={vi.fn()}
        showScenarioContext
      />,
      { wrapper: RouterTestContext },
    );
    expect(screen.getByRole("heading", { name: "Eltha" })).toBeVisible();
    const family = screen.getByRole("link", { name: "Eltha" });
    const firstUse = screen.getByText("Party levels 9–10 (Advanced)");
    expect(firstUse).toBeVisible();
    expect(
      family.compareDocumentPosition(firstUse) & Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
    expect(screen.getByText("Party levels 11–12")).toBeVisible();
    expect(screen.getByText("Cavern of Sundered Songs")).toBeVisible();
    expect(screen.queryByText(identity.title)).toBeNull();
  });
});

function recordDetailFixture(): RecordDetailView {
  return detailFixture("spell:dirge", "Dirge of Doom");
}
