import { ConfigProvider } from "antd";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { EncounterDifficulty } from "./EncounterDifficulty";
import type { EncounterDifficultyView } from "../../generated/atlas";
import { previewEncounterDifficulty } from "../../api/atlasApi";
vi.mock("../../api/atlasApi", () => ({ previewEncounterDifficulty: vi.fn() }));
const initial: EncounterDifficultyView = {
  rules: [],
  reference: { party_level: null, party_size: null },
  party_level: 5,
  party_size: 4,
  pc_count: 4,
  basis: "From encounter PC levels.",
  reference_note: null,
  total_xp: 80,
  known_xp: 80,
  rating: "Moderate",
  budgets: [
    { label: "Trivial", xp: 40 },
    { label: "Low", xp: 60 },
    { label: "Moderate", xp: 80 },
    { label: "Severe", xp: 120 },
    { label: "Extreme", xp: 160 },
  ],
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
function setup(
  onSave = vi.fn().mockResolvedValue(undefined),
  value = initial,
  onReference = vi.fn(),
) {
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
          onReference={onReference}
        />
      </QueryClientProvider>
    </ConfigProvider>,
  );
  fireEvent.click(screen.getByRole("button", { name: /Encounter difficulty:/ }));
  return { onSave, onReference };
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
  expect(screen.queryByRole("progressbar")).toBeNull();
  expect(
    screen.getByRole("button", { name: "Encounter difficulty: Difficulty incomplete" }),
  ).toBeVisible();
  await waitFor(() =>
    expect(screen.getByText(/Participant level is unavailable/)).toBeVisible(),
  );
});

it.each([
  ["Party level", "ArrowUp", 38, { party_level: 6, party_size: null }],
  ["Party level", "ArrowDown", 40, { party_level: 4, party_size: null }],
  ["PC count", "ArrowUp", 38, { party_level: null, party_size: 5 }],
])(
  "steps from the automatic value for %s %s",
  async (label, key, keyCode, reference) => {
    vi.mocked(previewEncounterDifficulty).mockResolvedValue(initial);
    setup();
    const input = screen.getByRole("spinbutton", { name: label });
    expect(input).toHaveValue(label === "Party level" ? "Auto: 5" : "Auto: 4");
    fireEvent.keyDown(input, { key, code: key, keyCode });
    await waitFor(() =>
      expect(previewEncounterDifficulty).toHaveBeenCalledWith("encounter", {
        expected_revision: 9n,
        reference,
      }),
    );
  },
);
it("uses an XP meter with a text alternative and leaves unknown totals unfilled", async () => {
  setup();
  await waitFor(() =>
    expect(screen.getByRole("progressbar", { name: "Encounter XP" })).toBeVisible(),
  );
  expect(screen.getByRole("progressbar", { name: "Encounter XP" })).toHaveAttribute(
    "aria-valuetext",
    "Moderate, 80 XP; Extreme budget 160 XP",
  );
  expect(screen.queryByText(/Includes defeated/)).not.toBeInTheDocument();
});

it("uses an exact native rules destination and retains the draft after reading rules", async () => {
  vi.mocked(previewEncounterDifficulty).mockResolvedValue(initial);
  const navigation = {
    record_key: "journals:S55aqwWIzpQRFhcq",
    embedded_path: [{ collection: "/pages", identity: { Stable: "WZgFEMDSP9TvYc4u" } }],
    field: null,
    passage: null,
    source_fingerprint: null,
  };
  const { onReference } = setup(undefined, {
    ...initial,
    rules: [{ title: "Encounter Budget", navigation }],
  });
  fireEvent.change(screen.getByRole("spinbutton", { name: "Party level" }), {
    target: { value: "6" },
  });
  const link = screen.getByRole("link", { name: "Encounter Budget" });
  expect(
    JSON.parse(
      new URL(link.getAttribute("href")!, "http://localhost").searchParams.get(
        "selection",
      )!,
    ),
  ).toEqual(navigation);
  fireEvent.click(link, { ctrlKey: true });
  expect(onReference).not.toHaveBeenCalled();
  fireEvent.click(link);
  expect(onReference).toHaveBeenCalledWith(
    navigation.record_key,
    expect.any(HTMLElement),
    navigation,
  );
  fireEvent.click(screen.getByRole("button", { name: /Encounter difficulty:/ }));
  await waitFor(() =>
    expect(screen.getByRole("spinbutton", { name: "Party level" })).toHaveValue("6"),
  );
});
