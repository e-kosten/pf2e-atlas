import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { flushSync } from "react-dom";
import { act, renderHook, waitFor } from "@testing-library/react";
import type { PropsWithChildren } from "react";
import { getEncounterTurnReview } from "../../api/atlasApi";
import { useEncounterTurnReview } from "./useEncounterTurnReview";
import type { useEncounterMutations } from "./useEncounterMutations";
vi.mock("../../api/atlasApi", () => ({
  getEncounterTurnReview: vi.fn(),
}));
const review: import("../../generated/atlas").EncounterTurnReviewView = {
  encounter_key: "encounter",
  expected_revision: 8n,
  current_participant_key: "incoming",
  next_participant_key: "next",
  reminders: [],
  outgoing: {
    participant_removed: false,
    damage_applied: false,
    recovered_episode_ids: [],
    boundary_key: "outgoing",
    participant_key: "actor",
    participant_name: "Actor",
    round_number: 1n,
    persistent_damage_groups: [],
    reminders: [
      {
        kind: "persistent_damage" as const,
        participant_key: "actor",
        participant_name: "Actor",
        condition_id: 1n,
        condition_name: "Persistent Damage",
        note: null,
      },
    ],
  },

  next_turn_unavailable_reason: null,
};
function setup(onAdvance: (revision: bigint) => Promise<boolean>) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const run = vi.fn().mockResolvedValue({
    active: true,
    refreshed: true,
    receipt: { encounter_key: "encounter" },
  });
  const mutations = {
    pending: false,
    run,
    retryRefresh: vi.fn(),
  } as unknown as ReturnType<typeof useEncounterMutations>;
  return {
    run,
    ...renderHook(
      () =>
        useEncounterTurnReview({
          encounterRef: "encounter",
          expectedRevision: 7n,
          mutations,
          onAdvance,
        }),
      {
        wrapper: ({ children }: PropsWithChildren) => (
          <QueryClientProvider client={client}>{children}</QueryClientProvider>
        ),
      },
    ),
  };
}
beforeEach(() => {
  vi.mocked(getEncounterTurnReview).mockResolvedValue(review);
});
it("advances once before opening context; dismissing never advances again", async () => {
  let complete!: (saved: boolean) => void;
  const advance = vi.fn(
    () =>
      new Promise<boolean>((resolve) => {
        complete = resolve;
      }),
  );
  const { result } = setup(advance);
  await waitFor(() => expect(result.current.loading).toBe(false));
  let pending!: Promise<void>;
  act(() => {
    pending = result.current.begin();
    void result.current.begin();
  });
  expect(advance).toHaveBeenCalledExactlyOnceWith(7n);
  expect(result.current.open).toBe(false);
  await act(async () => {
    vi.mocked(getEncounterTurnReview).mockResolvedValue({
      ...review,
      outgoing: { ...review.outgoing!, boundary_key: "new" },
    });
    complete(true);
    await pending;
  });
  expect(result.current.open).toBe(true);
  act(() => result.current.close());
  expect(result.current.open).toBe(false);
  expect(advance).toHaveBeenCalledTimes(1);
});
it("keeps a failed advance separate from reminder loading", async () => {
  const { result } = setup(vi.fn().mockResolvedValue(false));
  await act(async () => {
    await result.current.begin();
  });
  expect(result.current.open).toBe(false);
});
it("a reminder read failure after a saved advance does not undo progression", async () => {
  const advance = vi.fn().mockResolvedValue(true);
  const { result } = setup(advance);
  await waitFor(() => expect(result.current.loading).toBe(false));
  vi.mocked(getEncounterTurnReview).mockRejectedValue(
    new Error("Reminder read failed"),
  );
  await act(async () => {
    await result.current.begin();
  });
  expect(result.current.open).toBe(false);
  expect(result.current.error?.message).toBe("Reminder read failed");
  expect(advance).toHaveBeenCalledTimes(1);
});

it("advances without opening an empty reminder surface", async () => {
  vi.mocked(getEncounterTurnReview).mockResolvedValue({
    ...review,
    outgoing: null,
  });
  const advance = vi.fn().mockResolvedValue(true);
  const { result } = setup(advance);
  await act(async () => {
    await result.current.begin();
  });
  expect(advance).toHaveBeenCalledTimes(1);
  expect(result.current.open).toBe(false);
});

