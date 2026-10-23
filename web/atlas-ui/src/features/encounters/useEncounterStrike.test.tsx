import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { ConfigProvider } from "antd";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useEncounterStrike } from "./useEncounterStrike";
import type { EncounterParticipantView } from "../../generated/atlas";
import type { useEncounterMutations } from "./useEncounterMutations";
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
function Harness() {
  const strike = useEncounterStrike({
    encounterRef: "encounter",
    revision: 4n,
    generation: "generation",
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
            rollId: "original",
          })
        }
      >
        Target & checks
      </button>
      {strike.renderContext("original")}
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
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <QueryClientProvider
        client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
      >
        <Harness />
      </QueryClientProvider>
    </ConfigProvider>,
  );
  fireEvent.click(screen.getByRole("button", { name: "Target & checks" }));
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
