import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { EncounterActorReading } from "./EncounterActorReading";
import { AtlasApiError } from "../../api/atlasApi";
import { actorFixture, detailFixture, numberFact } from "../../test/fixtures";
import type {
  EmbeddedReadingBatchView,
  RecordPresentationView,
} from "../../generated/atlas";
import type { EmbeddedReadingContext } from "../../shared/records/EmbeddedAbilityReading";
const api = vi.hoisted(() => ({ getEmbeddedReading: vi.fn() }));
vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  ...api,
}));
const reading: EmbeddedReadingContext = {
  generation: "generation-a",
  parent: {
    record_key: "hazards:pillar",
    embedded_path: [],
    field: null,
    passage: null,
    source_fingerprint: null,
  },
};
function batch(context = reading): EmbeddedReadingBatchView {
  return {
    artifact_generation: context.generation,
    parent: context.parent,
    availability: "value",
    total: 1,
    start: 0,
    end: 1,
    previous: null,
    next: null,
    entries: [
      {
        navigation: context.parent,
        relationships: [],
        content: [
          {
            locator: {
              record: context.parent.record_key,
              embedded_path: [],
              field: "/system/details/routine",
            },
            role: "routine",
            source_fingerprint: null,
            body: { kind: "plain", text: "Three authored strikes" },
          },
        ],
      },
    ],
  };
}
function presentation(): RecordPresentationView {
  return {
    ...detailFixture(reading.parent.record_key, "Pillar").presentation,
    body: {
      kind: "hazard",
      value: {
        actor: actorFixture(),
        complex: { state: "value", value: true },
        stealth: numberFact(12),
        hardness: numberFact(5),
        defense_block: { state: "value", value: true },
        show_saves: true,
      },
    },
  };
}
function setup() {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const renderReading = (context = reading, value = presentation()) => (
    <QueryClientProvider client={client}>
      <EncounterActorReading
        presentation={value}
        reading={context}
        actorEncounter={{
          participantKey: "pillar",
          durability: <input aria-label="Current HP" defaultValue="55" />,
          conditions: null,
        }}
        encounter
        hideIdentity
        onReference={vi.fn()}
      />
    </QueryClientProvider>
  );
  return { client, renderReading };
}
beforeEach(() => {
  api.getEmbeddedReading.mockReset();
});
it("adds authored rules without replacing runtime controls or fetching again on value edits", async () => {
  let resolve!: (value: EmbeddedReadingBatchView) => void;
  api.getEmbeddedReading.mockImplementationOnce(
    () =>
      new Promise((r) => {
        resolve = r;
      }),
  );
  const { renderReading } = setup();
  const { rerender } = render(renderReading());
  const hp = screen.getByRole("textbox", { name: "Current HP" });
  hp.focus();
  fireEvent.change(hp, { target: { value: "51" } });
  await act(async () => resolve(batch()));
  expect(await screen.findByText("Three authored strikes")).toBeVisible();
  expect(screen.getByRole("textbox", { name: "Current HP" })).toBe(hp);
  expect(hp).toHaveFocus();
  expect(hp).toHaveValue("51");
  const updated = presentation();
  if (updated.body.kind === "hazard")
    updated.body.value.actor.armor_class = numberFact(42);
  rerender(renderReading(reading, updated));
  expect(screen.getByRole("textbox", { name: "Current HP" })).toBe(hp);
  expect(api.getEmbeddedReading).toHaveBeenCalledTimes(1);
  expect(api.getEmbeddedReading.mock.calls[0][0].selection).toEqual({
    kind: "actor_content",
  });
});
it("keeps failures local and retries; retains same-generation text after a failed refresh", async () => {
  api.getEmbeddedReading.mockRejectedValueOnce(new Error("offline"));
  const { client, renderReading } = setup();
  render(renderReading());
  expect(await screen.findByText("Rules text unavailable.")).toBeVisible();
  expect(screen.getByRole("textbox", { name: "Current HP" })).toBeEnabled();
  api.getEmbeddedReading.mockResolvedValueOnce(batch());
  fireEvent.click(screen.getByRole("button", { name: "Retry" }));
  expect(await screen.findByText("Three authored strikes")).toBeVisible();
  api.getEmbeddedReading.mockRejectedValueOnce(new Error("reconnecting"));
  await act(async () => {
    await client.refetchQueries({ queryKey: ["embedded-reading"] });
  });
  expect(
    await screen.findByText("Could not update rules text. Previous text shown."),
  ).toBeVisible();
  expect(screen.getByText("Three authored strikes")).toBeVisible();
});
it("drops stale-generation content and offers the parent reload", async () => {
  api.getEmbeddedReading.mockResolvedValueOnce(batch());
  const { client, renderReading } = setup();
  const reloadParent = vi.fn();
  render(renderReading({ ...reading, reloadParent }));
  expect(await screen.findByText("Three authored strikes")).toBeVisible();
  api.getEmbeddedReading.mockRejectedValue(
    new AtlasApiError(409, "Changed", {
      code: "artifact_generation_changed",
      message: "Changed",
    }),
  );
  await act(async () => {
    await client.refetchQueries({ queryKey: ["embedded-reading"] });
  });
  await waitFor(() =>
    expect(screen.queryByText("Three authored strikes")).not.toBeInTheDocument(),
  );
  fireEvent.click(screen.getByRole("button", { name: "Reload encounter" }));
  expect(reloadParent).toHaveBeenCalledOnce();
  expect(screen.getByRole("textbox", { name: "Current HP" })).toBeEnabled();
});
it("does not lend text across different actor identities or generations", async () => {
  api.getEmbeddedReading
    .mockResolvedValueOnce(batch())
    .mockImplementation(() => new Promise(() => {}));
  const { renderReading } = setup();
  const { rerender } = render(renderReading());
  expect(await screen.findByText("Three authored strikes")).toBeVisible();
  rerender(
    renderReading({
      ...reading,
      parent: { ...reading.parent, record_key: "hazards:other" },
    }),
  );
  expect(screen.queryByText("Three authored strikes")).not.toBeInTheDocument();
  rerender(renderReading({ ...reading, generation: "generation-b" }));
  expect(screen.queryByText("Three authored strikes")).not.toBeInTheDocument();
});

it("drops explicitly retired root text while its replacement is pending", async () => {
  api.getEmbeddedReading
    .mockResolvedValueOnce(batch())
    .mockImplementation(() => new Promise(() => {}));
  const { client, renderReading } = setup();
  render(renderReading());
  expect(await screen.findByText("Three authored strikes")).toBeVisible();
  await act(async () => {
    client.removeQueries({ queryKey: ["embedded-reading"] });
  });
  await waitFor(() =>
    expect(screen.queryByText("Three authored strikes")).not.toBeInTheDocument(),
  );
  expect(screen.getByRole("textbox", { name: "Current HP" })).toBeEnabled();
});
