import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { Form, Input } from "antd";
import { useState } from "react";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { AtlasApiError, searchPage } from "../../../api/atlasApi";
import { summaryFixture } from "../../../test/fixtures";
import { SearchPickerModal } from "./SearchPickerModal";

vi.mock("../../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../../api/atlasApi")>()),
  searchPage: vi.fn(),
}));
const search = vi.mocked(searchPage);
const result = {
  artifact_generation: "generation-a",
  rows: [{ record: summaryFixture("actors:ghoul", "Ghoul"), matches: [] }],
} as unknown as Awaited<ReturnType<typeof searchPage>>;

function Picker({
  open = true,
  disabled = false,
}: {
  open?: boolean;
  disabled?: boolean;
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
        criteria: { kind: "text_search", query, mode: "hybrid", filter: null },
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
    ...view,
    client,
    setDisabled: (disabled: boolean) =>
      view.rerender(
        <QueryClientProvider client={client}>
          <Picker disabled={disabled} />
        </QueryClientProvider>,
      ),
  };
}
beforeEach(() => search.mockReset());
describe("shared record picker", () => {
  it("preserves selection and blocks picker controls while an action is pending", async () => {
    search.mockResolvedValue(result);
    const view = renderPicker();
    fireEvent.change(screen.getByLabelText("Search"), { target: { value: "ghoul" } });
    const radio = await screen.findByRole("radio", { name: "Select Ghoul" });
    fireEvent.click(radio);
    view.setDisabled(true);
    expect(screen.getByLabelText("Search")).toBeDisabled();
    expect(radio).toBeDisabled();
    expect(screen.getByRole("button", { name: "OK" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Cancel" })).toBeDisabled();
    fireEvent.click(screen.getByText("Ghoul"));
    expect(radio).toBeChecked();
    expect(screen.getByRole("status")).toHaveTextContent("Selected Ghoul");
    view.setDisabled(false);
    expect(screen.getByRole("button", { name: "OK" })).toBeEnabled();
  });
  it("associates the visible label and exposes native selected state, clearing selection during a changed query", async () => {
    search.mockResolvedValue(result);
    renderPicker();
    const input = screen.getByLabelText("Search");
    fireEvent.change(input, { target: { value: "ghoul" } });
    const radio = await screen.findByRole("radio", { name: "Select Ghoul" });
    radio.focus();
    fireEvent.click(radio);
    expect(radio).toBeChecked();
    expect(screen.getByRole("status")).toHaveTextContent("Selected Ghoul");
    expect(screen.getByRole("button", { name: "OK" })).toBeEnabled();
    fireEvent.change(input, { target: { value: "dragon" } });
    expect(
      screen.queryByRole("radio", { name: "Select Ghoul" }),
    ).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: "OK" })).toBeDisabled();
  });
  it("distinguishes a search failure from no matches and retries the current request", async () => {
    search
      .mockRejectedValueOnce(new Error("Server unavailable"))
      .mockResolvedValue(result);
    renderPicker();
    fireEvent.change(screen.getByLabelText("Search"), { target: { value: "ghoul" } });
    expect(await screen.findByRole("alert")).toHaveTextContent("Server unavailable");
    expect(screen.queryByText("No matching records")).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Retry search" }));
    expect(
      await screen.findByRole("radio", { name: "Select Ghoul" }),
    ).toBeInTheDocument();
    await waitFor(() => expect(screen.queryByRole("alert")).not.toBeInTheDocument());
    expect(search).toHaveBeenCalledTimes(2);
  });

  it("adopts one generation response without repeating inference, and explicitly refreshes a conflict while retaining the form draft", async () => {
    search
      .mockResolvedValueOnce(result)
      .mockRejectedValueOnce(
        new AtlasApiError(409, "The reference collection changed.", {
          code: "artifact_generation_changed",
          message: "The reference collection changed.",
          recoverable_action: "refresh_results",
        }),
      )
      .mockResolvedValue({
        ...result,
        artifact_generation: "generation-b",
        rows: [{ record: summaryFixture("actors:dragon", "Dragon"), matches: [] }],
      });
    const { client } = renderPicker();
    client.setQueryData(["encounter", "draft-owner"], { name: "Encounter" });
    fireEvent.change(screen.getByLabelText("Note"), {
      target: { value: "Unsaved encounter note" },
    });
    fireEvent.change(screen.getByLabelText("Search"), { target: { value: "ghoul" } });
    expect(
      await screen.findByRole("radio", { name: "Select Ghoul" }),
    ).toBeInTheDocument();
    expect(search).toHaveBeenCalledTimes(1);
    expect(search.mock.calls[0][0]).toEqual(
      expect.objectContaining({ expected_artifact_generation: null }),
    );
    fireEvent.change(screen.getByLabelText("Search"), { target: { value: "dragon" } });
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "The reference collection changed.",
    );
    expect(search.mock.calls[1][0]).toEqual(
      expect.objectContaining({ expected_artifact_generation: "generation-a" }),
    );
    expect(screen.queryByRole("radio", { name: "Select Ghoul" })).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Refresh results" }));
    expect(
      await screen.findByRole("radio", { name: "Select Dragon" }),
    ).toBeInTheDocument();
    expect(search).toHaveBeenCalledTimes(3);
    expect(search.mock.calls[2][0]).toEqual({
      criteria: { kind: "text_search", query: "dragon", mode: "hybrid", filter: null },
      page: { number: 1, size: 25 },
      expected_artifact_generation: null,
    });
    expect(screen.getByLabelText("Search")).toHaveValue("dragon");
    expect(screen.getByLabelText("Note")).toHaveValue("Unsaved encounter note");
    expect(client.getQueryData(["encounter", "draft-owner"])).toEqual({
      name: "Encounter",
    });
    expect(
      client
        .getQueryCache()
        .findAll({ queryKey: ["search-picker"] })
        .some(
          (query) =>
            query.state.data &&
            (query.state.data as { artifact_generation: string })
              .artifact_generation === "generation-a",
        ),
    ).toBe(false);
  });
});
