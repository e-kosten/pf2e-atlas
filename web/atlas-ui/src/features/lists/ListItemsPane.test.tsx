import { App, ConfigProvider } from "antd";
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import type { SavedListItemView } from "../../generated/atlas";
import { summaryFixture } from "../../test/fixtures";
import { ListItemsPane } from "./ListItemsPane";

const active = (index: number): SavedListItemView => ({
  record_key: `rules:record-${index}`,
  position: index,
  status: "active",
  snapshot: { title: `Saved record ${index}`, kind: "rule" },
  record: summaryFixture(`rules:record-${index}`, `Saved record ${index}`),
});
const unresolved: SavedListItemView = {
  record_key: "rules:unavailable",
  position: 120,
  status: "unresolved",
  snapshot: { title: "Retained unavailable record", kind: "rule" },
};
function mount(items: SavedListItemView[], onRemove = vi.fn(), onSelect = vi.fn()) {
  const view = render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <App>
        <div className="reference-results">
          <ListItemsPane
            items={items}
            loading={false}
            selectedRecordKey={null}
            removingKey={null}
            onRemove={onRemove}
            onSelect={onSelect}
          />
        </div>
      </App>
    </ConfigProvider>,
  );
  return { ...view, onRemove, onSelect };
}

it("keeps 120 ordered member summaries and unavailable snapshots without radio or guessed links", () => {
  const items = [
    ...Array.from({ length: 120 }, (_, index) => active(index)),
    unresolved,
  ];
  const { container } = mount(items);
  const links = screen.getAllByRole("link");
  expect(links).toHaveLength(120);
  expect(links[0]).toHaveAttribute("href", "/records/rules:record-0");
  expect(links[119]).toHaveAttribute("href", "/records/rules:record-119");
  expect(screen.queryByRole("radio")).not.toBeInTheDocument();
  expect(
    screen.getByText("Retained unavailable record").closest("[aria-disabled]"),
  ).toHaveAttribute("aria-disabled", "true");
  expect(container.querySelector('a[href*="unavailable"]')).toBeNull();
  expect(
    screen.getByRole("button", { name: "Remove Retained unavailable record" }),
  ).toBeEnabled();
});

it("shares deliberate preview, native modified navigation, Enter and keyboard focus with search", async () => {
  const { onSelect } = mount([active(0), active(1)]);
  const first = screen.getByRole("link", { name: /Saved record 0/ });
  const second = screen.getByRole("link", { name: /Saved record 1/ });
  fireEvent.click(first, { metaKey: true });
  expect(onSelect).not.toHaveBeenCalled();
  fireEvent.keyDown(first, { key: " ", ctrlKey: true });
  expect(onSelect).not.toHaveBeenCalled();
  fireEvent.keyDown(first, { key: " " });
  expect(onSelect).toHaveBeenLastCalledWith("rules:record-0");
  fireEvent.keyDown(first, { key: "j" });
  expect(second).toHaveFocus();
  expect(onSelect).toHaveBeenCalledTimes(1);
  fireEvent.keyDown(second, { key: "Enter" });
  await waitFor(() => expect(window.location.pathname).toBe("/records/rules:record-1"));
  expect(onSelect).toHaveBeenCalledTimes(1);
});

it("allows cancellation and awaits failed member removal in one retryable confirmation without losing the snapshot", async () => {
  let reject!: (error: Error) => void;
  const onRemove = vi
    .fn()
    .mockReturnValueOnce(
      new Promise<void>((_resolve, fail) => {
        reject = fail;
      }),
    )
    .mockResolvedValue(undefined);
  mount([unresolved], onRemove);
  const trigger = screen.getByRole("button", {
    name: "Remove Retained unavailable record",
  });
  trigger.focus();
  fireEvent.click(trigger);
  fireEvent.click(await screen.findByRole("button", { name: "Cancel" }));
  await waitFor(() => expect(screen.queryByRole("tooltip")).not.toBeInTheDocument());
  expect(onRemove).not.toHaveBeenCalled();
  expect(trigger).toHaveFocus();
  fireEvent.click(trigger);
  fireEvent.click(await screen.findByRole("button", { name: "Remove" }));
  await waitFor(() => expect(onRemove).toHaveBeenCalledWith("rules:unavailable"));
  expect(screen.getByRole("button", { name: "Cancel" })).toBeDisabled();
  expect(screen.getByText("Retained unavailable record")).toBeVisible();
  await act(async () => reject(new Error("Removal unavailable")));
  await waitFor(() =>
    expect(screen.getByText("Unable to remove. Try again.")).toBeVisible(),
  );
  expect(screen.getByRole("tooltip")).toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "Remove" }));
  await waitFor(() => expect(screen.queryByRole("tooltip")).not.toBeInTheDocument());
  expect(onRemove).toHaveBeenCalledTimes(2);
});
