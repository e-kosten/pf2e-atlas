import { IndexTable } from "../../shared/ui/tables/IndexTable";
import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { ListTagSummary } from "./ListTagSummary";

function measuredTags(initialWidth: number) {
  let width = initialWidth;
  let resize!: ResizeObserverCallback;
  const OriginalObserver = globalThis.ResizeObserver;
  vi.stubGlobal(
    "ResizeObserver",
    class implements ResizeObserver {
      callback: ResizeObserverCallback;
      constructor(callback: ResizeObserverCallback) {
        this.callback = callback;
      }
      observe(target: Element) {
        if (target.classList.contains("list-tag-summary")) resize = this.callback;
      }
      unobserve() {}
      disconnect() {}
    },
  );
  const rect = vi
    .spyOn(HTMLElement.prototype, "getBoundingClientRect")
    .mockImplementation(function (this: HTMLElement) {
      const measuredWidth = this.classList.contains("list-tag-summary")
        ? width
        : this.hasAttribute("data-list-tag")
          ? 90
          : this.tagName === "BUTTON"
            ? 60
            : 0;
      return {
        x: 0,
        y: 0,
        left: 0,
        top: 0,
        right: measuredWidth,
        bottom: 24,
        width: measuredWidth,
        height: 24,
        toJSON() {},
      };
    });
  return {
    resize(next: number) {
      width = next;
      act(() => resize([], {} as ResizeObserver));
    },
    restore() {
      rect.mockRestore();
      vi.stubGlobal("ResizeObserver", OriginalObserver);
    },
  };
}

it("counts actual hidden tags, preserves whole chips and keeps portal clicks inside the summary through resize", async () => {
  const geometry = measuredTags(400);
  try {
    const activate = vi.fn();
    render(
      <IndexTable
        dataSource={[{ key: "review" }]}
        rowKey={(row) => row.key}
        onActivateRow={activate}
        columns={[
          {
            title: "Tags",
            render: () => <ListTagSummary tags={["Campaign", "Boss", "Planar"]} />,
          },
        ]}
      />,
    );
    expect(screen.getByText("Planar")).toBeVisible();
    expect(screen.queryByRole("button", { name: /more/ })).toBeNull();
    geometry.resize(200);
    expect(screen.getByText("Campaign")).toBeVisible();
    expect(screen.getByText("Boss")).not.toBeVisible();
    // Ant’s default transition:all would keep absolute measurement chips
    // visibly stacked until the visibility transition ended.
    expect(screen.getByText("Boss")).toHaveStyle({ transitionProperty: "none" });
    expect(screen.getByText("Planar")).not.toBeVisible();
    const trigger = screen.getByRole("button", { name: "+2 more" });
    fireEvent.click(trigger);
    const all = await screen.findByLabelText("All list tags");
    // Mocked geometry exercises admission and portal ownership; native overlay
    // visibility and the inline position are independent browser obligations.
    expect(within(all).getByText("Planar")).toBeInTheDocument();
    fireEvent.click(within(all).getByText("Planar"));
    expect(activate).not.toHaveBeenCalled();
    act(() => trigger.focus());
    fireEvent.keyDown(trigger, { key: "Escape" });
    expect(trigger).toHaveFocus();
    // Full row fit is independent of the currently mounted disclosure width.
    geometry.resize(280);
    await waitFor(() =>
      expect(screen.queryByRole("button", { name: /more/ })).toBeNull(),
    );
    expect(screen.getAllByText("Planar")[0]).toBeVisible();
  } finally {
    geometry.restore();
  }
});

it("uses two compact detail lines while the same width retains one index line", () => {
  const geometry = measuredTags(200);
  const tags = ["One", "Two", "Three", "Four", "Five", "Six"];
  try {
    const view = render(<ListTagSummary tags={tags} lines={2} />);
    expect(screen.getByText("Three")).toBeVisible();
    expect(screen.getByText("Four")).not.toBeVisible();
    expect(screen.getByRole("button", { name: "+3 more" })).toBeVisible();
    geometry.resize(280);
    expect(screen.getByText("Six")).toBeVisible();
    expect(screen.queryByRole("button", { name: /more/ })).toBeNull();
    geometry.resize(200);
    view.rerender(<ListTagSummary tags={tags} lines={1} />);
    expect(screen.getByRole("button", { name: "+5 more" })).toBeVisible();
    expect(screen.getByText("Two")).not.toBeVisible();
  } finally {
    geometry.restore();
  }
});
