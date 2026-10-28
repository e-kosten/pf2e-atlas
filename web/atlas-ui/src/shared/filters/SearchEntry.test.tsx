import { ConfigProvider } from "antd";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { detailFixture } from "../../test/fixtures";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { SearchEntry } from "./SearchEntry";

const api = vi.hoisted(() => ({ detail: vi.fn() }));
vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  getRecordDetail: api.detail,
}));
const seed = { kind: "record", record_key: "items:leather-armor" } as const;
function wrapper({ children }: { children: React.ReactNode }) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return (
    <QueryClientProvider client={client}>
      <ConfigProvider theme={{ token: { motion: false } }}>{children}</ConfigProvider>
    </QueryClientProvider>
  );
}

it("keeps keyword edits local until Enter, then restores incoming committed criteria", () => {
  const commit = vi.fn();
  const view = render(<SearchEntry query="fire" onSearch={commit} />);
  const input = screen.getByRole("searchbox", { name: "Search records" });
  fireEvent.change(input, { target: { value: "ice" } });
  expect(commit).not.toHaveBeenCalled();
  fireEvent.keyDown(input, { key: "Enter", keyCode: 13 });
  expect(commit).toHaveBeenCalledExactlyOnceWith("ice");
  view.rerender(<SearchEntry query="acid" onSearch={commit} />);
  expect(input).toHaveValue("acid");
});

it("inherits pending form disabling from the Ant provider", () => {
  render(
    <ConfigProvider componentDisabled>
      <SearchEntry query="fire" onSearch={vi.fn()} />
    </ConfigProvider>,
  );
  expect(screen.getByRole("searchbox")).toBeDisabled();
});

it("clears committed criteria when the last character is deleted", () => {
  const commit = vi.fn();
  render(<SearchEntry query="no matches" onSearch={commit} />);
  fireEvent.change(screen.getByRole("searchbox"), { target: { value: "" } });
  expect(commit).toHaveBeenCalledExactlyOnceWith("");
});

it("opens the seed reference through a native link and restores text-entry focus after Clear", async () => {
  api.detail.mockResolvedValue(detailFixture(seed.record_key, "Leather Armor"));
  const clear = vi.fn();
  const refresh = vi.fn();
  const commit = vi.fn();
  const view = render(
    <SearchEntry
      query=""
      onSearch={commit}
      similar={{ seed, title: "Leather Armor", onClear: clear, onSearch: refresh }}
    />,
    { wrapper },
  );
  const link = screen.getByRole("link", { name: "Leather Armor" });
  expect(link).toHaveAttribute("href", expect.stringContaining("leather-armor"));
  expect(screen.queryByRole("searchbox")).toBeNull();
  const modified = new MouseEvent("click", {
    bubbles: true,
    cancelable: true,
    metaKey: true,
  });
  link.dispatchEvent(modified);
  expect(modified.defaultPrevented).toBe(false);
  expect(api.detail).not.toHaveBeenCalled();
  fireEvent.click(link);
  expect(await screen.findByRole("dialog", { name: "Leather Armor" })).toBeVisible();
  fireEvent.click(screen.getByRole("button", { name: "Close reference preview" }));
  await waitFor(() => expect(link).toHaveFocus());
  fireEvent.click(screen.getByRole("button", { name: "Run similar search" }));
  expect(refresh).toHaveBeenCalledOnce();
  expect(commit).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button", { name: "Clear similar criterion" }));
  expect(clear).toHaveBeenCalledOnce();
  view.rerender(<SearchEntry query="" onSearch={commit} />);
  expect(screen.getByRole("searchbox")).toHaveFocus();
});

it("keeps trait seed links on the native trait destination", () => {
  render(
    <SearchEntry
      query=""
      onSearch={vi.fn()}
      similar={{
        seed: { kind: "trait", trait_key: "trait:fire" },
        title: "Fire",
        onClear: vi.fn(),
        onSearch: vi.fn(),
      }}
    />,
  );
  expect(screen.getByRole("link", { name: "Fire" })).toHaveAttribute(
    "href",
    "/traits/trait:fire",
  );
});
