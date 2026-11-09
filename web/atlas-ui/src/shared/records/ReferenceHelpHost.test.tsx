import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { ConfigProvider } from "antd";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { atlasRouter } from "../../app/router";
import { getRecordDetail, getDefinitionDetail } from "../../api/atlasApi";
import { detailFixture } from "../../test/fixtures";
import { RouterTestContext } from "../../test/router";
import { ReferenceHelpHost } from "./ReferenceHelpHost";
import { ReferenceHelpLink } from "./ReferenceHelp";
import { TraitBadge } from "./TraitBadge";
import type { DefinitionDetailView } from "../../generated/atlas";
vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  getDefinitionDetail: vi.fn(),
  getRecordDetail: vi.fn(),
}));
const target = detailFixture("rules:condition", "Condition").selected;
const help = {
  definition_key: "trait:agile",

  destination: { kind: "definition" as const, key: "trait:agile" },
};
const definition: DefinitionDetailView = {
  definition_key: "trait:agile",
  label: "Agile",
  artifact_generation: "test-generation",
  help,

  notices: [],

  content: {
    body: {
      kind: "html",
      inclusions: [],
      controls: [],
      html: '<p>A trait definition with a <span data-atlas-reference="0">Condition reference</span>.</p>',
    },
    references: [{ ordinal: 0, destination: { kind: "resolved_record", target } }],
  },
  related_filters: [],
  publications: [],
};
function setup() {
  history.replaceState(null, "", "/search");
  vi.mocked(getDefinitionDetail).mockResolvedValue(definition);
  vi.mocked(getRecordDetail).mockResolvedValue(
    detailFixture("rules:condition", "Condition"),
  );
  return render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <QueryClientProvider
        client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
      >
        <RouterTestContext>
          <ReferenceHelpHost>
            <TraitBadge
              trait={{
                kind: "trait",
                value: "agile",
                label: "Agile",
                trait_key: "trait:agile",
                help,
              }}
            />
          </ReferenceHelpHost>
        </RouterTestContext>
      </QueryClientProvider>
    </ConfigProvider>,
  );
}
it("opens trait definitions and nested records in one drawer, restores Back and close focus without changing the host URL", async () => {
  setup();
  const badge = screen.getByRole("link", { name: "Agile" });
  act(() => badge.focus());
  fireEvent.click(badge);
  let drawer = await screen.findByRole("dialog", { name: "Agile" });
  const content = await within(drawer).findByRole("link", {
    name: "Condition reference",
  });
  const scroll = content.closest<HTMLElement>(".record-preview-drawer__body")!;
  scroll.scrollTop = 70;
  expect(
    within(drawer).getByRole("link", { name: "Open reference full page" }),
  ).toHaveAttribute("href", "/traits/trait:agile");
  fireEvent.click(content);
  drawer = await screen.findByRole("dialog", { name: "Condition" });
  expect(screen.getAllByRole("dialog")).toHaveLength(1);
  fireEvent.click(within(drawer).getByRole("button", { name: "Back" }));
  drawer = await screen.findByRole("dialog", { name: "Agile" });
  expect(scroll.scrollTop).toBe(70);
  expect(
    within(drawer).getByRole("link", { name: "Condition reference" }),
  ).toHaveFocus();
  expect(location.pathname).toBe("/search");
  fireEvent.click(
    within(drawer).getByRole("button", { name: "Close reference preview" }),
  );
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  expect(badge).toHaveFocus();
});
it("keeps modified trait activation native", () => {
  setup();
  const badge = screen.getByRole("link", { name: "Agile" });
  const event = new MouseEvent("click", {
    bubbles: true,
    cancelable: true,
    ctrlKey: true,
  });
  act(() => {
    badge.dispatchEvent(event);
  });
  expect(event.defaultPrevented).toBe(false);
  expect(screen.queryByRole("dialog")).toBeNull();
});

it("closes a global reference on page departure while retaining it through profile-only navigation", async () => {
  setup();
  fireEvent.click(screen.getByRole("link", { name: "Agile" }));
  await screen.findByRole("dialog", { name: "Agile" });
  const fromLocation = atlasRouter.latestLocation;
  const navigate = (pathname: string) =>
    atlasRouter.emit({
      type: "onBeforeNavigate",
      fromLocation,
      toLocation: { ...fromLocation, pathname },
      pathChanged: pathname !== fromLocation.pathname,
      hrefChanged: true,
      hashChanged: false,
    });
  act(() => navigate(fromLocation.pathname));
  expect(screen.getByRole("dialog", { name: "Agile" })).toBeInTheDocument();
  act(() => navigate("/lists"));
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
});

it("honors explicit nested host actions while generation-only descendants retain the same scoped session", async () => {
  history.replaceState(null, "", "/search");
  vi.mocked(getRecordDetail).mockResolvedValue(
    detailFixture("rules:condition", "Condition"),
  );
  render(
    <RouterTestContext>
      <QueryClientProvider
        client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
      >
        <ConfigProvider theme={{ token: { motion: false } }}>
          <ReferenceHelpHost recordActions={() => <button>Global action</button>}>
            <ReferenceHelpHost
              generation="test-generation"
              recordActions={(record) => <button>Scoped action {record.title}</button>}
            >
              <ReferenceHelpHost generation="test-generation">
                <ReferenceHelpLink
                  help={{
                    definition_key: null,

                    destination: { kind: "record", navigation: target },
                  }}
                >
                  Open scoped record
                </ReferenceHelpLink>
              </ReferenceHelpHost>
            </ReferenceHelpHost>
          </ReferenceHelpHost>
        </ConfigProvider>
      </QueryClientProvider>
    </RouterTestContext>,
  );
  fireEvent.click(screen.getByRole("link", { name: "Open scoped record" }));
  const drawer = await screen.findByRole("dialog", { name: "Condition" });
  expect(screen.getAllByRole("dialog")).toHaveLength(1);
  expect(
    within(drawer).getByRole("button", { name: "Scoped action Condition" }),
  ).toBeVisible();
  expect(within(drawer).queryByRole("button", { name: "Global action" })).toBeNull();
});
