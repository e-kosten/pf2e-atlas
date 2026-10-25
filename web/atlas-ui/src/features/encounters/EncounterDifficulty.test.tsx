import { ConfigProvider } from "antd";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { EncounterDifficulty } from "./EncounterDifficulty";
import type { EncounterDifficultyView } from "../../generated/atlas";
import { previewEncounterDifficulty } from "../../api/atlasApi";
vi.mock("../../api/atlasApi", () => ({ previewEncounterDifficulty: vi.fn() }));
const initial: EncounterDifficultyView = {
  reference: { party_level: null, party_size: null },
  party_level: 5,
  party_size: 4,
  pc_count: 4,
  basis: "From encounter PC levels.",
  reference_note: null,
  total_xp: 80,
  known_xp: 80,
  rating: "Moderate",
  budgets: [{ label: "Moderate", xp: 80 }],
  opposition: [
    {
      participant_key: "a",
      name: "Enemy",
      kind: "Creature",
      level: 7,
      xp: 80,
      unavailable_reason: null,
    },
  ],
};
function setup(onSave = vi.fn().mockResolvedValue(undefined), value = initial) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <QueryClientProvider client={client}>
        <EncounterDifficulty
          encounterKey="encounter"
          revision={9n}
          value={value}
          onPreview={(request) => previewEncounterDifficulty("encounter", request)}
          disabled={false}
          onSave={onSave}
        />
      </QueryClientProvider>
    </ConfigProvider>,
  );
  fireEvent.click(screen.getByRole("button", { name: /Encounter difficulty:/ }));
  return { onSave };
}
it("previews edits without saving, and submits one explicit reference", async () => {
  let resolve!: (value: EncounterDifficultyView) => void;
  vi.mocked(previewEncounterDifficulty).mockImplementation(
    () =>
      new Promise((r) => {
        resolve = r;
      }),
  );
  const { onSave } = setup();
  const input = screen.getByRole("spinbutton", { name: "Party level" });
  fireEvent.change(input, { target: { value: "6" } });
  await waitFor(() =>
    expect(previewEncounterDifficulty).toHaveBeenCalledWith("encounter", {
      expected_revision: 9n,
      reference: { party_level: 6, party_size: null },
    }),
  );
  expect(screen.getByRole("button", { name: "Save" })).toBeDisabled();
  expect(onSave).not.toHaveBeenCalled();
  resolve({ ...initial, party_level: 6, total_xp: 60, rating: "Low" });
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Save" })).toBeEnabled(),
  );
  fireEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() => expect(onSave).toHaveBeenCalledTimes(1));
  expect(onSave).toHaveBeenCalledWith({ party_level: 6, party_size: null });
});
it("keeps a failed preview local and offers retry without enabling save", async () => {
  vi.mocked(previewEncounterDifficulty).mockRejectedValue(new Error("Disconnected"));
  const { onSave } = setup();
  fireEvent.change(screen.getByRole("spinbutton", { name: "PC count" }), {
    target: { value: "5" },
  });
  await waitFor(() =>
    expect(screen.getByText("Difficulty preview could not update")).toBeVisible(),
  );
  expect(screen.getByText("Previous calculation · update failed")).toBeVisible();
  expect(screen.getByRole("button", { name: "Save" })).toBeDisabled();
  vi.mocked(previewEncounterDifficulty).mockResolvedValue({
    ...initial,
    party_size: 5,
  });
  fireEvent.click(screen.getByRole("button", { name: "Retry" }));
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Save" })).toBeEnabled(),
  );
  expect(onSave).not.toHaveBeenCalled();
});
it("does not present missing opponent XP as a complete total", async () => {
  setup(undefined, {
    ...initial,
    total_xp: null,
    rating: null,
    opposition: [
      {
        ...initial.opposition[0],
        xp: null,
        unavailable_reason: "Participant level is unavailable.",
      },
    ],
  });
  expect(
    screen.getByRole("button", { name: "Encounter difficulty: Difficulty incomplete" }),
  ).toBeVisible();
  await waitFor(() =>
    expect(screen.getByText(/Participant level is unavailable/)).toBeVisible(),
  );
});
