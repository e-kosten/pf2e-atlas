import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { AtlasApiError, getDefinitionFilterOptions } from "../../api/atlasApi";
import type { DefinitionFilterOptionView } from "../../generated/atlas";
import { useDefinitionFilterOptions } from "./useDefinitionFilterOptions";

vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  getDefinitionFilterOptions: vi.fn(),
}));
const api = vi.mocked(getDefinitionFilterOptions);
const option = (index: number): DefinitionFilterOptionView => ({
  target: { kind: "definition", key: `language:token-${index}` },
  label: index < 2 ? "Shared label" : `Language ${index}`,
  help: {
    definition_key: `language:token-${index}`,
    availability: "label_only",
    destination: null,
    related_filters: [],
  },
});
function wrapper({ children }: { children: ReactNode }) {
  return (
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      {children}
    </QueryClientProvider>
  );
}
beforeEach(() => vi.clearAllMocks());

it("pages the complete artifact vocabulary including distinct equal labels and retained zero-match selections", async () => {
  const entries = Array.from({ length: 2549 }, (_, index) => option(index));
  api.mockImplementation(async (request) => ({
    artifact_generation: "generation-a",
    options: entries.slice(request.offset, request.offset + request.limit),
    selected: request.selected.map((target) =>
      entries.find((entry) => JSON.stringify(entry.target) === JSON.stringify(target))!,
    ),
    total: 2549n,
  }));
  const { result, rerender } = renderHook(
    ({ offset }) =>
      useDefinitionFilterOptions({
        generation: "generation-a",
        text: "",
        offset,
        selected: [entries[0].target],
        enabled: true,
      }),
    { initialProps: { offset: 0 }, wrapper },
  );
  const discovered: string[] = [];
  for (let offset = 0; offset < entries.length; offset += 40) {
    rerender({ offset });
    await waitFor(() => expect(result.current.data?.offset).toBe(offset));
    discovered.push(
      ...result.current.data!.options.map((entry) => JSON.stringify(entry.target)),
    );
    expect(result.current.data!.selected).toEqual([entries[0]]);
  }
  expect(new Set(discovered).size).toBe(2549);
  expect(
    api.mock.calls.every(
      ([request]) =>
        request.limit === 40 && request.expected_artifact_generation === "generation-a",
    ),
  ).toBe(true);
  expect(api.mock.calls).toHaveLength(64);
});

it.each(["generation", "selection"])(
  "rejects a mismatched %s response",
  async (mismatch) => {
    api.mockResolvedValue({
      artifact_generation: mismatch === "generation" ? "generation-b" : "generation-a",
      options: [],
      selected: [option(1)],
      total: 0n,
    });
    const { result } = renderHook(
      () =>
        useDefinitionFilterOptions({
          generation: "generation-a",
          text: "",
          offset: 0,
          selected: [option(0).target],
          enabled: true,
        }),
      { wrapper },
    );
    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(result.current.data).toBeUndefined();
    if (mismatch === "generation")
      expect(result.current.error).toBeInstanceOf(AtlasApiError);
  },
);
