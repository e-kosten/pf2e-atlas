import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { App, ConfigProvider } from "antd";
import type { ReactNode } from "react";
import { ListEntryNote } from "./ListEntryNote";

const api = vi.hoisted(() => ({
  getSavedListItemNote: vi.fn(),
  updateSavedListItemNote: vi.fn(),
}));
vi.mock("../../api/atlasApi", () => api);
const entry = {
  list_key: "list_research",
  slug: "research",
  record_key: "actions:one",
  title: "One",
  note: "A useful ally",
};
function setup(initialNote?: typeof entry) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  const wrapper = ({ children }: { children: ReactNode }) => (
    <ConfigProvider theme={{ token: { motion: false } }}>
      <App>
        <QueryClientProvider client={client}>{children}</QueryClientProvider>
      </App>
    </ConfigProvider>
  );
  return {
    client,
    ...render(
      <ListEntryNote
        listRef="list_research"
        recordKey="actions:one"
        initialNote={initialNote}
        initialNoteUpdatedAt={1}
      />,
      { wrapper },
    ),
  };
}
beforeEach(() => {
  vi.resetAllMocks();
  api.getSavedListItemNote.mockResolvedValue(entry);
  api.updateSavedListItemNote.mockImplementation((request) =>
    Promise.resolve({ ...entry, note: request.note }),
  );
});

it("shows Note and Edit, preserves drafts through refresh and hidden reference reading, and restores focus on Cancel", async () => {
  const view = setup();
  expect(await screen.findByText("A useful ally")).toBeVisible();
  expect(screen.getByText("Note")).toBeVisible();
  fireEvent.click(screen.getByRole("button", { name: "Edit" }));
  fireEvent.change(screen.getByRole("textbox", { name: "Note" }), {
    target: { value: "Unfinished thought" },
  });
  await act(async () => {
    view.client.setQueryData(["saved-list-item-note", "list_research", "actions:one"], {
      ...entry,
      note: "A newer saved note",
    });
  });
  expect(screen.getByRole("textbox")).toHaveValue("Unfinished thought");
  view.rerender(
    <ListEntryNote listRef="list_research" recordKey="actions:one" hidden />,
  );
  expect(screen.queryByRole("textbox")).not.toBeInTheDocument();
  view.rerender(<ListEntryNote listRef="list_research" recordKey="actions:one" />);
  expect(screen.getByRole("textbox")).toHaveValue("Unfinished thought");
  fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Edit" })).toHaveFocus(),
  );
  expect(screen.getByText("A newer saved note")).toBeVisible();
  expect(api.updateSavedListItemNote).not.toHaveBeenCalled();
});

it("retains the draft on failure, retries, and clears a blank note without losing focus", async () => {
  setup();
  fireEvent.click(await screen.findByRole("button", { name: "Edit" }));
  fireEvent.change(screen.getByRole("textbox"), { target: { value: "New purpose" } });
  api.updateSavedListItemNote.mockRejectedValueOnce(new Error("Write failed"));
  fireEvent.click(screen.getByRole("button", { name: "Save" }));
  expect(await screen.findByText("Unable to save note")).toBeVisible();
  expect(screen.getByRole("textbox")).toHaveValue("New purpose");
  expect(api.updateSavedListItemNote).toHaveBeenCalledExactlyOnceWith({
    list_ref: "list_research",
    record_key: "actions:one",
    note: "New purpose",
  });
  // Keep the refreshed server value aligned with the successful mutation.
  api.getSavedListItemNote.mockResolvedValue({ ...entry, note: "New purpose" });
  fireEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Edit" })).toHaveFocus(),
  );
  expect(screen.getByText("New purpose")).toBeVisible();
  fireEvent.click(screen.getByRole("button", { name: "Edit" }));
  fireEvent.change(screen.getByRole("textbox"), { target: { value: "  " } });
  api.getSavedListItemNote.mockResolvedValue({ ...entry, note: null });
  fireEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Add note" })).toHaveFocus(),
  );
  expect(api.updateSavedListItemNote).toHaveBeenLastCalledWith({
    list_ref: "list_research",
    record_key: "actions:one",
    note: null,
  });
});

it("isolates late mutation completions from a different selected entry", async () => {
  let resolve!: (value: typeof entry) => void;
  api.updateSavedListItemNote.mockReturnValue(
    new Promise((yes) => {
      resolve = yes;
    }),
  );
  const view = setup();
  fireEvent.click(await screen.findByRole("button", { name: "Edit" }));
  fireEvent.change(screen.getByRole("textbox"), {
    target: { value: "Old entry draft" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Cancel" })).toBeDisabled(),
  );
  api.getSavedListItemNote.mockResolvedValue({
    ...entry,
    record_key: "actions:two",
    note: "Second entry",
  });
  view.rerender(
    <ListEntryNote key="two" listRef="list_research" recordKey="actions:two" />,
  );
  expect(await screen.findByText("Second entry")).toBeVisible();
  await act(async () => resolve({ ...entry, note: "Old entry draft" }));
  expect(screen.getByText("Second entry")).toBeVisible();
  expect(screen.queryByText("Old entry draft")).not.toBeInTheDocument();
});

it("shows the selected member's note immediately and retains it through refresh failure and retry", async () => {
  api.getSavedListItemNote.mockRejectedValueOnce(new Error("Read failed"));
  setup(entry);
  expect(screen.getByText(entry.note)).toBeVisible();
  expect(screen.getByRole("button", { name: "Edit" })).toBeVisible();
  expect(await screen.findByText("Unable to refresh note")).toBeVisible();
  expect(screen.getByText(entry.note)).toBeVisible();
  api.getSavedListItemNote.mockResolvedValue({ ...entry, note: "Refreshed note" });
  fireEvent.click(screen.getByRole("button", { name: "Retry" }));
  expect(await screen.findByText("Refreshed note")).toBeVisible();
  expect(screen.queryByText("Unable to refresh note")).not.toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "Edit" }));
  expect(screen.getByRole("textbox", { name: "Note" })).toHaveValue("Refreshed note");
});
