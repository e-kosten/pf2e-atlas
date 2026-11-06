import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ConfigProvider, Grid } from "antd";
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { getDefinitionFilterOptions } from "../../api/atlasApi";
import type {
  DefinitionFilterOptionView,
  ReferenceFilterActionView,
} from "../../generated/atlas";
import { DefinitionFilterPicker } from "./DefinitionFilterPicker";
import {
  ReferenceHelpProvider,
  type ReferenceHelpReturn,
} from "../records/ReferenceHelp";
import { fieldFixture } from "../../test/fixtures";
import { RouterTestContext } from "../../test/router";

vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  getDefinitionFilterOptions: vi.fn(),
}));
const api = vi.mocked(getDefinitionFilterOptions);
const property: ReferenceFilterActionView = {
  kind: "search",
  label: "Creatures with this sense",
  artifact_generation: "generation-a",
  predicate: {
    kind: "exists",
    collection: "senses",
    scope_id: "same-sense",
    predicate: { kind: "compare", field: "type", op: "eq", value: "darkvision" },
  },
};
const configure: ReferenceFilterActionView = {
  kind: "configure_field",
  label: "Strength Modifier",
  field: fieldFixture("abilities.str.modifier", "number", "Strength Modifier"),
};
const first: DefinitionFilterOptionView = {
  target: { kind: "definition", key: "sense:darkvision" },
  label: "Supplied vision",
  help: {
    definition_key: "sense:darkvision",
    availability: "available",
    destination: { kind: "definition", key: "sense:darkvision" },
    related_filters: [property],
  },
};
const second: DefinitionFilterOptionView = {
  target: { kind: "definition", key: "ability:str" },
  label: "Supplied attribute",
  help: {
    definition_key: "ability:str",
    availability: "label_only",
    destination: null,
    related_filters: [configure],
  },
};
beforeEach(() => {
  vi.clearAllMocks();
  vi.spyOn(Grid, "useBreakpoint").mockReturnValue({ lg: true });
  api.mockImplementation(async (request) => ({
    artifact_generation: "generation-a",
    options: request.offset === 0 ? [first] : [second],
    selected: request.selected.flatMap((target) =>
      [first, second].filter(
        (entry) => JSON.stringify(entry.target) === JSON.stringify(target),
      ),
    ),
    total: 41n,
  }));
});
afterEach(() => vi.restoreAllMocks());
function setup() {
  const action = vi.fn(() => null);
  const help = vi.fn();
  render(
    <RouterTestContext>
      <ConfigProvider theme={{ token: { motion: false } }}>
        <QueryClientProvider
          client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
        >
          <ReferenceHelpProvider onHelp={help}>
            <DefinitionFilterPicker generation="generation-a" onAction={action} />
          </ReferenceHelpProvider>
        </QueryClientProvider>
      </ConfigProvider>
    </RouterTestContext>,
  );
  return { action, help };
}
it("keeps help separate from selection and inserts only the supplied property relationship", async () => {
  const { action, help } = setup();
  fireEvent.click(screen.getByRole("button", { name: "Edit Definition values" }));
  const read = (await screen.findByText("Read")).closest("a")!;
  expect(read).toHaveAccessibleName(/Read\s*Supplied vision/);
  fireEvent.click(read);
  await waitFor(() =>
    expect(help).toHaveBeenCalledWith(
      first.help,
      expect.any(HTMLElement),
      "generation-a",
      expect.any(Function),
    ),
  );
  act(() => (help.mock.calls[0][3] as ReferenceHelpReturn)("return"));
  await waitFor(() => expect(read).toHaveFocus());
  expect(action).not.toHaveBeenCalled();
  expect(screen.queryByRole("button", { name: property.label })).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: /Definition: Supplied vision/ }));
  await waitFor(() =>
    expect(api.mock.calls.some(([request]) => request.selected.length === 1)).toBe(
      true,
    ),
  );
  fireEvent.click(await screen.findByRole("button", { name: property.label }));
  expect(action).toHaveBeenLastCalledWith(property);
  fireEvent.click(screen.getByRole("button", { name: "More definitions" }));
  expect(
    await screen.findByRole("button", { name: /Definition: Supplied attribute/ }),
  ).toBeVisible();
  expect(
    screen.getByRole("region", { name: "Filters for Supplied vision" }),
  ).toBeVisible();
  fireEvent.click(
    screen.getByRole("button", { name: /Definition: Supplied attribute/ }),
  );
  await waitFor(() =>
    expect(screen.getByRole("button", { name: configure.label })).toBeVisible(),
  );
  await waitFor(() =>
    expect(api.mock.calls[api.mock.calls.length - 1]?.[0].selected).toEqual([
      second.target,
    ]),
  );
  fireEvent.click(screen.getByRole("button", { name: configure.label }));
  expect(action).toHaveBeenLastCalledWith(configure);
  expect(action).toHaveBeenCalledTimes(2);
});

