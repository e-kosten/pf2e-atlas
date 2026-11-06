import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen } from "@testing-library/react";
import { AtlasApiError, getDefinitionFilterOptions } from "../../api/atlasApi";
import { useState } from "react";
import { DefinitionReferenceValue } from "./DefinitionReferenceValue";

vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  getDefinitionFilterOptions: vi.fn(),
}));
const api = vi.mocked(getDefinitionFilterOptions);
const target = { kind: "definition" as const, key: "language:chthonian" };
const selected = {
  target,
  label: "Chthonian",
  help: {
    definition_key: target.key,
    availability: "label_only" as const,
    destination: null,
    related_filters: [],
  },
};
function setup() {
  return render(
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <DefinitionReferenceValue
        reference={{ target, include_contents: true }}
        generation="generation-a"
        onChange={vi.fn()}
      />
    </QueryClientProvider>,
  );
}
beforeEach(() => vi.clearAllMocks());
it.each(["label_only", "not_in_catalog"] as const)(
  "uses the selected backend label for %s without showing an identity fallback",
  async (availability) => {
    api.mockResolvedValue({
      artifact_generation: "generation-a",
      options: [],
      selected: [{ ...selected, help: { ...selected.help, availability } }],
      total: 0n,
    });
    setup();
    expect(await screen.findByText("Chthonian")).toBeVisible();
    expect(screen.queryByText("language:chthonian")).toBeNull();
    expect(api).toHaveBeenCalledWith(
      expect.objectContaining({
        selected: [target],
        expected_artifact_generation: "generation-a",
      }),
      expect.any(AbortSignal),
    );
  },
);
it.each([
  { name: "generation", generation: "generation-b", selected: [selected] },
  {
    name: "target",
    generation: "generation-a",
    selected: [
      { ...selected, target: { kind: "definition" as const, key: "language:common" } },
    ],
  },
])(
  "rejects a selected label from a different $name",
  async ({ generation, selected }) => {
    api.mockResolvedValue({
      artifact_generation: generation,
      options: [],
      selected,
      total: 0n,
    });
    setup();
    expect(await screen.findByRole("status")).toHaveTextContent(
      "Definition unavailable",
    );
    expect(screen.queryByText("Chthonian")).toBeNull();
    expect(screen.queryByText("language:chthonian")).toBeNull();
  },
);

it("refreshes through the host and retries with its newly adopted generation without editing the reference", async () => {
  const reference = { target, include_contents: false };
  const change = vi.fn();
  const refresh = vi.fn();
  api.mockImplementation(async (request) => {
    if (request.expected_artifact_generation === "generation-a")
      throw new AtlasApiError(409, "Changed", {
        code: "artifact_generation_changed",
        message: "Changed",
        recoverable_action: "refresh_results",
      });
    return {
      artifact_generation: "generation-b",
      options: [],
      selected: [{ ...selected, label: "Updated Chthonian" }],
      total: 0n,
    };
  });
  function Host() {
    const [generation, setGeneration] = useState("generation-a");
    return (
      <DefinitionReferenceValue
        reference={reference}
        generation={generation}
        onChange={change}
        onRefresh={async () => {
          refresh();
          setGeneration("generation-b");
        }}
      />
    );
  }
  render(
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <Host />
    </QueryClientProvider>,
  );
  fireEvent.click(await screen.findByRole("button", { name: "Refresh filters" }));
  expect(await screen.findByText("Updated Chthonian")).toBeVisible();
  expect(refresh).toHaveBeenCalledOnce();
  expect(
    api.mock.calls.map(([request]) => request.expected_artifact_generation),
  ).toEqual(["generation-a", "generation-b"]);
  expect(change).not.toHaveBeenCalled();
  expect(reference.include_contents).toBe(false);
});
