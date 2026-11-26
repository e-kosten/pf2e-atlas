import { fireEvent, render, screen, within } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { expect, it, vi } from "vitest";
import { RollTableReading } from "./RollTableReading";
import { knownFact, unavailableFact } from "../../test/fixtures";
import type { RecordNavigationView } from "../../generated/atlas";
const api = vi.hoisted(() => ({ getEmbeddedReading: vi.fn(), rollTable: vi.fn() }));
vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  ...api,
}));

const parent: RecordNavigationView = {
  record_key: "tables:one",
  embedded_path: [],
  field: null,
  passage: null,
  source_fingerprint: null,
};
function resultRow(id: string, range: [number, number], authoredName?: string) {
  return {
    authored_name: authoredName,
    range: knownFact(range),
    link: {
      navigation: {
        ...parent,
        embedded_path: [{ collection: "/results", identity: { Stable: id } }],
      },
      title: `Result ${range[0]}–${range[1]}`,
      family: "TableResult",
      usage: null,
      action_cost: unavailableFact<never>("not_applicable"),
    },
  };
}
it.each([
  { authoredName: "Pyromancer’s Staff", description: "", label: "Pyromancer’s Staff" },
  {
    authoredName: undefined,
    description: "<p>Staff of Fire</p>",
    label: "Staff of Fire",
  },
])(
  "opens $label via the checked table reference",
  async ({ authoredName, description, label }) => {
    const row = resultRow("result", [1, 20], authoredName);
    const navigation = row.link.navigation;
    const target: RecordNavigationView = { ...parent, record_key: "items:staff" };
    api.getEmbeddedReading.mockResolvedValue({
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
                embedded_path: navigation.embedded_path,
                field: "/description",
              },
              role: "table_result",
              source_fingerprint: null,
              body: {
                kind: "html",
                inclusions: [],
                html: description,
                controls: [],
              },
            },
          ],
          relationships: [
            { kind: "TableResult", destination: { kind: "resolved_record", target } },
          ],
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
            roll_note: "Reference rolls may repeat; results are not consumed.",
            formula: knownFact("1d20"),
            results: knownFact([row]),
          }}
        />
      </QueryClientProvider>,
    );
    const link = await screen.findByRole("link", { name: label });
    expect(link).toHaveAttribute("href", "/records/items:staff");
    expect(screen.getByRole("columnheader", { name: "Range" })).toBeVisible();
    expect(screen.getByRole("columnheader", { name: "Result" })).toBeVisible();
    expect(screen.queryByText("Open record")).not.toBeInTheDocument();
    fireEvent.click(link);
    expect(onReference).toHaveBeenCalledWith(
      "items:staff",
      expect.any(HTMLElement),
      target,
    );
    expect(api.getEmbeddedReading).toHaveBeenCalledTimes(1);
    expect(api.getEmbeddedReading.mock.calls[0][0].selection).toEqual({
      kind: "table_results",
      cursor: null,
    });
  },
);

