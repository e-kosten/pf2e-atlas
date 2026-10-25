import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ConfigProvider, Form, InputNumber, Select } from "antd";
import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { useState } from "react";
import type {
  EncounterDifficultyView,
  PreviewEncounterDifficultyRequest,
} from "../../generated/atlas";
import {
  EncounterAddDifficulty,
  type EncounterRecordAddForm,
} from "./EncounterAddDifficulty";
const current: EncounterDifficultyView = {
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
  xp_per_pc: 80,
  award_note: null,
  budgets: [
    { label: "Trivial", xp: 40 },
    { label: "Low", xp: 60 },
    { label: "Moderate", xp: 80 },
    { label: "Severe", xp: 120 },
    { label: "Extreme", xp: 160 },
  ],
  opposition: [],
};
function setup(
  onPreview: (
    request: PreviewEncounterDifficultyRequest,
  ) => Promise<EncounterDifficultyView>,
) {
  function Harness() {
    const [form] = Form.useForm<EncounterRecordAddForm>();
    const [selection, setSelection] = useState<string | null>(null);
    const [revision, setRevision] = useState(1n);
    return (
      <Form form={form}>
        <button type="button" onClick={() => setSelection("actors:ogre")}>
          Choose ogre
        </button>
        <button type="button" onClick={() => setSelection("actors:trap")}>
          Choose trap
        </button>
        <button type="button" onClick={() => setRevision(2n)}>
          Refresh encounter
        </button>
        <Form.Item name="quantity" initialValue={1}>
          <InputNumber aria-label="Quantity" />
        </Form.Item>
        <Form.Item name="adjustment">
          <Select
            aria-label="Adjustment"
            options={[{ value: "elite", label: "Elite" }]}
          />
        </Form.Item>
        <EncounterAddDifficulty
          encounterKey="encounter"
          revision={revision}
          value={current}
          onPreview={onPreview}
          form={form}
          selectedRecordKey={selection}
          recordName={selection === "actors:trap" ? "Trap" : "Ogre"}
          creature={selection === "actors:ogre"}
          disabled={false}
        />
      </Form>
    );
  }
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <QueryClientProvider
        client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
      >
        <Harness />
      </QueryClientProvider>
    </ConfigProvider>,
  );
  return screen.getByRole("region", { name: "Encounter difficulty preview" });
}
it("shows current difficulty, then previews quantity and adjustment without an editor", async () => {
  const onPreview = vi
    .fn()
    .mockResolvedValue({ ...current, total_xp: 120, rating: "Severe" });
  const bar = setup(onPreview);
  expect(within(bar).getByText("Current encounter")).toBeVisible();
  expect(within(bar).getByText("Moderate · 80 XP")).toBeVisible();
  expect(onPreview).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button", { name: "Choose ogre" }));
  await waitFor(() =>
    expect(within(bar).getByText("After adding 1 × Ogre")).toBeVisible(),
  );
  fireEvent.change(screen.getByRole("spinbutton", { name: "Quantity" }), {
    target: { value: "2" },
  });
  fireEvent.mouseDown(screen.getByRole("combobox", { name: "Adjustment" }));
  fireEvent.click(screen.getByText("Elite", { exact: true }));
  await waitFor(() =>
    expect(onPreview).toHaveBeenLastCalledWith({
      expected_revision: 1n,
      reference: current.reference,
      addition: {
        record_ref: "actors:ogre",
        quantity: 2,
        participant_variant: "elite",
      },
    }),
  );
  await waitFor(() =>
    expect(within(bar).getByText("After adding 2 × Ogre")).toBeVisible(),
  );
  expect(within(bar).queryByRole("spinbutton")).toBeNull();
  expect(within(bar).queryByRole("button")).toBeNull();
});
it("retains a named previous scenario across selections and failures, but not revisions", async () => {
  const onPreview = vi
    .fn()
    .mockResolvedValue({ ...current, total_xp: 120, rating: "Severe" });
  const bar = setup(onPreview);
  fireEvent.click(screen.getByRole("button", { name: "Choose ogre" }));
  await waitFor(() => expect(within(bar).getByText("Severe · 120 XP")).toBeVisible());
  onPreview.mockRejectedValue(new Error("Disconnected"));
  fireEvent.change(screen.getByRole("spinbutton", { name: "Quantity" }), {
    target: { value: "2" },
  });
  await waitFor(() =>
    expect(within(bar).getByText("Update failed · previous: 1 × Ogre")).toBeVisible(),
  );
  expect(within(bar).getByText("Severe · 120 XP")).toBeVisible();
  onPreview.mockResolvedValue({ ...current, total_xp: 160, rating: "Extreme" });
  fireEvent.click(within(bar).getByRole("button", { name: "Retry" }));
  await waitFor(() =>
    expect(within(bar).getByText("After adding 2 × Ogre")).toBeVisible(),
  );
  expect(within(bar).getByText("Extreme · 160 XP")).toBeVisible();
  onPreview.mockImplementation(() => new Promise(() => {}));
  fireEvent.click(screen.getByRole("button", { name: "Choose trap" }));
  expect(within(bar).getByText("Previous: 2 × Ogre")).toBeVisible();
  expect(within(bar).getByText("Extreme · 160 XP")).toBeVisible();
  expect(within(bar).queryByText("Moderate · 80 XP")).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Refresh encounter" }));
  expect(within(bar).getByText("Moderate · 80 XP")).toBeVisible();
});

