import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { ConfigProvider } from "antd";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useEncounterStrike } from "./useEncounterStrike";
import type { EncounterParticipantView } from "../../generated/atlas";
import type { useEncounterMutations } from "./useEncounterMutations";
import { AtlasApiError } from "../../api/atlasApi";
const api = vi.hoisted(() => ({
  previewEncounterStrikeContext: vi.fn(),
  rollFlatCheck: vi.fn(),
}));
vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  ...api,
}));
const reference = {
  artifact_generation: "generation",
  selection: {
    record_key: "creatures:actor",
    embedded_path: [],
    fields: [],
    passage: null,
    source_fingerprint: null,
  },
};
const participants = [
  { participant_key: "actor", display_name: "Attacker" },
  { participant_key: "target", display_name: "Target creature" },
] as EncounterParticipantView[];
function Harness({
  revision = 4n,
  generation = "generation",
  rollId = "original",
} = {}) {
  const strike = useEncounterStrike({
    encounterRef: "encounter",
    revision,
    generation,
    participants,
    mutations: { pending: false, refreshError: null } as ReturnType<
      typeof useEncounterMutations
    >,
  });
  return (
    <>
      <button
        onClick={() =>
          strike.launch({
            strike: { participant_key: "actor", reference },
            attackNumber: 2,
            rollId,
          })
        }
      >
        Target & checks
      </button>
      {strike.renderContext(rollId)}
    </>
  );
}
function setup(overrides = {}) {
  api.previewEncounterStrikeContext.mockResolvedValue({
    options: { attacks: [], unavailable: null },
    targeting_check: { dc: 11n, sources: [{ condition_id: 4n, name: "Undetected" }] },
    action: {
      issues: [],
      confused: false,
      disruption: null,
      targeting: {
        requires_guessed_location: true,
        check_senses: false,
        prevents_effect: false,
      },
    },
    ...overrides,
  });
  api.rollFlatCheck.mockResolvedValue({
    dc: 11n,
    success: true,
    outcome: { formula: "1d20", total: 15, expression: { kind: "number", value: 15 } },
  });
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const view = render(<Harness />, {
    wrapper: ({ children }) => (
      <ConfigProvider theme={{ token: { motion: false } }}>
        <QueryClientProvider client={client}>{children}</QueryClientProvider>
      </ConfigProvider>
    ),
  });
  fireEvent.click(screen.getByRole("button", { name: "Target & checks" }));
  return { ...view, client };
}
it("offers reference checks without requiring guessed-location answers or committing an attack", async () => {
  setup();
  await screen.findByText("Undetected · DC 11 flat check");
  expect(
    screen.queryByRole("button", { name: "Resolve checks" }),
  ).not.toBeInTheDocument();
  expect(screen.queryByRole("checkbox")).not.toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "Roll" }));
  await waitFor(() => expect(api.rollFlatCheck).toHaveBeenCalledWith({ dc: 11n }));
  await screen.findByText("Success");
  expect(api.previewEncounterStrikeContext).toHaveBeenCalledWith(
    "encounter",
    expect.objectContaining({ target_key: null }),
  );
});
it("excludes the attacker from ordinary target selection", async () => {
  setup();
  const target = await screen.findByRole("combobox", { name: "Strike target" });
  fireEvent.mouseDown(target);
  expect(await screen.findByText("Target creature")).toBeInTheDocument();
  expect(screen.queryByText("Attacker")).not.toBeInTheDocument();
  fireEvent.click(screen.getByText("Target creature"));
  await waitFor(() =>
    expect(api.previewEncounterStrikeContext).toHaveBeenLastCalledWith(
      "encounter",
      expect.objectContaining({
        target_key: "target",
      }),
    ),
  );
});

