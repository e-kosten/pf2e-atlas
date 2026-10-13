import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { Form, Input } from "antd";
import { useState } from "react";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import {
  AtlasApiError,
  searchPage,
  getRecordDetail,
  discoverFilterEditor,
  discoverFilterCounts,
} from "../../../api/atlasApi";
import { detailFixture, editorFixture, summaryFixture } from "../../../test/fixtures";
import { SearchPickerModal } from "./SearchPickerModal";

vi.mock("../../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../../api/atlasApi")>()),
  searchPage: vi.fn(),
  getRecordDetail: vi.fn(),
  discoverFilterEditor: vi.fn(),
  discoverFilterCounts: vi.fn(),
}));
const search = vi.mocked(searchPage);
function result(title = "Ghoul", generation = "generation-a") {
  return {
    artifact_generation: generation,
    mode: { kind: "list_records" as const },
    coverage: null,
    page: {
      number: 1,
      size: 25,
      count: 1,
      total: 1n,
      has_more: false,
      next_page: null,
    },
    rows: [
      { record: summaryFixture("actors:" + title.toLowerCase(), title), matches: [] },
    ],
  };
}
function Picker({
  disabled = false,
  open = true,
}: {
  disabled?: boolean;
  open?: boolean;
}) {
  const [form] = Form.useForm();
  const [selected, setSelected] = useState<string | null>(null);
  return (
    <SearchPickerModal
      title="Choose record"
      disabled={disabled}
      open={open}
      form={form}
      buildRequest={(query) => ({
        criteria: query.trim()
          ? { kind: "text_search", query, mode: "hybrid", filter: null }
          : { kind: "list_records", filter: null },
        expected_artifact_generation: null,
        page: { number: 1, size: 25 },
      })}
      emptyPrompt="Search for a record"
      emptyResults="No matching records"
      selectedPrompt="Choose a record"
      selectedLabel={(row) => `Selected ${row.record.title}`}
      selectedRecordKey={selected}
      onSelectedRecordKeyChange={setSelected}
      onCancel={vi.fn()}
      onFinish={vi.fn()}
      okButtonProps={{ disabled: !selected }}
    >
      <Form.Item name="note" label="Note">
        <Input />
      </Form.Item>
    </SearchPickerModal>
  );
}
function renderPicker() {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const view = render(
    <QueryClientProvider client={client}>
      <Picker />
    </QueryClientProvider>,
  );
  return {
    client,
    setDisabled: (disabled: boolean) =>
      view.rerender(
        <QueryClientProvider client={client}>
          <Picker disabled={disabled} />
        </QueryClientProvider>,
      ),
  };
}
async function submit(query: string) {
  const back = screen.queryByRole("button", { name: "Results" });
  if (back) fireEvent.click(back);
  const input = screen.getByRole("searchbox", { name: "Search records" });
  fireEvent.change(input, { target: { value: query } });
  fireEvent.keyDown(input, { key: "Enter", code: "Enter", keyCode: 13 });
}
beforeEach(() => {
  vi.clearAllMocks();
  search.mockReset();
  search.mockResolvedValue(result());
  vi.mocked(getRecordDetail).mockImplementation((key) =>
    Promise.resolve(detailFixture(key)),
  );
  vi.mocked(discoverFilterEditor).mockResolvedValue({
    ...editorFixture(),
    groups: [],
    common_field_ids: [],
  });
  vi.mocked(discoverFilterCounts).mockResolvedValue({
    counts: { status: "unavailable", reason: "non_conjunctive_context" },
  });
});
it("previews before explicit Add and retains the named candidate when a later query has no results", async () => {
  renderPicker();
  const link = await screen.findByRole("link", { name: /Ghoul/ });
  expect(getRecordDetail).not.toHaveBeenCalled();
  fireEvent.click(link);
  await waitFor(() => expect(getRecordDetail).toHaveBeenCalled());
  expect(screen.getByRole("button", { name: "Add" })).toBeEnabled();
  search.mockResolvedValue({
    ...result(),
    rows: [],
    page: { ...result().page, count: 0, total: 0n },
  });
  await submit("dragon");
  await waitFor(() => expect(screen.queryByRole("link", { name: /Ghoul/ })).toBeNull());
  expect(screen.getByText("Selected Ghoul")).toHaveTextContent("Selected Ghoul");
  expect(screen.getByRole("button", { name: "Add" })).toBeEnabled();
});
it("blocks selection and form actions while Add is pending, preserving the draft", async () => {
  const view = renderPicker();
  fireEvent.change(screen.getByLabelText("Note"), {
    target: { value: "Keep this note" },
  });
  fireEvent.click(await screen.findByRole("link", { name: /Ghoul/ }));
  view.setDisabled(true);
  expect(screen.getByRole("searchbox")).toBeDisabled();
  expect(screen.getByRole("button", { name: "Add" })).toBeDisabled();
  expect(screen.getByRole("button", { name: "Cancel" })).toBeDisabled();
  expect(screen.getByText("Selected Ghoul")).toHaveTextContent("Selected Ghoul");
  view.setDisabled(false);
  expect(screen.getByLabelText("Note")).toHaveValue("Keep this note");
  expect(screen.getByRole("button", { name: "Add" })).toBeEnabled();
});
it("distinguishes failed retrieval from empty results and retries the same request", async () => {
  search
    .mockRejectedValueOnce(new Error("Server unavailable"))
    .mockResolvedValue(result());
  renderPicker();
  expect(await screen.findByRole("alert")).toHaveTextContent("Server unavailable");
  expect(screen.queryByText("No matching records")).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Retry search" }));
  expect(await screen.findByRole("link", { name: /Ghoul/ })).toBeInTheDocument();
  expect(search).toHaveBeenCalledTimes(2);
});
it("adopts generation once and explicitly refreshes conflicts without clearing candidate or form", async () => {
  const { client } = renderPicker();
  await screen.findByRole("link", { name: /Ghoul/ });
  expect(search).toHaveBeenCalledTimes(1);
  fireEvent.click(screen.getByRole("link", { name: /Ghoul/ }));
  fireEvent.change(screen.getByLabelText("Note"), {
    target: { value: "Unsaved encounter note" },
  });
  client.setQueryData(["encounter", "draft-owner"], { name: "Encounter" });
  search
    .mockRejectedValueOnce(
      new AtlasApiError(409, "The reference collection changed.", {
        code: "artifact_generation_changed",
        message: "The reference collection changed.",
        recoverable_action: "refresh_results",
      }),
    )
    .mockResolvedValue(result("Dragon", "generation-b"));
  await submit("dragon");
  expect(await screen.findByRole("alert")).toHaveTextContent(
    "The reference collection changed.",
  );
  expect(search.mock.calls[1][0].expected_artifact_generation).toBe("generation-a");
  fireEvent.click(screen.getByRole("button", { name: "Refresh results" }));
  expect(await screen.findByRole("link", { name: /Dragon/ })).toBeInTheDocument();
  expect(search).toHaveBeenCalledTimes(3);
  expect(search.mock.calls[2][0]).toMatchObject({
    expected_artifact_generation: null,
    page: { number: 1, size: 25 },
    criteria: { query: "dragon", mode: "hybrid" },
  });
  expect(screen.getByLabelText("Note")).toHaveValue("Unsaved encounter note");
  expect(screen.getByText("Selected Ghoul")).toHaveTextContent("Selected Ghoul");
  expect(client.getQueryData(["encounter", "draft-owner"])).toEqual({
    name: "Encounter",
  });
});
