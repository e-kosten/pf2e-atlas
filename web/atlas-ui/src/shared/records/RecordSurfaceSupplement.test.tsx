import { fireEvent, render, screen } from "@testing-library/react";
import type { RecordSurfaceView } from "../../generated/atlas";
import {
  RecordSurfaceIssues,
  RecordSurfaceReferences,
} from "./RecordSurfaceSupplement";

type References = NonNullable<RecordSurfaceView["references"]>;
type ReferenceEdge = Extract<
  References["outgoing"],
  { state: "available" }
>["edges"][number];

it("groups a semantic subject while retaining every distinct fact", () => {
  const subject = { label: "Door mechanism" };
  const { container } = render(
    <RecordSurfaceIssues
      issues={[
        {
          fact_id: "first",
          code: "unsupported",
          placement: "record",
          subject,
          fact_label: "Source detail",
          message: "Unsupported authored detail.",
        },
        {
          fact_id: "second",
          code: "unsupported",
          placement: "record",
          subject,
          fact_label: "Source detail",
          message: "Unsupported authored detail.",
        },
      ]}
    />,
  );
  expect(screen.getAllByRole("alert")).toHaveLength(1);
  expect(container.querySelectorAll("[data-fact-id]")).toHaveLength(2);
  expect(screen.getAllByText("Unsupported authored detail.")).toHaveLength(2);
  expect(screen.queryByText("first")).not.toBeInTheDocument();
});

it("requests the server next limit and exposes an honest terminal cap", () => {
  const request = vi.fn();
  const section: References["outgoing"] = {
    state: "available",
    requested_limit: 8,
    next_limit: 16,
    records: [],
    edges: [],
    total_records: 65,
    total_edges: 72,
    truncated: true,
  };
  const { rerender } = render(
    <RecordSurfaceReferences
      references={{ outgoing: section, backlinks: { state: "not_requested" } }}
      onReference={vi.fn()}
      onRequestLimit={request}
    />,
  );
  fireEvent.click(screen.getByRole("button", { name: "References" }));
  fireEvent.click(screen.getByRole("button", { name: "View more" }));
  expect(request).toHaveBeenCalledWith("outgoing", 16);
  rerender(
    <RecordSurfaceReferences
      references={{
        outgoing: { ...section, requested_limit: 50, next_limit: undefined },
        backlinks: { state: "not_requested" },
      }}
      onReference={vi.fn()}
      onRequestLimit={request}
    />,
  );
  expect(screen.queryByRole("button", { name: "View more" })).not.toBeInTheDocument();
  expect(
    screen.getByText(/This panel shows up to 50 linked records/),
  ).toBeInTheDocument();
  expect(screen.queryByRole("button", { name: /next page/i })).not.toBeInTheDocument();
});

it("offers a retry for the failed reference direction", () => {
  const request = vi.fn();
  render(
    <RecordSurfaceReferences
      references={{
        outgoing: {
          state: "unavailable",
          requested_limit: 16,
          code: "internal_error",
          message: "Linked records could not be loaded.",
        },
        backlinks: { state: "not_requested" },
      }}
      onReference={vi.fn()}
      onRequestLimit={request}
    />,
  );
  fireEvent.click(screen.getByRole("button", { name: "References" }));
  fireEvent.click(screen.getByRole("button", { name: "Retry" }));
  expect(request).toHaveBeenCalledWith("outgoing", 16);
});

it("keeps large issue groups compact while exposing every retained fact", () => {
  const { container } = render(
    <RecordSurfaceIssues
      issues={Array.from({ length: 20 }, (_, index) => ({
        fact_id: `fact:${index}`,
        code: "unsupported",
        placement: "activity",
        subject: { label: "Door mechanism" },
        fact_label: "Source detail",
        message: "Authored detail is unavailable.",
      }))}
    />,
  );
  expect(screen.getAllByRole("alert")).toHaveLength(1);
  fireEvent.click(screen.getByRole("button", { name: "20 affected facts" }));
  expect(container.querySelectorAll("[data-fact-id]")).toHaveLength(20);
});

it.each(["backlinks", "outgoing"] as const)(
  "opens exact %s search beyond the panel cap",
  (direction) => {
    history.replaceState(null, "", "/records/spells:seed?q=wrong-name");
    const section: References["outgoing"] = {
      state: "available",
      requested_limit: 50,
      records: [],
      edges: [],
      total_records: 61,
      total_edges: 122,
      truncated: true,
    };
    render(
      <RecordSurfaceReferences
        recordKey="spells:seed"
        onReference={vi.fn()}
        references={{
          outgoing: { state: "not_requested" },
          backlinks: { state: "not_requested" },
          [direction]: section,
        }}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: "References" }));
    fireEvent.click(
      screen.getByRole("button", {
        name:
          direction === "backlinks"
            ? "See all referencing records"
            : "See all referenced records",
      }),
    );
    const action = screen.getByRole("button", {
      name:
        direction === "backlinks"
          ? "See all referencing records"
          : "See all referenced records",
    });
    expect(action).toHaveTextContent("Search all");
    expect(action.closest(".record-surface-references__heading")).not.toBeNull();
    const params = new URLSearchParams(location.search);
    expect(location.pathname).toBe("/search");
    expect(params.get("reference-record")).toBe("spells:seed");
    expect(params.get("reference-direction")).toBe(
      direction === "backlinks" ? "incoming" : "outgoing",
    );
    expect(params.has("q")).toBe(false);
  },
);