it("keeps unavailable names readable and shows a named roll beyond the loaded page with its prepared content", async () => {
  const missing = resultRow("missing", [1, 1], "Lost Staff");
  const potion = resultRow("potion", [2, 2]);
  const rolled = resultRow("rolled", [3, 3], "Pyromancer’s Staff");
  const target = { ...parent, record_key: "items:staff" };
  api.getEmbeddedReading.mockResolvedValue({
    parent,
    availability: "value",
    next: { artifact_generation: "generation", offset: 2 },
    entries: [
      {
        navigation: missing.link.navigation,
        relationships: [{ kind: "TableResult", destination: { kind: "unresolved" } }],
        content: [],
      },
      {
        navigation: potion.link.navigation,
        relationships: [],
        content: [
          {
            locator: {
              record: parent.record_key,
              embedded_path: potion.link.navigation.embedded_path,
              field: "/description",
            },
            role: "table_result",
            body: { kind: "plain", text: "Potion of Resistance (Moderate)" },
          },
        ],
      },
    ],
  });
  api.rollTable.mockResolvedValue({
    outcome: { formula: "3", total: 3, expression: { kind: "number", value: 3 } },
    invalid_ranges: [],
    matches: [
      {
        navigation: rolled.link.navigation,
        title: rolled.link.title,
        relationships: [
          { kind: "TableResult", destination: { kind: "resolved_record", target } },
        ],
        content: [
          {
            locator: {
              record: parent.record_key,
              embedded_path: rolled.link.navigation.embedded_path,
              field: "/description",
            },
            role: "table_result",
            body: {
              kind: "html",
              inclusions: [],
              html: '<p><strong>On activation</strong> attempt a <span data-atlas-interaction="0">Reflex save</span>.</p>',
              controls: [
                {
                  ordinal: 0,
                  control: {
                    kind: "check",
                    statistic: "reflex",
                    options: {},
                    details: { traits: [] },
                  },
                },
              ],
            },
          },
        ],
      },
    ],
  });
  const onReference = vi.fn();
  render(
    <QueryClientProvider client={new QueryClient()}>
      <RollTableReading
        selection={parent}
        context={{ generation: "generation", parent }}
        onReference={onReference}
        table={{
          roll_note: "Rolls may repeat.",
          formula: knownFact("3"),
          results: knownFact([missing, potion, rolled]),
        }}
      />
    </QueryClientProvider>,
  );
  const table = within(screen.getByRole("table"));
  expect(await table.findByText("Lost Staff")).toBeVisible();
  expect(table.getByText("Potion of Resistance (Moderate)")).toBeVisible();
  expect(table.queryByRole("link")).not.toBeInTheDocument();
  expect(screen.getByRole("button", { name: "Load more results" })).toBeVisible();
  fireEvent.click(screen.getByRole("button", { name: "Roll" }));
  const link = await screen.findByRole("link", { name: "Pyromancer’s Staff" });
  expect(screen.getByText(rolled.link.title)).toBeVisible();
  expect(table.queryByText("Pyromancer’s Staff")).not.toBeInTheDocument();
  expect(screen.getByText("On activation", { selector: "strong" })).toBeVisible();
  expect(
    screen.getByRole("button", { name: "Check details: Reflex save" }),
  ).toBeVisible();
  expect(screen.queryByText("Open record")).not.toBeInTheDocument();
  fireEvent.click(link);
  expect(onReference).toHaveBeenCalledWith("items:staff", link, target);
  expect(api.getEmbeddedReading).toHaveBeenCalledTimes(1);
});

it.each([
  ["missing", "1d6", "Table results unavailable", false],
  ["null", "1d6", "Table results unavailable", false],
  ["invalid", "1d6", "Table results unavailable", false],
  ["value", "1d6", "No results.", true],
  ["value", null, "No results.", false],
] as const)(
  "distinguishes %s results with formula %s",
  async (availability, formula, emptyText, canRoll) => {
    const parent: RecordNavigationView = {
      record_key: "tables:incomplete",
      embedded_path: [],
      field: null,
      passage: null,
      source_fingerprint: null,
    };
    api.getEmbeddedReading.mockResolvedValue({
      artifact_generation: "generation",
      parent,
      availability,
      entries: [],
      next: null,
    });
    render(
      <QueryClientProvider client={new QueryClient()}>
        <RollTableReading
          selection={parent}
          context={{ generation: "generation", parent }}
          onReference={vi.fn()}
          table={{
            roll_note: "Rolls may repeat.",
            formula: formula ? knownFact(formula) : unavailableFact("missing"),
            results:
              availability === "value" ? knownFact([]) : unavailableFact(availability),
          }}
        />
      </QueryClientProvider>,
    );
    expect(await screen.findByText(emptyText)).toBeVisible();
    const roll = screen.getByRole("button", { name: "Roll" });
    if (canRoll) expect(roll).toBeEnabled();
    else expect(roll).toBeDisabled();
    if (!formula) expect(screen.getByText("Roll formula unavailable")).toBeVisible();
  },
);
