import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ConfigProvider, Grid } from "antd";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { getDefinitionFilterOptions } from "../../api/atlasApi";
import type {
  DefinitionFilterOptionView,
  ReferenceFilterActionView,
} from "../../generated/atlas";
import { DefinitionFilterPicker } from "./DefinitionFilterPicker";
import { ReferenceHelpProvider } from "../records/ReferenceHelp";
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
  expect(help).toHaveBeenCalledWith(
    first.help,
    expect.any(HTMLElement),
    "generation-a",
  );
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
