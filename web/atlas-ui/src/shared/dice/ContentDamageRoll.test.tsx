import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { ConfigProvider } from "antd";
import { ContentDamageRoll } from "./ContentDamageRoll";
import { rollContentDamage } from "../../api/atlasApi";
import type {
  ContentDamageRollView,
  RollContentDamageRequest,
} from "../../generated/atlas";

vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  rollContentDamage: vi.fn(),
}));
const request: RollContentDamageRequest = {
  content_interpretation_version: "foundry-content/v11",
  source: {
    record: "items:example",
    embedded_path: [],
    field: "/system/description/value",
  },
  source_fingerprint: "current",
  ordinal: 0,
  authored_formula: "(4d8+10)[bludgeoning]",
};
const components = [
  {
    formula: "4d8+10",
    label: "bludgeoning damage",
    expression: {
      kind: "binary" as const,
      operator: "add" as const,
      left: { kind: "dice" as const, sides: [8, 8, 8, 8] },
      right: { kind: "number" as const, value: 10 },
    },
  },
];
const outcome: ContentDamageRollView = {
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
      <ContentDamageRoll request={request} components={components} />
    </ConfigProvider>,
  );
}
beforeEach(() => vi.mocked(rollContentDamage).mockReset());

it("rolls once on activation, retains the result through dismissal and only rerolls explicitly", async () => {
  vi.mocked(rollContentDamage).mockResolvedValue(outcome);
  show();
  const opener = screen.getByRole("button", { name: "Roll 4d8+10 bludgeoning damage" });
  fireEvent.click(opener);
  await screen.findByRole("status", { name: "4d8+10: 28" });
  expect(rollContentDamage).toHaveBeenCalledExactlyOnceWith(request);
  const dialog = screen.getByRole("dialog", { name: "4d8+10 bludgeoning damage" });
  fireEvent.keyDown(dialog, { key: "Escape" });
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  expect(opener).toHaveFocus();
  fireEvent.click(
    screen.getByRole("button", { name: "View roll for 4d8+10 bludgeoning damage" }),
  );
  await screen.findByRole("status", { name: "4d8+10: 28" });
  expect(rollContentDamage).toHaveBeenCalledTimes(1);
  fireEvent.click(screen.getByRole("button", { name: "Reroll" }));
  await waitFor(() => expect(rollContentDamage).toHaveBeenCalledTimes(2));
});

it("shows rejected source requests locally and retries only on an explicit action", async () => {
  const { AtlasApiError } = await import("../../api/atlasApi");
  vi.mocked(rollContentDamage).mockRejectedValue(
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
  expect(rollContentDamage).toHaveBeenCalledTimes(1);
  vi.mocked(rollContentDamage).mockResolvedValue(outcome);
  fireEvent.click(screen.getByRole("button", { name: "Retry" }));
  await screen.findByRole("status", { name: "4d8+10: 28" });
  expect(rollContentDamage).toHaveBeenCalledTimes(2);
});
