import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { ConfigProvider } from "antd";
import { ContentRoll } from "./ContentRoll";
import { rollContent } from "../../api/atlasApi";
import type { ContentRollResultView, RollContentRequest } from "../../generated/atlas";

vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  rollContent: vi.fn(),
}));
const request: RollContentRequest = {
  content_interpretation_version: "foundry-content/v12",
  source: {
    record: "items:example",
    embedded_path: [],
    field: "/system/description/value",
  },
  source_fingerprint: "current",
  ordinal: 0,
  authored_command: null,
  authored_formula: "(4d8+10)[bludgeoning]",
};
const components = [
  {
    formula: "4d8+10",
    label: "bludgeoning damage",
    unit: null,
    expression: {
      kind: "binary" as const,
      operator: "add" as const,
      left: { kind: "dice" as const, sides: [8, 8, 8, 8] },
      right: { kind: "number" as const, value: 10 },
    },
  },
];
const presentation = { heading: "Bludgeoning damage", components };
const outcome: ContentRollResultView = {
  components: [
    {
      label: "bludgeoning damage",
      outcome: {
        formula: "4d8+10",
        total: 28,
        expression: {
          kind: "binary",
          operator: "add",
          left: {
            kind: "dice",
            dice: [2, 4, 5, 7].map((value) => ({ sides: 8, value, kept: true })),
            total: 18,
          },
          right: { kind: "number", value: 10 },
          total: 28,
        },
      },
    },
  ],
};
function show() {
  return render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <ContentRoll request={request} presentation={presentation} />
    </ConfigProvider>,
  );
}
beforeEach(() => vi.mocked(rollContent).mockReset());

it("rolls once on activation, retains the result through dismissal and only rerolls explicitly", async () => {
  vi.mocked(rollContent).mockResolvedValue(outcome);
  show();
  const opener = screen.getByRole("button", { name: "Roll 4d8+10 bludgeoning damage" });
  fireEvent.click(opener);
  await screen.findByRole("status", { name: "4d8+10: 28" });
  expect(rollContent).toHaveBeenCalledExactlyOnceWith(request);
  const dialog = screen.getByRole("dialog", { name: "Bludgeoning damage" });
  fireEvent.keyDown(dialog, { key: "Escape" });
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  expect(opener).toHaveFocus();
  fireEvent.click(
    screen.getByRole("button", { name: "View roll for 4d8+10 bludgeoning damage" }),
  );
  await screen.findByRole("status", { name: "4d8+10: 28" });
  expect(rollContent).toHaveBeenCalledTimes(1);
  fireEvent.click(screen.getByRole("button", { name: "Reroll" }));
  await waitFor(() => expect(rollContent).toHaveBeenCalledTimes(2));
});

it("shows rejected source requests locally and retries only on an explicit action", async () => {
  const { AtlasApiError } = await import("../../api/atlasApi");
  vi.mocked(rollContent).mockRejectedValue(
    new AtlasApiError(400, "This source changed. Reopen the record before rolling."),
  );
  show();
  fireEvent.click(
    screen.getByRole("button", { name: "Roll 4d8+10 bludgeoning damage" }),
  );
  expect(await screen.findByRole("alert")).toHaveTextContent("This source changed");
  fireEvent.click(screen.getByRole("button", { name: "Close roll" }));
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  fireEvent.click(
    screen.getByRole("button", { name: "Roll 4d8+10 bludgeoning damage" }),
  );
  await screen.findByRole("alert");
  expect(rollContent).toHaveBeenCalledTimes(1);
  vi.mocked(rollContent).mockResolvedValue(outcome);
  fireEvent.click(screen.getByRole("button", { name: "Retry" }));
  await screen.findByRole("status", { name: "4d8+10: 28" });
  expect(rollContent).toHaveBeenCalledTimes(2);
});

