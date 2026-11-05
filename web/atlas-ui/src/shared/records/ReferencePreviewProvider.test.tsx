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
import { getRecordDetail, getTraitDetail } from "../../api/atlasApi";
import { detailFixture } from "../../test/fixtures";
import { RouterTestContext } from "../../test/router";
import { ReferencePreviewProvider } from "./ReferencePreviewProvider";
import { TraitBadge } from "./TraitBadge";
import type { TraitDetailView } from "../../generated/atlas";
vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  getTraitDetail: vi.fn(),
  getRecordDetail: vi.fn(),
}));
const target = detailFixture("rules:condition", "Condition").selected;
const definition: TraitDetailView = {
  trait_key: "trait:agile",
  label: "Agile",
  artifact_generation: "test-generation",
  catalogs: ["weaponTraits"],
  source_revision: "pin",
  indexing_locale: "en",
  content: {
    body: {
      kind: "html",
      inclusions: [],
      controls: [],
      html: '<p>A trait definition with a <span data-atlas-reference="0">Condition reference</span>.</p>',
    },
    references: [{ ordinal: 0, target, status: "resolved_record", url: null }],
  },
};
function setup() {
  history.replaceState(null, "", "/search");
  vi.mocked(getTraitDetail).mockResolvedValue(definition);
  vi.mocked(getRecordDetail).mockResolvedValue(
    detailFixture("rules:condition", "Condition"),
  );
  return render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <QueryClientProvider
        client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
      >
        <RouterTestContext>
          <ReferencePreviewProvider>
            <TraitBadge
              trait={{
                kind: "trait",
                value: "agile",
                label: "Agile",
                trait_key: "trait:agile",
              }}
            />
          </ReferencePreviewProvider>
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

it("restores record header trait openers and body scroll after Back", async () => {
  setup();
  const record = detailFixture("rules:condition", "Condition");
  record.presentation.identity.traits = [
    { kind: "trait", value: "agile", label: "Agile", trait_key: "trait:agile" },
  ];
  vi.mocked(getRecordDetail).mockResolvedValue(record);
  fireEvent.click(screen.getByRole("link", { name: "Agile" }));
  fireEvent.click(await screen.findByRole("link", { name: "Condition reference" }));
  const drawer = await screen.findByRole("dialog", { name: "Condition" });
  const badge = within(drawer).getByRole("link", { name: "Agile" });
  const scroll = drawer.querySelector<HTMLElement>(".record-preview-drawer__body")!;
  scroll.scrollTop = 90;
  fireEvent.click(badge);
  expect(await screen.findByRole("dialog", { name: "Agile" })).toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "Back" }));
  await screen.findByRole("dialog", { name: "Condition" });
  expect(scroll.scrollTop).toBe(90);
  expect(within(drawer).getByRole("link", { name: "Agile" })).toHaveFocus();
  expect(screen.getAllByRole("dialog")).toHaveLength(1);
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
