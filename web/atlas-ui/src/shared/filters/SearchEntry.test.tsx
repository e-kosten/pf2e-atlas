import { ReferenceHelpHost } from "../records/ReferenceHelpHost";
import { ConfigProvider } from "antd";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { detailFixture } from "../../test/fixtures";
import { RouterTestContext } from "../../test/router";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { SearchEntry } from "./SearchEntry";
import type { SearchSubmission } from "./filterControls";

const api = vi.hoisted(() => ({ detail: vi.fn(), definition: vi.fn() }));
vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  getRecordDetail: api.detail,
  getDefinitionDetail: api.definition,
}));
const seed = { kind: "record", record_key: "items:leather-armor" } as const;
function wrapper({ children }: { children: React.ReactNode }) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return (
    <QueryClientProvider client={client}>
      <ConfigProvider theme={{ token: { motion: false } }}>
        <RouterTestContext>
          <ReferenceHelpHost>{children}</ReferenceHelpHost>
        </RouterTestContext>
      </ConfigProvider>
    </QueryClientProvider>
  );
}

it("keeps keyword edits local until Enter, then restores incoming committed criteria", () => {
  const commit = vi.fn<(query: string, submission: SearchSubmission) => void>();
  const view = render(<SearchEntry query="fire" onSearch={commit} />);
  const input = screen.getByRole("searchbox", { name: "Search records" });
  fireEvent.change(input, { target: { value: "ice" } });
  expect(commit).not.toHaveBeenCalled();
  fireEvent.keyDown(input, { key: "Enter", keyCode: 13 });
  expect(commit).toHaveBeenCalledExactlyOnceWith("ice", expect.any(Object));
  expect(input).toHaveValue("ice");
  view.rerender(<SearchEntry query="acid" onSearch={commit} />);
  expect(input).toHaveValue("acid");
  view.rerender(<SearchEntry query="fire" onSearch={commit} />);
  expect(input).toHaveValue("fire");
});

it("keeps newer typing through an older acknowledgement and accepts a normalized repeat", () => {
  const commit = vi.fn<(query: string, submission: SearchSubmission) => void>();
  const view = render(<SearchEntry query="" onSearch={commit} />);
  const input = screen.getByRole("searchbox");
  fireEvent.change(input, { target: { value: "fire" } });
  fireEvent.keyDown(input, { key: "Enter", keyCode: 13 });
  fireEvent.keyUp(input, { key: "Enter", keyCode: 13 });
  const first = commit.mock.calls[0][1];
  fireEvent.change(input, { target: { value: " fire " } });
  fireEvent.keyDown(input, { key: "Enter", keyCode: 13 });
  const second = commit.mock.calls[1][1];

  view.rerender(
    <SearchEntry query="fire" onSearch={commit} acceptedSubmission={first} />,
  );
  expect(input).toHaveValue(" fire ");
  view.rerender(
    <SearchEntry query="fire" onSearch={commit} acceptedSubmission={second} />,
  );
  expect(input).toHaveValue("fire");
  expect(commit).toHaveBeenCalledTimes(2);
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
  expect(commit).toHaveBeenCalledExactlyOnceWith("", expect.any(Object));
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