it("retains keyboard focus while rerolling and ignores repeated activation until completion", async () => {
  vi.mocked(rollContent).mockResolvedValue(outcome);
  show();
  fireEvent.click(
    screen.getByRole("button", { name: "Roll 4d8+10 bludgeoning damage" }),
  );
  await screen.findByRole("status", { name: "4d8+10: 28" });
  let complete!: (value: ContentRollResultView) => void;
  vi.mocked(rollContent).mockImplementationOnce(
    () =>
      new Promise((resolve) => {
        complete = resolve;
      }),
  );
  const reroll = screen.getByRole("button", { name: "Reroll" });
  reroll.focus();
  fireEvent.click(reroll);
  expect(reroll).toHaveAttribute("aria-disabled", "true");
  expect(reroll).not.toBeDisabled();
  expect(reroll).toHaveFocus();
  fireEvent.click(reroll);
  expect(rollContent).toHaveBeenCalledTimes(2);
  complete(outcome);
  await waitFor(() => expect(reroll).toHaveAttribute("aria-disabled", "false"));
  expect(reroll).toHaveFocus();
});

it("promotes an authored purpose to the stable dialog heading without repeating it in the body", async () => {
  const { AtlasApiError } = await import("../../api/atlasApi");
  let reject!: (cause: unknown) => void;
  vi.mocked(rollContent).mockImplementationOnce(
    () =>
      new Promise((_resolve, fail) => {
        reject = fail;
      }),
  );
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <ContentRoll
        request={request}
        presentation={{
          ...presentation,
          heading: "Counteract",
          components: [{ ...components[0], label: "Counteract" }],
        }}
      />
    </ConfigProvider>,
  );
  fireEvent.click(screen.getByRole("button", { name: "Roll 4d8+10 Counteract" }));
  const dialog = await screen.findByRole("dialog", { name: "Counteract" });
  expect(screen.getAllByText("Counteract", { exact: true })).toHaveLength(1);
  expect(screen.queryByText("Roll result")).toBeNull();
  await act(async () => {
    reject(new AtlasApiError(400, "This source changed."));
  });
  await screen.findByRole("alert");
  expect(screen.getByRole("dialog", { name: "Counteract" })).toBe(dialog);
  expect(screen.getAllByText("Counteract", { exact: true })).toHaveLength(1);
  fireEvent.click(screen.getByRole("button", { name: "Close roll" }));
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
});

it.each([1, 3])("shows time units beside a duration total of %s", async (value) => {
  const unit = { singular: "round", plural: "rounds" };
  vi.mocked(rollContent).mockResolvedValue({
    components: [
      {
        label: "rounds",
        outcome: {
          formula: "1d4",
          total: value,
          expression: {
            kind: "dice",
            dice: [{ sides: 4, value, kept: true }],
            total: value,
          },
        },
      },
    ],
  });
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <ContentRoll
        request={request}
        presentation={{
          heading: "Duration",
          components: [
            {
              formula: "1d4",
              label: "rounds",
              unit,
              expression: { kind: "dice", sides: [4] },
            },
          ],
        }}
      />
    </ConfigProvider>,
  );
  fireEvent.click(screen.getByRole("button", { name: "Roll 1d4 rounds" }));
  const suffix = value === 1 ? "round" : "rounds";
  await screen.findByRole("status", { name: `1d4: ${value} ${suffix}` });
  expect(screen.getByRole("dialog", { name: "Duration" })).toBeVisible();
  expect(screen.getAllByText(suffix, { exact: true })).toHaveLength(1);
});

it("retains type labels that distinguish multiple damage components", async () => {
  vi.mocked(rollContent).mockResolvedValue({
    components: ["fire damage", "cold damage"].map((label) => ({
      ...outcome.components[0],
      label,
    })),
  });
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <ContentRoll
        request={request}
        presentation={{
          heading: "Damage",
          components: [
            { ...components[0], label: "fire damage" },
            { ...components[0], label: "cold damage" },
          ],
        }}
      />
    </ConfigProvider>,
  );
  fireEvent.click(
    screen.getByRole("button", {
      name: "Roll 4d8+10 fire damage + 4d8+10 cold damage",
    }),
  );
  await screen.findByRole("dialog", { name: "Damage" });
  await screen.findAllByRole("status", { name: "4d8+10: 28" });
  expect(screen.getByText("fire damage", { exact: true })).toBeVisible();
  expect(screen.getByText("cold damage", { exact: true })).toBeVisible();
});
