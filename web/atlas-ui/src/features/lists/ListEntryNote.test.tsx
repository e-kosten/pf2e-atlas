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
import { AtlasApiError } from "../../api/apiError";
import { ListEntryNoteEditor, useListEntryNote } from "./ListEntryNote";

function ListEntryNote({
  hidden,
  ...entry
}: NonNullable<Parameters<typeof useListEntryNote>[0]["selected"]> & {
  hidden?: boolean;
}) {
  const state = useListEntryNote({
    selected: entry,
    selectedRecordKey: entry.recordKey,
  });
  return <ListEntryNoteEditor state={state} hidden={hidden} />;
}

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
  api.updateSavedListItemNote.mockImplementation((_listRef, _recordKey, request) =>
    Promise.resolve({ ...entry, note: request.note.value }),
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
  expect(api.updateSavedListItemNote).toHaveBeenCalledExactlyOnceWith(
    "list_research",
    "actions:one",
    {
      note: { expected: entry.note, value: "New purpose" },
    },
  );
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
  expect(api.updateSavedListItemNote).toHaveBeenLastCalledWith(
    "list_research",
    "actions:one",
    {
      note: { expected: "New purpose", value: null },
    },
  );
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
  view.rerender(<ListEntryNote listRef="list_research" recordKey="actions:two" />);
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

const conflict = new AtlasApiError(409, "Conflicting note", {
  code: "saved_edit_conflict",
  message: "Conflicting note",
  details: { fields: ["note"] },
});

it("overwrites against the reviewed saved note and reopens review after another conflict", async () => {
  const { client } = setup();
  fireEvent.click(await screen.findByRole("button", { name: "Edit" }));
  fireEvent.change(screen.getByRole("textbox", { name: "Note" }), {
    target: { value: "My plan" },
  });
  await act(async () => {
    client.setQueryData(["saved-list-item-note", "list_research", "actions:one"], {
      ...entry,
      note: "First saved change",
    });
  });
  api.getSavedListItemNote.mockRejectedValueOnce(new Error("Read failed"));
  api.updateSavedListItemNote.mockRejectedValueOnce(conflict);
  fireEvent.click(screen.getByRole("button", { name: "Save" }));

  expect(
    await screen.findByRole("button", { name: "Retry loading saved note" }),
  ).toBeVisible();
  const dialog = screen.getByRole("dialog", { name: "Review note conflict" });
  const footer = screen.getByRole("region", { name: "List entry note" });
  await waitFor(() =>
    expect(within(dialog).getByText("Review note conflict")).toHaveFocus(),
  );
  fireEvent.click(within(dialog).getByRole("button", { name: "Back to note" }));
  const reopen = within(footer).getByRole("button", { name: "Review conflict" });
  await waitFor(() => expect(reopen).toHaveFocus());
  expect(within(footer).getByRole("textbox", { name: "Note" })).toHaveValue("My plan");
  expect(
    within(footer).queryByRole("button", { name: "Save" }),
  ).not.toBeInTheDocument();
  fireEvent.click(reopen);
  expect(api.updateSavedListItemNote).toHaveBeenLastCalledWith(
    "list_research",
    "actions:one",
    {
      note: { expected: entry.note, value: "My plan" },
    },
  );

  api.getSavedListItemNote.mockResolvedValue({
    ...entry,
    slug: "renamed",
    note: "First saved change",
  });
  fireEvent.click(
    await screen.findByRole("button", { name: "Retry loading saved note" }),
  );
  const overwrite = await screen.findByRole("button", { name: "Overwrite" });
  expect(screen.getByText("First saved change")).toBeVisible();
  expect(api.getSavedListItemNote).toHaveBeenLastCalledWith(
    "list_research",
    "actions:one",
  );
  expect(api.updateSavedListItemNote).toHaveBeenCalledTimes(1);

  api.getSavedListItemNote.mockResolvedValue({ ...entry, note: "Second saved change" });
  api.updateSavedListItemNote.mockRejectedValueOnce(conflict);
  fireEvent.click(overwrite);
  expect(await screen.findByText("Second saved change")).toBeVisible();
  expect(api.updateSavedListItemNote).toHaveBeenLastCalledWith(
    "list_research",
    "actions:one",
    {
      note: { expected: "First saved change", value: "My plan" },
    },
  );
  fireEvent.click(screen.getByRole("button", { name: "Use saved" }));
  expect(screen.queryByRole("textbox", { name: "Note" })).not.toBeInTheDocument();
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Edit" })).toHaveFocus(),
  );
  expect(api.updateSavedListItemNote).toHaveBeenCalledTimes(2);
});

it.each([
  { text: "Same edit", saved: "Same edit" },
  { text: "   ", saved: null },
])(
  "finishes editing without review when the saved note already matches $saved",
  async ({ text, saved }) => {
    setup();
    fireEvent.click(await screen.findByRole("button", { name: "Edit" }));
    fireEvent.change(screen.getByRole("textbox", { name: "Note" }), {
      target: { value: text },
    });
    let finishRead = () => {};
    api.getSavedListItemNote.mockReturnValueOnce(
      new Promise((resolve) => {
        finishRead = () => resolve({ ...entry, note: saved });
      }),
    );
    api.updateSavedListItemNote.mockRejectedValueOnce(conflict);
    fireEvent.click(screen.getByRole("button", { name: "Save" }));
    await waitFor(() => expect(api.getSavedListItemNote).toHaveBeenCalledTimes(2));
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    await act(async () => finishRead());
    const edit = await screen.findByRole("button", {
      name: saved ? "Edit" : "Add note",
    });
    expect(edit).toHaveFocus();
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    expect(api.updateSavedListItemNote).toHaveBeenCalledExactlyOnceWith(
      "list_research",
      "actions:one",
      {
        note: { expected: entry.note, value: saved },
      },
    );
  },
);

it("keeps a copyable draft when an external removal makes the note unavailable", async () => {
  const { client } = setup();
  fireEvent.click(await screen.findByRole("button", { name: "Edit" }));
  fireEvent.change(screen.getByRole("textbox", { name: "Note" }), {
    target: { value: "Copy this draft" },
  });
  api.getSavedListItemNote.mockRejectedValue(
    new AtlasApiError(404, "Missing item", {
      code: "saved_list_item_not_found",
      message: "Missing item",
    }),
  );
  await act(async () => {
    await client.invalidateQueries({
      queryKey: ["saved-list-item-note", "list_research", "actions:one"],
    });
  });
  expect(await screen.findByText("Item removed from this list")).toBeVisible();
  expect(screen.getByRole("textbox", { name: "Note" })).toHaveValue("Copy this draft");
  expect(screen.getByRole("textbox", { name: "Note" })).toHaveAttribute("readonly");
  expect(screen.queryByRole("button", { name: "Save" })).not.toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "Discard draft" }));
  expect(screen.queryByRole("textbox", { name: "Note" })).not.toBeInTheDocument();
  expect(api.updateSavedListItemNote).not.toHaveBeenCalled();
});
