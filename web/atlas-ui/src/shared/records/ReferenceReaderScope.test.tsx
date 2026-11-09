import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { App, ConfigProvider } from "antd";
import { getDefinitionDetail, getRecordDetail } from "../../api/atlasApi";
import type {
  DefinitionDetailView,
  RecordDetailView,
  RecordNavigationView,
  ReferenceHelpView,
} from "../../generated/atlas";
import { detailFixture } from "../../test/fixtures";
import { RouterTestContext } from "../../test/router";
import { selectedNavigation, parseSearchParams } from "../../app/urlState";
import { ReferenceHelpHost } from "./ReferenceHelpHost";
import { ReferenceReader } from "./ReferenceReader";
import { RecordView } from "../../features/records/RecordViews";
import { rootNavigation, useReferenceReading } from "./useReferenceReading";

vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  getRecordDetail: vi.fn(),
  getDefinitionDetail: vi.fn(),
}));
const traitHelp: ReferenceHelpView = {
  definition_key: "trait:agile",

  destination: { kind: "definition", key: "trait:agile" },
};
const trait: DefinitionDetailView = {
  artifact_generation: "test-generation",
  definition_key: "trait:agile",
  label: "Agile",
  help: traitHelp,
  content: { body: { kind: "plain", text: "Agile rules." }, references: [] },

  notices: [],

  related_filters: [],
  publications: [],
};
function child(key: string): RecordNavigationView {
  return {
    ...rootNavigation(key),
    embedded_path: [{ collection: "items", identity: { Stable: "authored-child" } }],
    field: "system.description.value",
    source_fingerprint: "verified-child",
    spell_rank: 5,
  };
}
function childDetail(key: string): RecordDetailView {
  const result = detailFixture(key, key === "actors:ghoul" ? "Ghoul" : "Dragon");
  result.selected = child(key);
  result.presentation.identity = {
    ...result.record,
    title: key === "actors:ghoul" ? "Claw" : "Bite",
    traits: [
      {
        kind: "trait",
        value: "agile",
        label: "Agile",
        trait_key: "trait:agile",
        help: traitHelp,
      },
    ],
  };
  return result;
}