it("closing finishes the interaction without a mutation or reopening on refresh", async () => {
  const advance = vi.fn().mockResolvedValue(true);
  const { result, run } = setup(advance);
  await waitFor(() => expect(result.current.loading).toBe(false));
  vi.mocked(getEncounterTurnReview).mockResolvedValue({
    ...review,
    outgoing: { ...review.outgoing!, boundary_key: "new" },
  });
  await act(async () => {
    await result.current.begin();
  });
  expect(result.current.open).toBe(true);
  act(() => result.current.close());
  await act(async () => {
    await result.current.refresh();
  });
  expect(result.current.open).toBe(false);
  expect(result.current.boundaryKey).toBeUndefined();
  expect(run).not.toHaveBeenCalled();
  expect(advance).toHaveBeenCalledOnce();
});

it("retains reminders through a revision refresh only for the same encounter", async () => {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const mutations = { pending: false } as ReturnType<typeof useEncounterMutations>;
  const { result, rerender } = renderHook(
    ({ encounterRef, expectedRevision }) =>
      useEncounterTurnReview({
        encounterRef,
        expectedRevision,
        mutations,
        onAdvance: vi.fn(),
      }),
    {
      initialProps: { encounterRef: "encounter", expectedRevision: 7n },
      wrapper: ({ children }: PropsWithChildren) => (
        <QueryClientProvider client={client}>{children}</QueryClientProvider>
      ),
    },
  );
  await waitFor(() => expect(result.current.data).toEqual(review));
  vi.mocked(getEncounterTurnReview).mockImplementation(() => new Promise(() => {}));
  rerender({ encounterRef: "encounter", expectedRevision: 8n });
  expect(result.current.data).toEqual(review);
  rerender({ encounterRef: "different", expectedRevision: 8n });
  expect(result.current.data).toBeUndefined();
});

it("waits for the initial boundary snapshot before advancing and opens only the new reminder", async () => {
  let finish!: (value: typeof review) => void;
  vi.mocked(getEncounterTurnReview).mockImplementationOnce(
    () =>
      new Promise((resolve) => {
        finish = resolve;
      }),
  );
  const advance = vi.fn().mockResolvedValue(true);
  const { result } = setup(advance);
  let pending!: Promise<void>;
  act(() => {
    pending = result.current.begin();
  });
  expect(advance).not.toHaveBeenCalled();
  await act(async () => {
    finish({ ...review, outgoing: null });
    await pending;
  });
  expect(advance).toHaveBeenCalledOnce();
  expect(result.current.open).toBe(true);
  expect(result.current.boundaryKey).toBe("outgoing");
});

it("still advances once if the initial reminder snapshot fails without opening old reminders", async () => {
  vi.mocked(getEncounterTurnReview).mockRejectedValueOnce(new Error("Unavailable"));
  const advance = vi.fn().mockResolvedValue(true);
  const { result } = setup(advance);
  await act(async () => {
    await result.current.begin();
  });
  expect(advance).toHaveBeenCalledOnce();
  expect(result.current.open).toBe(false);
  expect(result.current.boundaryKey).toBeUndefined();
});

