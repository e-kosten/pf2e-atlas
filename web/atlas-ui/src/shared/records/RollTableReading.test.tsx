import { fireEvent, render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { expect, it, vi } from "vitest";
import { RollTableReading } from "./RollTableReading";
import { knownFact, unavailableFact } from "../../test/fixtures";
import type { RecordNavigationView } from "../../generated/atlas";
const api = vi.hoisted(() => ({ getOwnedReading: vi.fn() }));
vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  ...api,
}));

it("reads a batch of complete table text and opens its checked reference in place", async () => {
  const parent: RecordNavigationView = {
    record_key: "tables:one",
    owners: [],
    field: null,
    passage: null,
    source_fingerprint: null,
  };
  const navigation: RecordNavigationView = {
    ...parent,
    owners: [{ collection: "/results", identity: { Stable: "result" } }],
  };
  const target: RecordNavigationView = { ...parent, record_key: "items:staff" };
  api.getOwnedReading.mockResolvedValue({
    artifact_generation: "generation",
    parent,
    availability: "value",
    total: 1,
    start: 0,
    end: 1,
    previous: null,
    next: null,
    entries: [
      {
        navigation,
        content: [
          {
            locator: {
              record: parent.record_key,
              owners: navigation.owners,
              field: "/text",
            },
            role: "table_result",
            source_fingerprint: null,
            body: { kind: "html", html: "<p>Staff of Fire</p>", controls: [] },
          },
        ],
        relationships: [{ kind: "TableResult", target }],
      },
    ],
  });
  const onReference = vi.fn();
  render(
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <RollTableReading
        context={{ generation: "generation", parent }}
        selection={parent}
        onReference={onReference}
        table={{
          formula: knownFact("1d20"),
          results: knownFact([
            {
              range: knownFact([1, 20]),
              link: {
                navigation,
                title: "Result 1–20",
                family: "TableResult",
                usage: null,
                action_cost: unavailableFact("not_applicable"),
              },
            },
          ]),
        }}
      />
    </QueryClientProvider>,
  );
  const link = await screen.findByRole("link", { name: "Staff of Fire" });
  expect(screen.getByRole("columnheader", { name: "Range" })).toBeVisible();
  expect(screen.getByRole("columnheader", { name: "Result" })).toBeVisible();
  expect(screen.queryByText("Open record")).not.toBeInTheDocument();
  fireEvent.click(link);
  expect(onReference).toHaveBeenCalledWith(
    "items:staff",
    expect.any(HTMLElement),
    target,
  );
  expect(api.getOwnedReading).toHaveBeenCalledTimes(1);
  expect(api.getOwnedReading.mock.calls[0][0].selection).toEqual({
    kind: "table_results",
    cursor: null,
  });
});