it("starts a new selection immediately and ignores its late superseded response", async () => {
  let finishOgre!: (value: EncounterDifficultyView) => void;
  let finishTrap!: (value: EncounterDifficultyView) => void;
  const onPreview = vi.fn(
    (request: PreviewEncounterDifficultyRequest) =>
      new Promise<EncounterDifficultyView>((resolve) => {
        if (request.addition?.record_ref === "actors:ogre") finishOgre = resolve;
        else finishTrap = resolve;
      }),
  );
  const bar = setup(onPreview);
  await act(async () =>
    fireEvent.click(screen.getByRole("button", { name: "Choose ogre" })),
  );
  expect(onPreview).toHaveBeenCalledOnce();
  await act(async () =>
    fireEvent.click(screen.getByRole("button", { name: "Choose trap" })),
  );
  expect(onPreview).toHaveBeenCalledTimes(2);
  await act(async () => finishTrap({ ...current, total_xp: 160, rating: "Extreme" }));
  await waitFor(() =>
    expect(within(bar).getByText("After adding 1 × Trap")).toBeVisible(),
  );
  expect(within(bar).getByText("Extreme · 160 XP")).toBeVisible();
  await act(async () => finishOgre({ ...current, total_xp: 120, rating: "Severe" }));
  await waitFor(() =>
    expect(within(bar).getByText("After adding 1 × Trap")).toBeVisible(),
  );
  expect(within(bar).getByText("Extreme · 160 XP")).toBeVisible();
  expect(within(bar).queryByText("Severe · 120 XP")).toBeNull();
});

it("shows a completed unavailable result instead of masking it with a retained total", async () => {
  const onPreview = vi
    .fn()
    .mockResolvedValue({ ...current, total_xp: 120, rating: "Severe" });
  const bar = setup(onPreview);
  fireEvent.click(screen.getByRole("button", { name: "Choose ogre" }));
  await within(bar).findByText("Severe · 120 XP");
  onPreview.mockResolvedValue({
    ...current,
    total_xp: null,
    rating: null,
    opposition: [
      {
        participant_key: null,
        name: "Trap",
        kind: "Hazard",
        level: 5,
        xp: null,
        unavailable_reason: "Hazard complexity is unavailable.",
      },
    ],
  });
  fireEvent.click(screen.getByRole("button", { name: "Choose trap" }));
  await within(bar).findByText("After adding 1 × Trap");
  expect(within(bar).getByText("Difficulty incomplete")).toBeVisible();
  expect(within(bar).queryByText("Severe · 120 XP")).toBeNull();
  expect(within(bar).queryByRole("progressbar")).toBeNull();
});