it("keeps selected-reader child references, owner suppression and source actions local beneath the real app host", async () => {
  history.replaceState(null, "", "/search");
  const parent = detailFixture("actors:ghoul", "Ghoul", "actors:ghoul");
  parent.relationships[0].destination = {
    kind: "resolved_embedded",
    target: child("actors:ghoul"),
  };
  parent.relationships.push({
    ...parent.relationships[0],
    ordinal: 1,
    destination: { kind: "resolved_embedded", target: child("actors:dragon") },
  });
  parent.presentation.content[0].body = {
    kind: "html",
    controls: [],
    inclusions: [],
    html: '<p><a data-atlas-reference="0" href="#">Own child</a> <a data-atlas-reference="1" href="#">Other child</a></p>',
  };
  vi.mocked(getRecordDetail).mockImplementation(async (key, request) =>
    request?.embedded_path?.length
      ? childDetail(key)
      : key === "actors:ghoul"
        ? parent
        : detailFixture(key, "Dragon"),
  );
  vi.mocked(getDefinitionDetail).mockResolvedValue(trait);
  const actions = vi.fn((record) => <button>Reader action {record.title}</button>);
  function SelectedReader() {
    const reading = useReferenceReading({ navigation: rootNavigation("actors:ghoul") });
    return (
      <ReferenceReader
        reading={reading}
        selectedRecordKey="actors:ghoul"
        recordActions={actions}
        showRelated={false}
      />
    );
  }
  render(
    <RouterTestContext>
      <QueryClientProvider
        client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
      >
        <ConfigProvider theme={{ token: { motion: false } }}>
          <ReferenceHelpHost recordActions={() => <button>Global action</button>}>
            <SelectedReader />
          </ReferenceHelpHost>
        </ConfigProvider>
      </QueryClientProvider>
    </RouterTestContext>,
  );
  const own = await screen.findByRole("link", { name: "Own child" });
  own.focus();
  fireEvent.click(own);
  let drawer = await screen.findByRole("dialog", { name: "Claw" });
  expect(screen.getAllByRole("dialog")).toHaveLength(1);
  expect(within(drawer).queryByRole("link", { name: "Return to Ghoul" })).toBeNull();
  expect(
    within(drawer).getByRole("button", { name: "Reader action Ghoul" }),
  ).toBeVisible();
  expect(within(drawer).queryByRole("button", { name: "Global action" })).toBeNull();
  const fullPage = within(drawer).getByRole("link", {
    name: "Open reference full page",
  });
  expect(
    selectedNavigation(
      parseSearchParams(
        new URL(fullPage.getAttribute("href")!, "http://localhost").search,
      ),
      "actors:ghoul",
    ),
  ).toEqual(child("actors:ghoul"));
  fireEvent.click(within(drawer).getByRole("link", { name: "Agile" }));
  drawer = await screen.findByRole("dialog", { name: "Agile" });
  expect(screen.getAllByRole("dialog")).toHaveLength(1);
  fireEvent.click(within(drawer).getByRole("button", { name: "Back" }));
  drawer = await screen.findByRole("dialog", { name: "Claw" });
  expect(
    within(drawer).getByRole("button", { name: "Reader action Ghoul" }),
  ).toBeVisible();
  fireEvent.click(
    within(drawer).getByRole("button", { name: "Close reference preview" }),
  );
  await waitFor(() => expect(own).toHaveFocus());
  expect(screen.getByRole("heading", { name: "Ghoul" })).toBeVisible();
  fireEvent.click(screen.getByRole("link", { name: "Other child" }));
  drawer = await screen.findByRole("dialog", { name: "Bite" });
  expect(within(drawer).getByRole("link", { name: "Return to Dragon" })).toBeVisible();
  expect(
    within(drawer).getByRole("button", { name: "Reader action Dragon" }),
  ).toBeVisible();
  expect(within(drawer).queryByRole("button", { name: "Global action" })).toBeNull();
  expect(screen.getAllByRole("dialog")).toHaveLength(1);
  expect(screen.getByRole("heading", { name: "Ghoul" })).toBeVisible();
});

it("preserves the standalone visible root owner when its scoped Host is nested under the app fallback", async () => {
  history.replaceState(null, "", "/records/actors:ghoul");
  const parent = detailFixture("actors:ghoul", "Ghoul", "actors:ghoul");
  parent.relationships[0].destination = {
    kind: "resolved_embedded",
    target: child("actors:ghoul"),
  };
  vi.mocked(getRecordDetail).mockImplementation(async (key, request) =>
    request?.embedded_path?.length ? childDetail(key) : parent,
  );
  render(
    <RouterTestContext>
      <QueryClientProvider
        client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
      >
        <ConfigProvider theme={{ token: { motion: false } }}>
          <App>
            <ReferenceHelpHost recordActions={() => <button>Global action</button>}>
              <RecordView route={{ recordKey: "actors:ghoul" }} />
            </ReferenceHelpHost>
          </App>
        </ConfigProvider>
      </QueryClientProvider>
    </RouterTestContext>,
  );
  const opener = await screen.findByRole("link", { name: "Nested Rule" });
  opener.focus();
  fireEvent.click(opener);
  const drawer = await screen.findByRole("dialog", { name: "Claw" });
  expect(within(drawer).queryByRole("link", { name: "Return to Ghoul" })).toBeNull();
  expect(within(drawer).getByRole("button", { name: "Add to list" })).toBeVisible();
  expect(within(drawer).queryByRole("button", { name: "Global action" })).toBeNull();
  expect(screen.getAllByRole("dialog")).toHaveLength(1);
  fireEvent.click(
    within(drawer).getByRole("button", { name: "Close reference preview" }),
  );
  await waitFor(() => expect(opener).toHaveFocus());
  expect(screen.getByRole("heading", { name: "Ghoul" })).toBeVisible();
});