it.each(["desktop", "phone"] as const)(
  "suspends and returns the same %s picker session after reading",
  async (device) => {
    vi.mocked(Grid.useBreakpoint).mockReturnValue(
      device === "phone" ? { xs: true, sm: false } : { lg: true },
    );
    const next = {
      ...first,
      label: "Supplied Chthonian",
      target: { kind: "definition" as const, key: "language:chthonian" },
      help: {
        ...first.help,
        definition_key: "language:chthonian",
        destination: { kind: "definition" as const, key: "language:chthonian" },
      },
    };
    api.mockImplementation(async (request) => ({
      artifact_generation: "generation-a",
      options: request.offset === 0 ? [first] : [next],
      selected: request.selected.flatMap((target) =>
        [first, next].filter(
          (entry) => JSON.stringify(entry.target) === JSON.stringify(target),
        ),
      ),
      total: 41n,
    }));
    const { help, action } = setup();
    const trigger = screen.getByRole("button", { name: "Edit Definition values" });
    fireEvent.click(trigger);
    const search = await screen.findByRole("textbox", {
      name: "Search definition choices",
    });
    fireEvent.change(search, { target: { value: "Supplied" } });
    await waitFor(() =>
      expect(api.mock.calls.some(([request]) => request.text === "Supplied")).toBe(
        true,
      ),
    );
    fireEvent.click(screen.getByRole("button", { name: "More definitions" }));
    const row = await screen.findByRole("button", {
      name: /Definition: Supplied Chthonian/,
    });
    fireEvent.click(row);
    await waitFor(() =>
      expect(
        api.mock.calls.some(
          ([request]) => request.offset === 40 && request.selected.length === 1,
        ),
      ).toBe(true),
    );
    const read = screen.getByText("Read").closest("a")!;
    read.focus();
    fireEvent.click(read);
    await waitFor(() => expect(help).toHaveBeenCalledOnce());
    expect(trigger).toHaveAttribute("aria-expanded", "false");
    expect(read.isConnected).toBe(true);
    expect(search).toHaveValue("Supplied");
    expect(action).not.toHaveBeenCalled();
    act(() => (help.mock.calls[0][3] as ReferenceHelpReturn)("return"));
    await waitFor(() => expect(read).toHaveFocus());
    await waitFor(() => expect(read).toBeVisible());
    expect(search).toHaveValue("Supplied");
    expect(screen.getByRole("status")).toHaveTextContent("Page 2 of 2");
    expect(
      screen.getByRole("button", { name: /Remove selected Supplied Chthonian/ }),
    ).toBeVisible();
  },
);

it.each(["desktop", "phone"] as const)(
  "does not resume a suspended %s picker on full-page departure",
  async (device) => {
    vi.mocked(Grid.useBreakpoint).mockReturnValue(
      device === "phone" ? { xs: true, sm: false } : { lg: true },
    );
    const { help } = setup();
    const trigger = screen.getByRole("button", { name: "Edit Definition values" });
    fireEvent.click(trigger);
    const read = (await screen.findByText("Read")).closest("a")!;
    fireEvent.click(read, { ctrlKey: true });
    expect(help).not.toHaveBeenCalled();
    expect(trigger).toHaveAttribute("aria-expanded", "true");
    fireEvent.click(read);
    await waitFor(() => expect(help).toHaveBeenCalledOnce());
    act(() => (help.mock.calls[0][3] as ReferenceHelpReturn)("navigate"));
    await waitFor(() => expect(trigger).toHaveAttribute("aria-expanded", "false"));
    expect(
      screen.queryByRole("textbox", { name: "Search definition choices" }),
    ).toBeNull();
  },
);
