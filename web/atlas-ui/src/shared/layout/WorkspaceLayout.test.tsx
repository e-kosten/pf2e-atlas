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