it("does not ask for detection when only the untracked relationship is unknown", async () => {
  setup({
    targeting_check: null,
    action: {
      issues: [],
      confused: false,
      disruption: null,
      targeting: {
        requires_guessed_location: false,
        prevents_effect: false,
        check_senses: false,
      },
    },
  });
  const target = await screen.findByRole("combobox", { name: "Strike target" });
  fireEvent.mouseDown(target);
  fireEvent.click(await screen.findByText("Target creature"));
  await screen.findByText("No additional checks from tracked conditions.");
  expect(screen.queryByText("Check detection.")).not.toBeInTheDocument();
});
it("keeps the usable-senses reminder when a condition needs it", async () => {
  setup({
    targeting_check: null,
    action: {
      issues: [],
      confused: false,
      disruption: null,
      targeting: {
        requires_guessed_location: false,
        prevents_effect: false,
        check_senses: true,
      },
    },
  });
  await screen.findByText("Check detection · Consider usable senses and range.");
  expect(
    screen.queryByText("No additional checks from tracked conditions."),
  ).not.toBeInTheDocument();
});

it("keeps same-target checks and dice results mounted through a revision refresh and failure", async () => {
  const view = setup();
  await screen.findByText("Undetected · DC 11 flat check");
  fireEvent.click(screen.getByRole("button", { name: "Roll" }));
  await screen.findByText("Success");
  const target = screen.getByRole("combobox", { name: "Strike target" });
  const rollCount = api.rollFlatCheck.mock.calls.length;
  const reroll = screen.getByRole("button", { name: "Reroll" });
  let reject!: (cause: Error) => void;
  api.previewEncounterStrikeContext.mockImplementationOnce(
    () =>
      new Promise((_resolve, fail) => {
        reject = fail;
      }),
  );
  view.rerender(<Harness revision={5n} />);
  expect(screen.getByRole("combobox", { name: "Strike target" })).toBe(target);
  expect(screen.getByText("Success")).toBeInTheDocument();
  expect(reroll).toBeDisabled();
  await act(async () => reject(new Error("Unavailable")));
  await screen.findByText(/Previous checks shown\./);
  expect(screen.getByText("Success")).toBeInTheDocument();
  expect(reroll).toBeDisabled();
  fireEvent.click(screen.getByRole("button", { name: "Retry" }));
  await waitFor(() => expect(reroll).toBeEnabled());
  expect(api.rollFlatCheck).toHaveBeenCalledTimes(rollCount);
});

it("keeps the target selector mounted but never lends checks to another target or generation", async () => {
  const view = setup();
  await screen.findByText("Undetected · DC 11 flat check");
  api.previewEncounterStrikeContext.mockImplementation(() => new Promise(() => {}));
  const target = screen.getByRole("combobox", { name: "Strike target" });
  fireEvent.mouseDown(target);
  fireEvent.click(await screen.findByText("Target creature"));
  expect(screen.getByRole("combobox", { name: "Strike target" })).toBe(target);
  expect(screen.queryByText("Undetected · DC 11 flat check")).not.toBeInTheDocument();
  view.rerender(<Harness generation="replacement" />);
  expect(
    screen.queryByRole("combobox", { name: "Strike target" }),
  ).not.toBeInTheDocument();
});

it("retires obsolete checks even if another read failure replaces the generation error", async () => {
  const { client, rerender } = setup();
  await screen.findByText("Undetected · DC 11 flat check");
  api.previewEncounterStrikeContext.mockRejectedValueOnce(
    new AtlasApiError(409, "Changed", {
      code: "artifact_generation_changed",
      message: "Changed",
    }),
  );
  await act(async () => {
    await client.invalidateQueries({ queryKey: ["strike-assistance"] });
  });
  await screen.findByRole("button", { name: "Reload encounter" });
  expect(screen.queryByText("Undetected · DC 11 flat check")).not.toBeInTheDocument();
  api.previewEncounterStrikeContext.mockRejectedValueOnce(new Error("Unavailable"));
  await act(async () => {
    await client.refetchQueries({ queryKey: ["strike-assistance"] });
  });
  expect(screen.queryByText("Undetected · DC 11 flat check")).not.toBeInTheDocument();
  expect(screen.getByRole("button", { name: "Reload encounter" })).toBeInTheDocument();
  act(() => {
    client.setQueriesData(
      { queryKey: ["strike-assistance"] },
      (previous: unknown) => previous,
    );
  });
  rerender(<Harness rollId="another" />);
  fireEvent.click(screen.getByRole("button", { name: "Target & checks" }));
  expect(
    await screen.findByRole("button", { name: "Reload encounter" }),
  ).toBeInTheDocument();
});
