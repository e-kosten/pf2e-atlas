import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { Form } from "antd";
import { useState } from "react";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { openResultWindow } from "../../../api/atlasApi";
import { summaryFixture } from "../../../test/fixtures";
import { SearchPickerModal } from "./SearchPickerModal";

vi.mock("../../../api/atlasApi", () => ({ openResultWindow: vi.fn() }));
const search = vi.mocked(openResultWindow);
const result = {
  rows: [{ record: summaryFixture("actors:ghoul", "Ghoul"), matches: [] }],
} as unknown as Awaited<ReturnType<typeof openResultWindow>>;

function Picker({ open = true }: { open?: boolean }) {
  const [form] = Form.useForm();
  const [selected, setSelected] = useState<string | null>(null);
  return (
    <SearchPickerModal
      title="Choose record"
      open={open}
      form={form}
      buildRequest={(query) => ({
        mode: { kind: "text_search", query, mode: "hybrid", filter: null },
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
    />
  );
}
function renderPicker() {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return render(
    <QueryClientProvider client={client}>
      <Picker />
    </QueryClientProvider>,
  );
}
beforeEach(() => search.mockReset());
describe("shared record picker", () => {
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
});
