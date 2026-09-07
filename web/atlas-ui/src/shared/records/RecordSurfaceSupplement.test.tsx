import { fireEvent, render, screen } from "@testing-library/react";
import type { RecordSurfaceView } from "../../generated/atlas";
import {
  RecordSurfaceIssues,
  RecordSurfaceReferences,
} from "./RecordSurfaceSupplement";

type References = NonNullable<RecordSurfaceView["references"]>;

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
    screen.getByText(/Atlas currently exposes up to 50 linked records/),
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
