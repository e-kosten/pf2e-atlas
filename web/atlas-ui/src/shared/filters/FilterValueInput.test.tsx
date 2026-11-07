import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ConfigProvider } from "antd";
import { useState } from "react";
import { FilterValueInput } from "./FilterValueInput";
import type {
  DiscoverFilterValuesRequest,
  FilterValueListView,
  DiscoverFilterCountsRequest,
  FilterCountsView,
} from "../../generated/atlas";
import { detailFixture, editorFixture, fieldFixture } from "../../test/fixtures";
import { RouterTestContext } from "../../test/router";

const api = vi.hoisted(() => ({
  discoverFilterValues:
    vi.fn<
      (
        request: DiscoverFilterValuesRequest,
        signal?: AbortSignal,
      ) => Promise<FilterValueListView>
    >(),
  discoverFilterCounts:
    vi.fn<
      (
        request: DiscoverFilterCountsRequest,
        signal?: AbortSignal,
      ) => Promise<FilterCountsView>
    >(),
}));
vi.mock("../../api/atlasApi", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../api/atlasApi")>()),
  ...api,
}));
function renderInput(
  closed = false,
  onChange = vi.fn(),
  initialValues = ["absent-selected"],
  withCounts = false,
  excluded = false,
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
        excluded={excluded}
        value={value}
        onChange={(next) => {
          setValue(next as string[]);
          onChange(next);
        }}
        scope={{ kind: "saved_list", list_ref: "research" }}
        countContext={
          withCounts
            ? {
                kind: "saved_list",
                list_ref: "research",
                filter: {
                  kind: "set_match",
                  clause_id: "traits",
                  field: "traits",
                  op: "includes_any",
                  values: value,
                },
                text: null,
                mode: "hybrid",
              }
            : undefined
        }
        clauseId="traits"
        countLimit={128}
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
function openVocabulary() {
  fireEvent.click(screen.getByRole("button", { name: "Edit Traits values" }));
}
it("keeps short choices as normal selection options when supplied help is available", async () => {
  const field = editorFixture([
    { ...fieldFixture("traits", "set", "Traits"), choices: ["fire"] },
  ]).groups[0].fields[0];
  field.choices[0].help = {
    definition_key: "trait:fire",

    destination: { kind: "definition", key: "trait:fire" },
  };
  const change = vi.fn();
  render(
    <RouterTestContext>
      <ConfigProvider theme={{ token: { motion: false } }}>
        <QueryClientProvider
          client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
        >
          <FilterValueInput
            field={field}
            operator="includes_any"
            value={[]}
            onChange={change}
            scope={{ kind: "artifact" }}
            countsEnabled={false}
          />
        </QueryClientProvider>
      </ConfigProvider>
    </RouterTestContext>,
  );
  fireEvent.mouseDown(screen.getByRole("combobox", { name: "Traits" }));
  const choice = await screen.findByTitle("fire");
  expect(screen.queryByRole("link", { name: /Read/ })).toBeNull();
  fireEvent.click(choice);
  expect(change).toHaveBeenCalledWith(["fire"]);
});
describe("shared value control vocabulary", () => {
  it("allows excluding a known-zero choice", async () => {
    const change = vi.fn();
    renderInput(true, change, ["fire"], true, true);
    fireEvent.mouseDown(screen.getByRole("combobox", { name: "Traits" }));
    fireEvent.click(await screen.findByText("cold (0)"));
    expect(change).toHaveBeenCalledWith(["fire", "cold"]);
  });
  beforeEach(() => {
    vi.clearAllMocks();
    api.discoverFilterCounts.mockResolvedValue({
      search_profile: "bge-base",
      artifact_generation: "generation-a",
      counts: {
        status: "available",
        value: {
          field: "traits",
          options: [
            { value: "fire", distinct_roots: 8n, selected: false },
            { value: "cold", distinct_roots: 0n, selected: false },
          ],
          total_values: 2n,
          exhaustive: true,
          count_basis: "exact counterfactual displayed roots",
        },
      },
    });
    api.discoverFilterValues.mockImplementation((request) =>
      Promise.resolve(
        request.kind === "contextual"
          ? {
              kind: "contextual",
              search_profile: "bge-base",
              artifact_generation: "generation-a",
              outcome: { status: "unavailable", reason: "non_conjunctive_context" },
            }
          : {
              kind: "vocabulary",
              scope: request.scope,
              values: {
                field: "traits",
                selected_values: request.selected_values,
                total_values: 90n,
                options: request.offset
                  ? [{ value: "second-page", label: "Second-page" }]
                  : [{ value: "fire", label: "Fire" }],
                basis: "product_key_scope",
              },
            },
      ),
    );
  });
  it("uses closed labels locally without requesting any discovery", () => {
    renderInput(true);
    expect(screen.getByRole("combobox", { name: "Traits" })).toBeInTheDocument();
    expect(api.discoverFilterValues).not.toHaveBeenCalled();
  });
  it("omits blank catalog values that would clear an Advanced operand", async () => {
    api.discoverFilterValues.mockResolvedValue({
      kind: "vocabulary",
      scope: { kind: "saved_list", list_ref: "research" },
      values: {
        field: "traits",
        selected_values: [],
        total_values: 3n,
        options: [
          { value: "", label: "" },
          { value: " ", label: "" },
          { value: "fire", label: "Fire" },
        ],
        basis: "product_key_scope",
      },
    });
    const change = vi.fn();
    renderInput(false, change, []);
    openVocabulary();
    const fire = await screen.findByRole("button", {
      name: "Traits: Fire, activate to select",
    });
    expect(screen.getAllByRole("button", { name: /^Traits:/ })).toEqual([fire]);
    fireEvent.click(fire);
    expect(change).toHaveBeenCalledExactlyOnceWith(["fire"]);
  });
  it("counts closed choices only when the advanced selector opens", async () => {
    renderInput(true, vi.fn(), ["fire"], true);
    expect(api.discoverFilterCounts).not.toHaveBeenCalled();
    fireEvent.mouseDown(screen.getByRole("combobox", { name: "Traits" }));
    await waitFor(() =>
      expect(api.discoverFilterCounts).toHaveBeenCalledWith(
        expect.objectContaining({
          field_id: "traits",
          clause_id: "traits",
          values: ["fire", "cold"],
        }),
        expect.any(AbortSignal),
      ),
    );
    expect(await screen.findByText("cold (0)")).toBeInTheDocument();
    expect(api.discoverFilterValues).not.toHaveBeenCalled();
  });
  it("annotates advanced open vocabulary independently of paging and retained selections", async () => {
    renderInput(false, vi.fn(), ["absent-selected"], true);
    expect(api.discoverFilterValues).not.toHaveBeenCalled();
    expect(api.discoverFilterCounts).not.toHaveBeenCalled();
    openVocabulary();
    await waitFor(() =>
      expect(api.discoverFilterCounts).toHaveBeenCalledWith(
        expect.objectContaining({
          context: expect.objectContaining({
            kind: "saved_list",
            list_ref: "research",
          }),
          clause_id: "traits",
          values: ["absent-selected", "fire"],
        }),
        expect.any(AbortSignal),
      ),
    );
    expect(await screen.findByLabelText("8 matching records")).toBeInTheDocument();
  });
  it("keeps selected values while paging server suggestions in the saved-list scope", async () => {
    const onChange = vi.fn();
    renderInput(false, onChange);
    openVocabulary();
    await waitFor(() => expect(api.discoverFilterValues).toHaveBeenCalled());
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
    fireEvent.click(
      await screen.findByRole("button", {
        name: "Traits: Second-page, activate to select",
      }),
    );
    expect(onChange).toHaveBeenCalledExactlyOnceWith([
      "absent-selected",
      "second-page",
    ]);
    expect(api.discoverFilterValues).toHaveBeenLastCalledWith(
      expect.objectContaining({ scope: { kind: "saved_list", list_ref: "research" } }),
      expect.any(AbortSignal),
    );
  });
  it("sends search terms to the server and resets paging while preserving selections", async () => {
    renderInput();
    openVocabulary();
    const input = screen.getByRole("textbox", { name: "Search traits choices" });
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
  it("keeps off-search contextual selections in chips but out of search results", async () => {
    api.discoverFilterValues.mockResolvedValue({
      kind: "contextual",
      search_profile: "bge-base",
      artifact_generation: "generation-a",
      outcome: {
        status: "available",
        value: {
          field: "traits",
          options: [
            { value: "acid", label: "Acid", distinct_roots: 4n, selected: true },
            { value: "cold", label: "Cold", distinct_roots: 2n, selected: true },
          ],
          total_values: 2n,
          exhaustive: true,
          count_basis: "exact counterfactual displayed roots",
        },
      },
    });
    renderInput(false, vi.fn(), ["acid", "cold"], true);
    openVocabulary();
    expect(await screen.findByText("2 values in this list")).toBeInTheDocument();
    fireEvent.change(screen.getByRole("textbox", { name: "Search traits choices" }), {
      target: { value: "Acid" },
    });
    expect(await screen.findByText("1 value in this list")).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /^Traits: Acid, selected/ }),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: /^Traits: Cold, selected/ }),
    ).toBeNull();
    expect(
      screen.getByRole("button", { name: "Remove selected Cold from Traits" }),
    ).toBeVisible();
  });

  it("clears a vocabulary operand in one action without opening the picker", async () => {
    const change = vi.fn();
    renderInput(false, change, ["acid", "cold"]);
    const clear = screen.getByRole("button", { name: "Clear all Traits values" });
    fireEvent.click(clear);
    expect(change).toHaveBeenCalledExactlyOnceWith([]);
    const opener = screen.getByRole("button", { name: "Edit Traits values" });
    expect(opener).toHaveAttribute("aria-expanded", "false");
    expect(opener).toHaveAccessibleDescription("0 selected");
    await waitFor(() => expect(opener).toHaveFocus());
  });

  it("does not reset an already opened suggestion page when the initial debounce settles", async () => {
    vi.useFakeTimers();
    try {
      renderInput();
      await act(async () => {
        await vi.advanceTimersByTimeAsync(1);
      });
      openVocabulary();
      await act(async () => {
        await vi.advanceTimersByTimeAsync(1);
      });
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
      expect(
        screen.getByRole("button", { name: "Traits: Second-page, activate to select" }),
      ).toBeInTheDocument();
    } finally {
      vi.useRealTimers();
    }
  });
  it("clears the actual search input between consecutive keyboard trait selections", async () => {
    const onChange = vi.fn();
    renderInput(false, onChange, []);
    openVocabulary();
    const input = screen.getByRole("textbox", { name: "Search traits choices" });
    fireEvent.focus(input);
    fireEvent.change(input, { target: { value: "wood" } });
    fireEvent.keyDown(input, { key: "Enter", code: "Enter", keyCode: 13 });
    fireEvent.keyUp(input, { key: "Enter", code: "Enter", keyCode: 13 });
    expect(onChange).toHaveBeenLastCalledWith(["wood"]);
    expect(input).toHaveValue("");

    fireEvent.change(input, {
      target: { value: (input as HTMLInputElement).value + "magical" },
    });
    fireEvent.keyDown(input, { key: "Enter", code: "Enter", keyCode: 13 });
    fireEvent.keyUp(input, { key: "Enter", code: "Enter", keyCode: 13 });
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

it("uses supplied Actions labels and Read while refusing arbitrary manual values", async () => {
  const field = editorFixture([fieldFixture("actions", "set", "Actions")]).groups[0]
    .fields[0];
  field.allow_custom_values = false;
  const key = "bestiary-ability-glossary-srd:grab";
  const onChange = vi.fn();
  vi.mocked(api.discoverFilterValues).mockImplementation(async (request) => {
    if (request.kind !== "vocabulary") throw new Error("Expected vocabulary");
    return {
      kind: "vocabulary",
      values: {
        field: "actions",
        selected_values: request.selected_values,
        total_values: 1n,
        options: [
          {
            value: key,
            label: "Grab",
            help: {
              definition_key: null,
              destination: {
                kind: "record",
                navigation: {
                  record_key: key,
                  embedded_path: [],
                  field: null,
                  passage: null,
                  source_fingerprint: null,
                },
              },
            },
          },
        ],
        basis: "product_key_scope",
      },
    };
  });
  render(
    <RouterTestContext>
      <ConfigProvider theme={{ token: { motion: false } }}>
        <QueryClientProvider
          client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
        >
          <FilterValueInput
            field={field}
            operator="includes_any"
            value={[key]}
            onChange={onChange}
            scope={{ kind: "artifact" }}
            countsEnabled={false}
          />
        </QueryClientProvider>
      </ConfigProvider>
    </RouterTestContext>,
  );
  expect(
    screen.getByRole("button", {
      name: "Remove selected Loading action… from Actions",
    }),
  ).toBeVisible();
  await waitFor(() =>
    expect(
      screen.getByRole("button", {
        name: "Remove selected Grab from Actions",
      }),
    ).toBeVisible(),
  );
  fireEvent.click(screen.getByRole("button", { name: "Edit Actions values" }));
  expect(await screen.findByRole("link", { name: /^Read/ })).toHaveAttribute(
    "href",
    `/records/${key}`,
  );
  fireEvent.change(screen.getByRole("textbox", { name: "Search actions choices" }), {
    target: { value: "Invented action" },
  });
  await waitFor(() =>
    expect(api.discoverFilterValues).toHaveBeenCalledWith(
      expect.objectContaining({ text: "Invented action" }),
      expect.any(AbortSignal),
    ),
  );
  expect(screen.queryByRole("button", { name: /Use exact value/ })).toBeNull();
  fireEvent.keyDown(screen.getByRole("textbox", { name: "Search actions choices" }), {
    key: "Enter",
  });
  expect(onChange).not.toHaveBeenCalled();
});

it("retains a chosen action's label while searching and reopening its picker", async () => {
  const field = editorFixture([fieldFixture("actions", "set", "Actions")]).groups[0]
    .fields[0];
  field.allow_custom_values = false;
  const key = "bestiary-ability-glossary-srd:grab";
  api.discoverFilterValues.mockClear().mockImplementation(async (request) => {
    if (request.kind !== "vocabulary") throw new Error("Expected vocabulary");
    if (request.selected_values.length) return new Promise(() => {});
    return {
      kind: "vocabulary",
      scope: request.scope,
      values: {
        field: "actions",
        selected_values: [],
        total_values: 1n,
        options: [
          {
            value: key,
            label: "Grab",
            help: {
              definition_key: null,
              destination: { kind: "record", navigation: detailFixture(key).selected },
            },
          },
        ],
        basis: "product_key_scope",
      },
    };
  });
  function ControlledInput() {
    const [value, setValue] = useState<unknown>([]);
    return (
      <FilterValueInput
        field={field}
        operator="includes_any"
        value={value}
        onChange={setValue}
        scope={{ kind: "artifact" }}
        countsEnabled={false}
      />
    );
  }
  render(
    <RouterTestContext>
      <ConfigProvider theme={{ token: { motion: false } }}>
        <QueryClientProvider client={new QueryClient()}>
          <ControlledInput />
        </QueryClientProvider>
      </ConfigProvider>
    </RouterTestContext>,
  );
  const trigger = () => screen.getByRole("button", { name: "Edit Actions values" });
  const search = () => screen.getByRole("textbox", { name: "Search actions choices" });
  fireEvent.click(trigger());
  fireEvent.click(
    await screen.findByRole("button", { name: "Actions: Grab, activate to select" }),
  );
  fireEvent.change(search(), { target: { value: "another action" } });
  await waitFor(() =>
    expect(api.discoverFilterValues).toHaveBeenCalledWith(
      expect.objectContaining({ selected_values: [key], text: "another action" }),
      expect.any(AbortSignal),
    ),
  );
  const selectedRow = () =>
    screen.getByRole("button", { name: "Actions: Grab, selected; activate to remove" });
  expect(
    screen.queryByRole("button", {
      name: "Actions: Grab, selected; activate to remove",
    }),
  ).toBeNull();
  expect(
    screen.getByRole("button", { name: "Remove selected Grab from Actions" }),
  ).toBeVisible();
  fireEvent.keyDown(search(), { key: "Escape" });
  fireEvent.click(trigger());
  await waitFor(() => expect(selectedRow()).toBeVisible());
  fireEvent.keyDown(search(), { key: "Escape" });
  const remove = screen.getByRole("button", {
    name: "Remove selected Grab from Actions",
  });
  remove.focus();
  fireEvent.keyDown(remove, { key: "Enter" });
  expect(remove).not.toBeInTheDocument();
  await waitFor(() => expect(trigger()).toHaveFocus());
  expect(trigger()).toHaveAttribute("aria-expanded", "false");
});
