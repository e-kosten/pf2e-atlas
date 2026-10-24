import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { App, ConfigProvider } from "antd";
import type { ReactNode } from "react";
import { AddToListButton } from "./AddToListButton";
import { CreateListModal } from "./CreateListModal";

const api = vi.hoisted(() => ({
  addSavedListItem: vi.fn(),
  createSavedList: vi.fn(),
  getSavedLists: vi.fn(),
}));
vi.mock("../../api/atlasApi", () => api);

const lists = {
  lists: [
    {
      list_key: "list_review",
      slug: "review",
      name: "Review",
      description: null,
      tags: [],
      item_count: 0,
      created_at: "2026-01-01",
      updated_at: "2026-01-01",
    },
  ],
};

function setup(ui: ReactNode) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false, staleTime: Infinity } },
  });
  render(
    <QueryClientProvider client={client}>
      <ConfigProvider theme={{ token: { motion: false } }}>
        <App>{ui}</App>
      </ConfigProvider>
    </QueryClientProvider>,
  );
  return client;
}

async function chooseList() {
  fireEvent.click(screen.getByRole("button", { name: "Add to saved list" }));
  const selector = await screen.findByRole("combobox", { name: "Saved list" });
  await waitFor(() => expect(selector).toBeEnabled());
  fireEvent.mouseDown(selector);
  fireEvent.click(await screen.findByText("Review", { exact: true }));
  return selector;
}

beforeEach(() => {
  for (const mock of Object.values(api)) mock.mockReset();
  api.getSavedLists.mockResolvedValue(lists);
});

it("keeps a new-list draft during one pending submission and a failed attempt", async () => {
  let reject!: (error: Error) => void;
  api.createSavedList.mockReturnValueOnce(
    new Promise((_yes, no) => {
      reject = no;
    }),
  );
  api.createSavedList.mockResolvedValueOnce({
    list: { ...lists.lists[0], slug: "expedition" },
  });
  const cancel = vi.fn(),
    created = vi.fn();
  setup(<CreateListModal open onCancel={cancel} onCreated={created} />);
  const name = screen.getByRole("textbox", { name: /Name$/ });
  fireEvent.change(name, { target: { value: "Expedition" } });
  const form = name.closest("form")!;
  fireEvent.submit(form);
  fireEvent.submit(form);
  await waitFor(() => expect(api.createSavedList).toHaveBeenCalledOnce());
  expect(name).toBeDisabled();
  expect(screen.getByRole("button", { name: "Cancel" })).toBeDisabled();
  expect(screen.getByRole("button", { name: "Close" })).toBeDisabled();
  fireEvent.keyDown(screen.getByRole("dialog"), { key: "Escape" });
  fireEvent.mouseDown(document.querySelector(".ant-modal-wrap")!);
  fireEvent.click(document.querySelector(".ant-modal-wrap")!);
  expect(cancel).not.toHaveBeenCalled();
  await act(async () => reject(new Error("Service unavailable")));
  expect(await screen.findByRole("alert")).toHaveTextContent("Service unavailable");
  expect(name).toHaveValue("Expedition");
  expect(name).toBeEnabled();
  expect(created).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button", { name: "Create" }));
  await waitFor(() => expect(created).toHaveBeenCalledWith("expedition"));
  expect(api.createSavedList).toHaveBeenCalledTimes(2);
});

it("keeps list choice and dialog through pending/failure, then closes after retry", async () => {
  let reject!: (error: Error) => void;
  api.addSavedListItem.mockReturnValueOnce(
    new Promise((_yes, no) => {
      reject = no;
    }),
  );
  api.addSavedListItem.mockResolvedValueOnce({
    list_key: "list_review",
    slug: "review",
  });
  setup(<AddToListButton recordKey="spells:light" />);
  const selector = await chooseList();
  const dialog = screen.getByRole("dialog", { name: "Add to List" });
  const add = within(dialog).getByRole("button", { name: "Add" });
  fireEvent.click(add);
  fireEvent.click(add);
  await waitFor(() => expect(api.addSavedListItem).toHaveBeenCalledOnce());
  expect(selector).toBeDisabled();
  expect(within(dialog).getByRole("button", { name: "Cancel" })).toBeDisabled();
  fireEvent.keyDown(dialog, { key: "Escape" });
  fireEvent.mouseDown(document.querySelector(".ant-modal-wrap")!);
  fireEvent.click(document.querySelector(".ant-modal-wrap")!);
  expect(dialog).toBeVisible();
  await act(async () => reject(new Error("Record could not be added")));
  expect(await within(dialog).findByRole("alert")).toHaveTextContent(
    "Record could not be added",
  );
  expect(selector.parentElement).toHaveTextContent("Review");
  expect(selector).toBeEnabled();
  fireEvent.click(add);
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  expect(api.addSavedListItem).toHaveBeenCalledTimes(2);
  expect(api.addSavedListItem).toHaveBeenLastCalledWith({
    list_ref: "review",
    record_ref: "spells:light",
  });
});

it("distinguishes an unavailable list catalog from empty and retries locally", async () => {
  api.getSavedLists
    .mockRejectedValueOnce(new Error("Service unavailable"))
    .mockResolvedValueOnce(lists);
  setup(<AddToListButton recordKey="spells:light" />);
  fireEvent.click(screen.getByRole("button", { name: "Add to saved list" }));
  expect(await screen.findByRole("alert")).toHaveTextContent(
    "Saved lists could not load",
  );
  expect(screen.queryByText(/No saved lists yet/)).toBeNull();
  expect(screen.getByRole("combobox", { name: "Saved list" })).toBeDisabled();
  expect(screen.getByRole("button", { name: "Add" })).toBeDisabled();
  fireEvent.click(screen.getByRole("button", { name: "Retry" }));
  await waitFor(() =>
    expect(screen.getByRole("combobox", { name: "Saved list" })).toBeEnabled(),
  );
  expect(screen.queryByRole("alert")).toBeNull();
});

it("retains choices and focused selector while reporting a failed catalog refresh", async () => {
  const client = setup(<AddToListButton recordKey="spells:light" />);
  const selector = await chooseList();
  act(() => selector.focus());
  api.getSavedLists.mockRejectedValueOnce(new Error("Disconnected"));
  await act(async () => {
    await client.invalidateQueries({ queryKey: ["saved-lists"] });
  });
  expect(await screen.findByRole("alert")).toHaveTextContent(
    "Saved lists could not refresh",
  );
  expect(screen.getByRole("combobox", { name: "Saved list" })).toBe(selector);
  expect(selector).toHaveFocus();
  expect(selector.parentElement).toHaveTextContent("Review");
  expect(screen.getByRole("button", { name: "Add" })).toBeEnabled();
});

it("gives a genuine empty catalog its existing saved-lists destination", async () => {
  api.getSavedLists.mockResolvedValueOnce({ lists: [] });
  setup(<AddToListButton recordKey="spells:light" />);
  fireEvent.click(screen.getByRole("button", { name: "Add to saved list" }));
  await waitFor(() => expect(screen.getByText(/No saved lists yet/)).toBeVisible());
  expect(screen.getByRole("link", { name: "Open saved lists" })).toHaveAttribute(
    "href",
    "/lists",
  );
  expect(screen.queryByRole("alert")).toBeNull();
  expect(screen.getByRole("button", { name: "Add" })).toBeDisabled();
});
