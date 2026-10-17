import { fireEvent, render, screen } from "@testing-library/react";
import { WorkspaceLayout } from "./WorkspaceLayout";

it("keeps pane identity through collapse and restores detail for a new selection", () => {
  const content = {
    filter: <aside className="filter-panel">Criteria</aside>,
    results: <section className="results-panel">Rows</section>,
    detail: <section className="detail-panel">Record</section>,
  };
  const { container, rerender } = render(
    <WorkspaceLayout {...content} selectedRecordKey="items:first" />,
  );
  const detail = container.querySelector(".workspace-pane--detail");
  expect(detail?.parentElement).toHaveClass("workspace-grid");
  fireEvent.click(screen.getByRole("button", { name: "Collapse Detail" }));
  expect(detail).toHaveClass("workspace-pane--collapsed");
  expect(detail).not.toHaveTextContent("Record");
  rerender(<WorkspaceLayout {...content} selectedRecordKey="items:second" />);
  expect(detail).not.toHaveClass("workspace-pane--collapsed");
  expect(detail).toHaveTextContent("Record");

  fireEvent.click(screen.getByRole("button", { name: "Collapse Filters" }));
  expect(container.querySelector(".workspace-pane--filter")).toHaveClass(
    "workspace-pane--collapsed",
  );
  fireEvent.click(screen.getAllByRole("button", { name: "Expand Filters" })[0]);
  expect(container.querySelector(".workspace-pane--filter")).not.toHaveClass(
    "workspace-pane--collapsed",
  );
});

it("keeps a noncollapsible participant visible while the roster collapses", () => {
  const { container } = render(
    <WorkspaceLayout
      filter={<div>Roster rows</div>}
      results={<div>Participant content</div>}
      selectedRecordKey={null}
      labels={{ filter: "Roster", results: "Participant" }}
      collapsiblePanes={["filter"]}
    />,
  );
  expect(screen.queryByRole("button", { name: "Collapse Participant" })).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Collapse Roster" }));
  expect(screen.getByText("Participant content")).toBeVisible();
  expect(container.querySelector(".workspace-pane--results")).not.toHaveClass(
    "workspace-pane--collapsed",
  );
  expect(container.querySelector(".workspace-grid")).toHaveStyle({
    "--pane-columns": "44px var(--panel-gap) minmax(0, 1fr)",
  });
});