it.each(["outgoing", "backlinks"] as const)(
  "renders each %s child edge plus a coexisting parent target",
  (direction) => {
    const onReference = vi.fn();
    const record = {
      record_key: "spells:linked",
      title: "Linked spell",
      kind: "spell",
    };
    const edge = (displayText: string, childLocator?: string): ReferenceEdge => ({
      from_record_key: direction === "outgoing" ? "spells:seed" : record.record_key,
      to_record_key: direction === "outgoing" ? record.record_key : "spells:seed",
      ...(childLocator
        ? direction === "outgoing"
          ? { target_child_locator: childLocator }
          : { source_child_locator: childLocator }
        : {}),
      display_text: displayText,
      reference_text: displayText,
      source: {
        kind: "description",
        visibility: "public",
        relation_kind: "reference",
      },
    });
    const section: References["outgoing"] = {
      state: "available",
      requested_limit: 8,
      records: [record],
      edges: [
        edge("First child", "v1~j~s~6669727374"),
        edge("Second child", "v1~j~s~7365636f6e64"),
        edge("Parent only"),
      ],
      total_records: 1,
      total_edges: 3,
      truncated: false,
    };
    const empty: References["outgoing"] = {
      state: "available",
      requested_limit: 8,
      records: [],
      edges: [],
      total_records: 0,
      total_edges: 0,
      truncated: false,
    };
    render(
      <RecordSurfaceReferences
        onReference={onReference}
        references={{ outgoing: empty, backlinks: empty, [direction]: section }}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: "References" }));
    expect(screen.getAllByText("Linked spell")).toHaveLength(1);
    expect(screen.getByText("1 record")).toBeVisible();
    expect(screen.getByText("3 references")).toBeVisible();
    const first = screen.getByRole("button", {
      name: "Open Linked spell: First child",
    });
    const second = screen.getByRole("button", {
      name: "Open Linked spell: Second child",
    });
    expect(
      first.compareDocumentPosition(second) & Node.DOCUMENT_POSITION_FOLLOWING,
    ).not.toBe(0);
    fireEvent.click(first);
    fireEvent.click(second);
    fireEvent.click(screen.getByRole("button", { name: "Linked spell" }));
    expect(onReference).toHaveBeenNthCalledWith(
      1,
      record.record_key,
      "v1~j~s~6669727374",
    );
    expect(onReference).toHaveBeenNthCalledWith(
      2,
      record.record_key,
      "v1~j~s~7365636f6e64",
    );
    expect(onReference).toHaveBeenNthCalledWith(3, record.record_key);
    expect(screen.queryByText(/Showing/)).not.toBeInTheDocument();
  },
);

it("shows a shared consequence once without merging distinct ordinal/shape facts", () => {
  const consequence =
    "Authored content remains readable; exact values are in Source & provenance.";
  const { container } = render(
    <RecordSurfaceIssues
      issues={[
        {
          fact_id: "a",
          code: "unsupported",
          placement: "record",
          fact_label: "Authored field",
          message: "Additional authored fact 1 contains text.",
          consequence,
        },
        {
          fact_id: "b",
          code: "unsupported",
          placement: "record",
          fact_label: "Authored field",
          message: "Additional authored fact 2 contains a list.",
          consequence,
        },
      ]}
    />,
  );
  expect(screen.getAllByRole("alert")).toHaveLength(1);
  expect(screen.getAllByText(consequence)).toHaveLength(1);
  expect(screen.getByText("Additional authored fact 1 contains text.")).toBeVisible();
  expect(screen.getByText("Additional authored fact 2 contains a list.")).toBeVisible();
  expect(container.querySelectorAll("[data-fact-id]")).toHaveLength(2);
});

it.each([
  { total: 1, edges: 1, expected: "1 record" },
  { total: 2, edges: 3, expected: "2 records" },
])("keeps complete reference counts compact: %j", ({ total, edges, expected }) => {
  const records = Array.from({ length: total }, (_, index) => ({
    record_key: `spells:r${index}`,
    title: `Spell ${index}`,
    kind: "spell",
  }));
  render(
    <RecordSurfaceReferences
      onReference={vi.fn()}
      references={{
        outgoing: {
          state: "available",
          requested_limit: 8,
          total_records: total,
          total_edges: edges,
          truncated: false,
          records,
          edges: [],
        },
        backlinks: { state: "not_requested" },
      }}
    />,
  );
  fireEvent.click(screen.getByRole("button", { name: "References" }));
  expect(screen.getByText(expected)).toBeVisible();
  expect(screen.queryByText(/Showing/)).not.toBeInTheDocument();
  if (total === edges)
    expect(screen.queryByText("1 reference")).not.toBeInTheDocument();
  else expect(screen.getByText("3 references")).toBeVisible();
});
