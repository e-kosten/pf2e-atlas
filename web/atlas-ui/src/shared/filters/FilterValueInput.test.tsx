import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ConfigProvider } from "antd";
import { useState } from "react";
import { FilterValueInput } from "./FilterValueInput";
import { editorFixture, fieldFixture } from "../../test/fixtures";

const api = vi.hoisted(() => ({ discoverFilterValues: vi.fn() }));
vi.mock("../../api/atlasApi", () => api);
function renderInput(
  closed = false,
  onChange = vi.fn(),
  initialValues = ["absent-selected"],
) {
  const definition = {
    ...fieldFixture("traits", "set", "Traits"),
    choices: closed ? ["fire", "cold"] : [],
  };
  const field = editorFixture([definition]).groups[0].fields[0];
  function ControlledInput() {
    const [value, setValue] = useState(initialValues);
    return (
      <FilterValueInput
        field={field}
        operator="includes_any"
        value={value}
        onChange={(next) => {
          setValue(next as string[]);
          onChange(next);
        }}
        scope={{ kind: "saved_list", list_ref: "research" }}
      />
    );
  }
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <QueryClientProvider
        client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
      >
        <ControlledInput />
      </QueryClientProvider>
    </ConfigProvider>,
  );
}
describe("shared value control vocabulary", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    api.discoverFilterValues.mockImplementation((request) =>
      Promise.resolve({
        kind: "vocabulary",
        scope: request.scope,
        values: {
          field: "traits",
          selected_values: request.selected_values,
          total_values: 90n,
          options: request.offset ? [{ value: "second-page" }] : [{ value: "fire" }],
          basis: "product_key_scope_before_edition_preference",
        },
      }),
    );
  });
  it("uses closed labels locally without requesting any discovery", () => {
    renderInput(true);
    expect(screen.getByRole("combobox", { name: "Traits" })).toBeInTheDocument();
    expect(api.discoverFilterValues).not.toHaveBeenCalled();
  });
  it("keeps selected values while paging server suggestions in the saved-list scope", async () => {
    renderInput();
    await waitFor(() => expect(api.discoverFilterValues).toHaveBeenCalled());
    fireEvent.mouseDown(screen.getByRole("combobox", { name: "Traits" }));
    fireEvent.click(await screen.findByRole("button", { name: "More values" }));
    await waitFor(() =>
      expect(api.discoverFilterValues).toHaveBeenLastCalledWith(
        expect.objectContaining({
          kind: "vocabulary",
          scope: { kind: "saved_list", list_ref: "research" },
          offset: 40,
          selected_values: ["absent-selected"],
        }),
        expect.any(AbortSignal),
      ),
    );
    expect(screen.getAllByText("absent-selected").length).toBeGreaterThan(0);
    expect(
      await screen.findByRole("option", { name: "second-page" }),
    ).toBeInTheDocument();
    expect(screen.getByText("Available values in this list")).toBeInTheDocument();
  });
  it("sends search terms to the server and resets paging while preserving selections", async () => {
    renderInput();
    const input = screen.getByRole("combobox", { name: "Traits" });
    fireEvent.change(input, { target: { value: "fi" } });
    await waitFor(() =>
      expect(api.discoverFilterValues).toHaveBeenLastCalledWith(
        expect.objectContaining({
          text: "fi",
          offset: 0,
          selected_values: ["absent-selected"],
        }),
        expect.any(AbortSignal),
      ),
    );
  });
  it("does not reset an already opened suggestion page when the initial debounce settles", async () => {
    vi.useFakeTimers();
    try {
      renderInput();
      await act(async () => {
        await vi.advanceTimersByTimeAsync(1);
      });
      fireEvent.mouseDown(screen.getByRole("combobox", { name: "Traits" }));
      fireEvent.click(screen.getByRole("button", { name: "More values" }));
      await act(async () => {
        await vi.advanceTimersByTimeAsync(1);
      });
      expect(api.discoverFilterValues).toHaveBeenLastCalledWith(
        expect.objectContaining({ offset: 40 }),
        expect.any(AbortSignal),
      );
      await act(async () => {
        await vi.advanceTimersByTimeAsync(250);
      });
      expect(screen.getByRole("button", { name: "Previous values" })).toBeEnabled();
      expect(screen.getByRole("option", { name: "second-page" })).toBeInTheDocument();
    } finally {
      vi.useRealTimers();
    }
  });
  it("clears the actual search input between consecutive keyboard trait selections", async () => {
    const onChange = vi.fn();
    renderInput(false, onChange, []);
    const input = screen.getByRole("combobox", { name: "Traits" });
    fireEvent.focus(input);
    fireEvent.change(input, { target: { value: "wood" } });
    fireEvent.keyDown(input, { key: "Enter", code: "Enter", keyCode: 13 });
    fireEvent.keyUp(input, { key: "Enter", code: "Enter", keyCode: 13 });
    fireEvent.keyDown(input, { key: "Escape", code: "Escape", keyCode: 27 });
    fireEvent.keyUp(input, { key: "Escape", code: "Escape", keyCode: 27 });
    expect(onChange).toHaveBeenLastCalledWith(["wood"]);
    expect(input).toHaveValue("");

    fireEvent.change(input, {
      target: { value: (input as HTMLInputElement).value + "magical" },
    });
    fireEvent.keyDown(input, { key: "Enter", code: "Enter", keyCode: 13 });
    fireEvent.keyUp(input, { key: "Enter", code: "Enter", keyCode: 13 });
    fireEvent.keyDown(input, { key: "Escape", code: "Escape", keyCode: 27 });
    fireEvent.keyUp(input, { key: "Escape", code: "Escape", keyCode: 27 });
    expect(onChange).toHaveBeenLastCalledWith(["wood", "magical"]);
    expect(input).toHaveValue("");
    await waitFor(() =>
      expect(api.discoverFilterValues).toHaveBeenLastCalledWith(
        expect.objectContaining({
          scope: { kind: "saved_list", list_ref: "research" },
          selected_values: ["wood", "magical"],
          text: null,
          offset: 0,
        }),
        expect.any(AbortSignal),
      ),
    );
  });
});