it.each([false, true])(
  "keeps an immutable captured snapshot across an observer revision change (failure=%s)",
  async (fails) => {
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    const mutations = { pending: false } as ReturnType<typeof useEncounterMutations>;
    vi.mocked(getEncounterTurnReview).mockResolvedValue({
      ...review,
      outgoing: null,
    });
    let observer!: (value: typeof review) => void;
    let captured!: (value: typeof review) => void;
    let rejectCaptured!: (error: Error) => void;
    const advance = vi.fn(async () => {
      vi.mocked(getEncounterTurnReview)
        .mockImplementationOnce(
          () =>
            new Promise((resolve) => {
              observer = resolve;
            }),
        )
        .mockImplementationOnce(
          () =>
            new Promise((resolve, reject) => {
              captured = resolve;
              rejectCaptured = reject;
            }),
        );
      flushSync(() => rerender({ revision: 8n }));
      return true;
    });
    const { result, rerender } = renderHook(
      ({ revision }) =>
        useEncounterTurnReview({
          encounterRef: "encounter",
          expectedRevision: revision,
          mutations,
          onAdvance: advance,
        }),
      {
        initialProps: { revision: 7n },
        wrapper: ({ children }: PropsWithChildren) => (
          <QueryClientProvider client={client}>{children}</QueryClientProvider>
        ),
      },
    );
    await waitFor(() => expect(result.current.data?.outgoing).toBeNull());
    let pending!: Promise<void>;
    act(() => {
      pending = result.current.begin();
    });
    await waitFor(() => expect(captured).toBeTypeOf("function"));
    await act(async () => {
      if (fails) rejectCaptured(new Error("Snapshot unavailable"));
      else captured(review);
      await pending;
    });
    expect(advance).toHaveBeenCalledOnce();
    if (fails) {
      expect(result.current.error?.message).toBe("Snapshot unavailable");
      expect(result.current.open).toBe(false);
    } else {
      expect(result.current.open).toBe(true);
      expect(result.current.boundaryKey).toBe("outgoing");
    }
    await act(async () => {
      observer(review);
    });
    await waitFor(() => expect(result.current.data).toEqual(review));
    expect(advance).toHaveBeenCalledOnce();
    if (fails) expect(result.current.error?.message).toBe("Snapshot unavailable");
    else expect(result.current.open).toBe(true);
  },
);

it("clears an expired reminder without inserting a success banner before the refreshed read", async () => {
  const { result, run } = setup(vi.fn().mockResolvedValue(true));
  await waitFor(() => expect(result.current.loading).toBe(false));
  let finish!: (value: typeof review) => void;
  vi.mocked(getEncounterTurnReview).mockImplementation(
    () =>
      new Promise((resolve) => {
        finish = resolve;
      }),
  );
  let removal!: ReturnType<typeof result.current.execute>;
  act(() => {
    removal = result.current.execute({
      kind: "remove-condition",
      participantKey: "actor",
      conditionId: 1n,
      expectedRevision: 8n,
    });
  });
  await waitFor(() => expect(finish).toBeTypeOf("function"));
  expect(run).toHaveBeenCalledOnce();
  expect(result.current.receipt).toBe("");
  await act(async () => {
    finish({ ...review, outgoing: { ...review.outgoing!, reminders: [] } });
    await removal;
  });
  expect(result.current.receipt).toBe("");
  await waitFor(() => expect(result.current.data?.outgoing?.reminders).toEqual([]));
});

it("retains single and grouped rolls until the next turn begins", async () => {
  const { result, run } = setup(vi.fn().mockResolvedValue(true));
  await waitFor(() => expect(result.current.loading).toBe(false));
  const first = {
    formula: "3",
    total: 3,
    expression: { kind: "number" as const, value: 3 },
  };
  const second = {
    formula: "8",
    total: 8,
    expression: { kind: "number" as const, value: 8 },
  };
  run.mockResolvedValueOnce({
    active: true,
    refreshed: true,
    receipt: { encounter_key: "encounter", roll: first },
  });
  run.mockResolvedValueOnce({
    active: true,
    refreshed: true,
    receipt: { encounter_key: "encounter", groups: [{ roll: null }, { roll: second }] },
  });
  // Both updates may complete before React renders. Neither may replace the other.
  await act(async () => {
    await result.current.execute({
      kind: "persistent-recovery",
      request: {
        participant_key: "actor",
        expected_encounter_revision: 7n,
        boundary_key: "outgoing",
        mode: "end_turn",
        episode_id: "fire",
        selected_source_id: 1n,
        outcome: { kind: "roll" },
        dc_override: null,
      },
    });
    await result.current.execute({
      kind: "persistent-damage",
      request: {
        participant_key: "actor",
        expected_encounter_revision: 8n,
        boundary_key: "outgoing",
        artifact_generation: null,
        groups: [
          { episode_id: "cold", selected_source_id: 2n, damage: { kind: "roll" } },
        ],
        context: {
          keep_current_initiative: null,
          keep_upright: null,
          natural_rest_healing: null,
          attack_or_spell_damage: null,
          lethality: null,
          critical: null,
          enemy_source: null,
          responsible_participant_key: null,
          base_death_threshold: null,
        },
      },
    });
  });
  expect(result.current.outcomes).toEqual([first, second]);
  await act(async () => {
    await result.current.begin();
  });
  expect(result.current.outcomes).toEqual([]);
});
