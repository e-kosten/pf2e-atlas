import { fireEvent, render, screen, waitFor } from "@testing-library/react";
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
    expression: {
      kind: "binary" as const,
      operator: "add" as const,
      left: { kind: "dice" as const, sides: [8, 8, 8, 8] },
      right: { kind: "number" as const, value: 10 },
    },
  },
];
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
      <ContentRoll request={request} components={components} />
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
  const dialog = screen.getByRole("dialog", { name: "4d8+10 bludgeoning damage" });
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
