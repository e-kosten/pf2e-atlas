import { useContext } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryHistory } from "@tanstack/react-router";
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { createAtlasRouter } from "../../app/router";
import type { ProgressionRequest, ProgressionView } from "../../generated/atlas";
import { RouterTestContext } from "../../test/router";
import { detailFixture } from "../../test/fixtures";
import { ContentFields } from "./RecordPresentation";
import { RecordProgression } from "./RecordProgression";
import { RecordReadingWorkspace } from "./RecordReadingWorkspace";
import { OpenRecordProgression } from "./recordExploreLocation";
import { rootNavigation } from "./useReferenceReading";

const getProgression = vi.hoisted(() => vi.fn());
vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  getProgression,
}));

const speaker = rootNavigation("feats:SpeakerDefense01");
const heritage = rootNavigation("heritages:BuddingSpeaker01");
const training = rootNavigation("feats:SpeakerTraining1");
const ancestryFeat = rootNavigation("feats:ElvenInstincts01");
const aiuvarin = rootNavigation("heritages:AiuvarinHeritage");
const authoredField = "/system/prerequisites/value/0/value";
const requirement = [
  { kind: "any_of", path: "/0", depth: 0, label: "Any of" },
  {
    kind: "record",
    path: "/0/0",
    depth: 1,
    label: "Budding Speaker",
    navigation: heritage,
  },
  {
    kind: "record",
    path: "/0/1",
    depth: 1,
    label: "Speaker in Training",
    navigation: training,
  },
] satisfies ProgressionView["prerequisites"];

function response(request: ProgressionRequest, fields: Partial<ProgressionView> = {}) {
  return {
    selection: rootNavigation(request.selection.record_key),
    artifact_generation: request.artifact_generation,
    prerequisites: [],
    required_by: { entries: [] },
    ancestry_feat_pools: [],
    pool: { entries: [] },
    available_through: { entries: [] },
    ...fields,
  } satisfies ProgressionView;
}

function ReadAction() {
  const openProgression = useContext(OpenRecordProgression);
  return <button onClick={openProgression}>Progression</button>;
}

function setup(key: string, onReload?: () => void) {
  const router = createAtlasRouter(
    createMemoryHistory({ initialEntries: ["/records/" + key] }),
  );
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return {
    router,
    ...render(
      <QueryClientProvider client={client}>
        <RouterTestContext router={router}>
          <RecordReadingWorkspace
            explore={
              <RecordProgression
                detail={{ ...detailFixture(key), progression: { inline_links: [] } }}
                onReload={onReload}
              />
            }
          >
            <ReadAction />
          </RecordReadingWorkspace>
        </RouterTestContext>
      </QueryClientProvider>,
    ),
  };
}

beforeEach(() => {
  getProgression.mockReset();
});

it("keeps original prerequisite text and links only the resolved occurrence", () => {
  const onReference = vi.fn();
  render(
    <ContentFields
      fields={[
        {
          locator: {
            record: speaker.record_key,
            field: authoredField,
            embedded_path: [],
          },
          role: "prerequisite",
          source_fingerprint: null,
          body: {
            kind: "html",
            html: "<strong>Budding Speaker</strong> heritage or Speaker in Training",
            controls: [],
            inclusions: [],
          },
        },
      ]}
      progressionLinks={[
        { field: authoredField, text: "Budding Speaker", navigation: heritage },
      ]}
      onReference={onReference}
    />,
  );
  expect(screen.getByText("Prerequisites").parentElement).toHaveTextContent(
    "Budding Speaker heritage or Speaker in Training",
  );
  const link = screen.getByRole("link", { name: "Budding Speaker" });
  expect(link).toHaveAttribute("href", expect.stringContaining(heritage.record_key));
  expect(
    screen.queryByRole("link", { name: "Speaker in Training" }),
  ).not.toBeInTheDocument();
  fireEvent.click(link);
  expect(onReference).toHaveBeenCalledWith(heritage.record_key, link, heritage);
});

it("opens a reverse alternative in place and preserves the progression history", async () => {
  getProgression.mockImplementation(async (request: ProgressionRequest) =>
    request.selection.record_key === heritage.record_key
      ? response(request, {
          required_by: {
            entries: [
              {
                title: "Speaker’s Defense",
                navigation: speaker,
                level: 5,
                requirements: requirement,
                witnesses: [{ path: "/0/0", alternative: true }],
              },
            ],
          },
        })
      : response(request, { prerequisites: requirement }),
  );
  const { router } = setup(heritage.record_key);
  fireEvent.click(screen.getByRole("button", { name: "Progression" }));
  expect(await screen.findByText("Required by")).toBeVisible();
  expect(screen.getByText("Alternative prerequisite")).toBeVisible();
  expect(screen.getByText("Level 5")).toBeVisible();
  expect(router.latestLocation.search.record_tab).toBe("progression");
  fireEvent.click(screen.getByRole("button", { name: "Expand Speaker’s Defense" }));
  expect(await screen.findByText("Any of")).toBeVisible();
  expect(screen.getByRole("link", { name: "Speaker in Training" })).toBeVisible();
  expect(
    screen.queryByRole("button", { name: "Expand Budding Speaker" }),
  ).not.toBeInTheDocument();
  expect(screen.getByRole("link", { name: "Speaker’s Defense" })).toBeVisible();
  await act(async () => router.history.back());
  expect(screen.getByRole("button", { name: "Progression" })).toBeVisible();
  await act(async () => router.history.forward());
  expect(await screen.findByText("Required by")).toBeVisible();
  expect(await screen.findByText("Any of")).toBeVisible();
});

it("loads a pool on demand and continues it with the returned offset", async () => {
  getProgression.mockImplementation(async (request: ProgressionRequest) => {
    if (request.direction !== "pool") {
      return response(request, {
        ancestry_feat_pools: [{ label: "Elf", trait_key: "elf" }],
      });
    }
    const entry = request.offset ? training : ancestryFeat;
    return response(request, {
      pool: {
        entries: [
          {
            navigation: entry,
            level: request.offset ? 5 : 1,
            title: request.offset ? "Speaker in Training" : "Elven Instincts",
            requirements: [],
            witnesses: [],
          },
        ],
        next_offset: request.offset ? undefined : 1,
      },
    });
  });
  setup(aiuvarin.record_key);
  fireEvent.click(screen.getByRole("button", { name: "Progression" }));
  const summary = await screen.findByText("Elf");
  expect(getProgression).toHaveBeenCalledTimes(1);
  fireEvent.click(summary);
  expect(await screen.findByRole("link", { name: "Elven Instincts" })).toBeVisible();
  fireEvent.click(screen.getByRole("button", { name: "Show more" }));
  await waitFor(() =>
    expect(screen.getByRole("link", { name: "Speaker in Training" })).toBeVisible(),
  );
  expect(screen.getByRole("link", { name: "Elven Instincts" })).toBeVisible();
  expect(getProgression).toHaveBeenLastCalledWith(
    expect.objectContaining({ direction: "pool", trait_key: "elf", offset: 1 }),
    expect.any(AbortSignal),
  );
});

it("offers record reload when the progression response is from another generation", async () => {
  const reload = vi.fn();
  getProgression.mockImplementation(async (request: ProgressionRequest) =>
    response(request, { artifact_generation: "changed", prerequisites: requirement }),
  );
  setup(speaker.record_key, reload);
  fireEvent.click(screen.getByRole("button", { name: "Progression" }));
  fireEvent.click(await screen.findByRole("button", { name: "Reload record" }));
  expect(reload).toHaveBeenCalledOnce();
  expect(screen.queryByText("Any of")).not.toBeInTheDocument();
});
