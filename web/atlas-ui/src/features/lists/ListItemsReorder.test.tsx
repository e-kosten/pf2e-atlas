import { App, ConfigProvider } from "antd";
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import type { ComponentProps } from "react";
import type { SavedListItemView } from "../../generated/atlas";
import { ListItemsPane } from "./ListItemsPane";

const items: SavedListItemView[] = ["First", "Unavailable", "Last"].map(
  (title, index) => ({
    record_key: `rules:item${index}`,
    position: index + 1,
    status: "unresolved",
    snapshot: { title },
  }),
);
const props = {
  items,
  loading: false,
  selectedRecordKey: null,
  removingKey: null,
  onRemove: vi.fn(),
  onSelect: vi.fn(),
};
function pane(extra: Partial<ComponentProps<typeof ListItemsPane>>) {
  return (
    <ConfigProvider theme={{ token: { motion: false } }}>
      <App>
        <ListItemsPane {...props} {...extra} />
      </App>
    </ConfigProvider>
  );
}
function order(container: HTMLElement) {
  return [...container.querySelectorAll<HTMLElement>("[data-sortable-key]")].map(
    (node) => node.dataset.sortableKey,
  );
}
async function keyboardDrop(name = "First", arrow = "ArrowDown") {
  const handle = screen.getByRole("button", { name: `Reorder ${name}` });
  handle.focus();
  fireEvent.keyDown(handle, { code: "Space", key: " " });
  await act(async () => {
    await vi.advanceTimersByTimeAsync(1);
  });
  fireEvent.keyDown(document, { code: arrow, key: arrow });
  await act(async () => {
    await vi.advanceTimersByTimeAsync(1);
  });
  fireEvent.keyDown(document, { code: "Space", key: " " });
}

beforeEach(() => {
  vi.useFakeTimers({ shouldAdvanceTime: true });
  vi.spyOn(HTMLElement.prototype, "getBoundingClientRect").mockImplementation(function (
    this: HTMLElement,
  ) {
    const siblings = this.parentElement?.querySelectorAll("[data-sortable-key]");
    const index = siblings ? [...siblings].indexOf(this) : 0;
    const top = Math.max(index, 0) * 100;
    return {
      x: 0,
      y: top,
      top,
      bottom: top + 80,
      left: 0,
      right: 300,
      width: 300,
      height: 80,
      toJSON() {},
    };
  });
});
afterEach(() => {
  vi.useRealTimers();
  vi.restoreAllMocks();
});

it("retains the dropped order during a slow save and refresh, blocks another move, and accepts canonical order", async () => {
  let complete!: (value: boolean) => void;
  const onReorder = vi.fn(
    () =>
      new Promise<boolean>((resolve) => {
        complete = resolve;
      }),
  );
  const view = render(pane({ onReorder }));
  await keyboardDrop();
  await waitFor(() =>
    expect(onReorder).toHaveBeenCalledWith("rules:item0", "rules:item1", "after"),
  );
  expect(order(view.container)).toEqual(["rules:item1", "rules:item0", "rules:item2"]);
  expect(screen.getByRole("button", { name: "Reorder Last" })).toBeDisabled();
  expect(screen.getByRole("button", { name: "Remove Last" })).toBeDisabled();
  view.rerender(pane({ onReorder, loading: true, mutating: true }));
  expect(order(view.container)).toEqual(["rules:item1", "rules:item0", "rules:item2"]);
  await act(async () => {
    complete(true);
  });
  const canonical = [items[1], items[0], items[2]].map((item, index) => ({
    ...item,
    position: index + 1,
  }));
  view.rerender(pane({ onReorder, items: canonical }));
  await act(async () => {
    await vi.advanceTimersByTimeAsync(32);
  });
  expect(order(view.container)).toEqual(["rules:item1", "rules:item0", "rules:item2"]);
  expect(screen.getByRole("button", { name: "Reorder First" })).toBeEnabled();
  expect(screen.getByRole("button", { name: "Reorder First" })).toHaveFocus();
  expect(onReorder).toHaveBeenCalledOnce();
});

it("restores canonical order after rejection and permits a fresh retry", async () => {
  const onReorder = vi
    .fn()
    .mockRejectedValueOnce(new Error("write failed"))
    .mockResolvedValue(true);
  const view = render(pane({ onReorder }));
  await keyboardDrop();
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Reorder First" })).toBeEnabled(),
  );
  expect(order(view.container)).toEqual(items.map((item) => item.record_key));
  await keyboardDrop();
  await waitFor(() => expect(onReorder).toHaveBeenCalledTimes(2));
  expect(order(view.container)).toEqual(["rules:item1", "rules:item0", "rules:item2"]);
});

it("cancels without writing and hides reorder under active filters", async () => {
  const onReorder = vi.fn().mockResolvedValue(true);
  const view = render(pane({ onReorder }));
  const handle = screen.getByRole("button", { name: "Reorder First" });
  handle.focus();
  fireEvent.keyDown(handle, { code: "Space", key: " " });
  await act(async () => {
    await vi.advanceTimersByTimeAsync(1);
  });
  fireEvent.keyDown(document, { code: "Escape", key: "Escape" });
  expect(onReorder).not.toHaveBeenCalled();
  expect(order(view.container)).toEqual(items.map((item) => item.record_key));
  view.rerender(pane({ onReorder, filtered: true }));
  expect(screen.queryByRole("button", { name: /^Reorder/ })).not.toBeInTheDocument();
  expect(screen.getByText("Unavailable")).toBeVisible();
});
