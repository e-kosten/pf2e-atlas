import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { ConfigProvider, Grid } from "antd";
import { StrictMode } from "react";
import {
  getDefinitionDetail,
  getEmbeddedReading,
  getRecordDetail,
} from "../../api/atlasApi";
import type {
  DefinitionDetailView,
  EmbeddedReadingBatchView,
} from "../../generated/atlas";
import { RouterTestContext } from "../../test/router";
import { SearchProfileProvider } from "../../test/SearchProfileProvider";
import {
  activityFixture,
  actorFixture,
  detailFixture,
  knownFact,
} from "../../test/fixtures";
import { RecordPreviewDrawer } from "./RecordPreviewDrawer";
import { useRecordPreview } from "./useRecordPreview";

vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  getRecordDetail: vi.fn(),
  getEmbeddedReading: vi.fn(),
  getDefinitionDetail: vi.fn(),
}));

it.each([
  ["desktop", "strike", "landed"],
  ["narrow", "strike", "landed"],
  ["desktop", "ability", "landed"],
  ["narrow", "ability", "landed"],
  ["desktop", "ability", "selected"],
  ["narrow", "ability", "selected"],
] as const)(
  "restores %s history after the %s mounts in %s reading, retaining position and final opener",
  async (host, kind, reading) => {
    vi.spyOn(Grid, "useBreakpoint").mockReturnValue({
      md: host === "desktop",
      xs: host === "narrow",
    });
    const activity = {
      ...activityFixture("Jaws", "actors:ghoul"),
      kind,
      family: kind === "ability" ? "action" : "melee",
      ability_group: kind === "ability" ? ("primary" as const) : null,
    };
    const parent = detailFixture(
      "actors:ghoul",
      "Ghoul",
      kind === "strike" ? "rules:linked" : undefined,
    );
    parent.presentation.body = {
      kind: "creature",
      value: { ...actorFixture(), activities: knownFact([activity]) },
    };
    const content = detailFixture("actors:ghoul", "Ghoul", "rules:linked");
    if (reading === "selected") {
      parent.selected = activity.navigation;
      parent.relationships = content.relationships;
      parent.presentation = {
        ...detailFixture("actors:ghoul", "Jaws", "rules:linked").presentation,
        body: { kind: "activity", value: activity },
      };
    }
    const batch: EmbeddedReadingBatchView = {
      artifact_generation: parent.artifact_generation,
      parent: parent.selected,
      availability: "value",
      total: 1,
      start: 0,
      end: 1,
      previous: null,
      next: null,
      entries: [
        {
          navigation: activity.navigation,
          content: content.presentation.content,
          relationships: content.relationships,
        },
      ],
    };
    const getDetail = vi.mocked(getRecordDetail);
    const getAbilities = vi.mocked(getEmbeddedReading);
    getDetail.mockReset();
    getAbilities.mockReset();
    getDetail.mockImplementation((key) =>
      Promise.resolve(
        key === "actors:ghoul" ? parent : detailFixture(key, "Linked rule"),
      ),
    );
    getAbilities.mockResolvedValue(batch);
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    function Flow() {
      const preview = useRecordPreview();
      return (
        <>
          <button
            onClick={(event) =>
              preview.open(
                parent.record.record_key,
                event.currentTarget,
                parent.selected,
                reading === "landed" ? activity.navigation : undefined,
              )
            }
          >
            Open landed parent
          </button>
          <RecordPreviewDrawer preview={preview} />
        </>
      );
    }
    const selector = ".record-preview-drawer__body";
    const root = document.documentElement;
    const priorOuterScroll = root.scrollTop;
    root.scrollTop = 0;
    // The scrolling library discovers ancestors by dimensions before measuring
    // their rectangles, so provide geometry from the instant each host mounts.
    for (const [property, value] of Object.entries({
      clientHeight: 300,
      offsetHeight: 300,
      scrollHeight: 3000,
      clientWidth: 500,
      offsetWidth: 500,
      scrollWidth: 500,
    }) as Array<
      [
        (
          | "clientHeight"
          | "offsetHeight"
          | "scrollHeight"
          | "clientWidth"
          | "offsetWidth"
          | "scrollWidth"
        ),
        number,
      ]
    >) {
      const original =
        Object.getOwnPropertyDescriptor(HTMLElement.prototype, property)?.get ??
        Object.getOwnPropertyDescriptor(Element.prototype, property)!.get!;
      vi.spyOn(HTMLElement.prototype, property, "get").mockImplementation(function (
        this: HTMLElement,
      ) {
        if (!this.matches(selector)) return original.call(this);
        this.style.overflowY = "auto";
        this.style.overflowX = "auto";
        this.style.border = "0px solid transparent";
        return value;
      });
    }
    const originalRect = HTMLElement.prototype.getBoundingClientRect;
    // The desktop host mounts only with ready content. Install jsdom dimensions
    // on its first layout measurement, before the real landing effect runs.
    vi.spyOn(HTMLElement.prototype, "getBoundingClientRect").mockImplementation(
      function (this: HTMLElement) {
        const scroll = document.querySelector<HTMLElement>(selector);
        if (scroll && this === scroll) return new DOMRect(0, 50, 500, 300);
        if (scroll?.contains(this) && this.hasAttribute("data-atlas-activity-anchor"))
          return new DOMRect(0, 150 - scroll.scrollTop, 400, 30);
        return originalRect.call(this);
      },
    );
    const originalScroll = HTMLElement.prototype.scrollIntoView;
    HTMLElement.prototype.scrollIntoView = function () {
      const scroll = this.closest<HTMLElement>(selector);
      if (scroll) {
        scroll.scrollTop = 100;
        root.scrollTop = 719;
      }
    };
    try {
      render(
        <StrictMode>
          <QueryClientProvider client={client}>
            <SearchProfileProvider>
              <ConfigProvider theme={{ token: { motion: false } }}>
                <Flow />
              </ConfigProvider>
            </SearchProfileProvider>
          </QueryClientProvider>
        </StrictMode>,
      );
      const opener = screen.getByRole("button", { name: "Open landed parent" });
      fireEvent.click(opener);
      await waitFor(() => expect(document.querySelector(selector)).not.toBeNull());
      const scroll = document.querySelector<HTMLElement>(selector)!;
      const nested = await screen.findByRole("link", { name: "Nested Rule" });
      await waitFor(() =>
        expect(scroll.scrollTop).toBe(reading === "landed" ? 100 : 0),
      );
      expect(document.documentElement.scrollTop).toBe(0);
      scroll.scrollTop = 425;
      // Preview positioning reacts to captured scroll events. Its rerender must
      // not turn the same embedded landing into another first-arrival scroll.
      fireEvent.scroll(scroll);
      expect(scroll.scrollTop).toBe(425);
      nested.focus();
      fireEvent.click(nested);
      await waitFor(() =>
        expect(
          screen.getByRole(host === "narrow" ? "dialog" : "heading", {
            name: "Linked rule",
          }),
        ).toBeVisible(),
      );
      let finish: (() => void) | undefined;
      if (kind === "ability" && reading === "landed") {
        client.removeQueries({ queryKey: ["embedded-reading"] });
        getAbilities.mockImplementationOnce(
          () =>
            new Promise((resolve) => {
              finish = () => resolve(batch);
            }),
        );
      }
      fireEvent.click(screen.getByRole("button", { name: "Back" }));
      if (kind === "ability" && reading === "landed") {
        await waitFor(() => expect(finish).toBeDefined());
        scroll.scrollTop = 0;
        expect(screen.queryByRole("link", { name: "Nested Rule" })).toBeNull();
        await act(async () => finish!());
      }
      const returned = await screen.findByRole("link", { name: "Nested Rule" });
      await waitFor(() => expect(scroll.scrollTop).toBe(425));
      expect(returned).toHaveFocus();
      expect(document.documentElement.scrollTop).toBe(0);
      const closeName = "Close reference preview";
      fireEvent.click(screen.getByRole("button", { name: closeName }));
      await waitFor(() =>
        expect(screen.queryByRole("button", { name: closeName })).toBeNull(),
      );
      expect(opener).toHaveFocus();
      // Allow the new host's jsdom geometry to be installed before its cached
      // target could synchronously mount. The address is still a fresh opening.
      client.removeQueries({ queryKey: ["record-detail"] });
      fireEvent.click(opener);
      await screen.findByRole("link", { name: "Nested Rule" });
      await waitFor(() =>
        expect(document.querySelector<HTMLElement>(selector)!.scrollTop).toBe(
          reading === "landed" ? 100 : 0,
        ),
      );
    } finally {
      cleanup();
      client.clear();
      vi.restoreAllMocks();
      HTMLElement.prototype.scrollIntoView = originalScroll;
      root.scrollTop = priorOuterScroll;
    }
  },
);

it("restores a header trait trigger and body scroll through mixed history without leaving its Drawer", async () => {
  const actor = detailFixture(
    "actors:abrikandilu",
    "Abrikandilu",
    "abilities:darkvision",
  );
  const darkvision = detailFixture(
    "abilities:darkvision",
    "Darkvision",
    "spells:darkness",
  );
  const darkness = detailFixture("spells:darkness", "Darkness", "rules:unused");
  for (const [detail, label] of [
    [actor, "Darkvision"],
    [darkvision, "Darkness"],
  ] as const) {
    const body = detail.presentation.content[0].body;
    if (body.kind === "html")
      body.html = `<p><a data-atlas-reference="0" href="#">${label}</a></p>`;
  }
  darkness.presentation.content[0].body = { kind: "plain", text: "Darkness body." };
  const help = {
    definition_key: "trait:concentrate",

    destination: { kind: "definition" as const, key: "trait:concentrate" },
  };
  darkness.presentation.identity.traits = [
    {
      kind: "trait",
      trait_key: "trait:concentrate",
      value: "concentrate",
      label: "Concentrate",
      help,
    },
  ];
  const trait: DefinitionDetailView = {
    artifact_generation: darkness.artifact_generation,
    definition_key: "trait:concentrate",
    label: "Concentrate",
    help,
    content: {
      body: { kind: "plain", text: "Concentrate trait body." },
      references: [],
    },

    notices: [],

    related_filters: [],
    publications: [],
  };
  const records = new Map(
    [actor, darkvision, darkness].map((detail) => [detail.record.record_key, detail]),
  );
  vi.mocked(getRecordDetail).mockImplementation(async (key) => records.get(key)!);
  vi.mocked(getDefinitionDetail).mockResolvedValue(trait);
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  function Flow() {
    const preview = useRecordPreview();
    return (
      <>
        <a href="/traits/trait:concentrate">Unrelated host link</a>
        <button
          onClick={(event) =>
            preview.open(actor.record.record_key, event.currentTarget, actor.selected)
          }
        >
          Open Abrikandilu
        </button>
        <RecordPreviewDrawer preview={preview} />
      </>
    );
  }
  render(
    <RouterTestContext>
      <QueryClientProvider client={client}>
        <ConfigProvider theme={{ token: { motion: false } }}>
          <Flow />
        </ConfigProvider>
      </QueryClientProvider>
    </RouterTestContext>,
  );
  const opener = screen.getByRole("button", { name: "Open Abrikandilu" });
  opener.focus();
  fireEvent.click(opener);
  let drawer = await screen.findByRole("dialog", { name: "Abrikandilu" });
  const pane = drawer.querySelector<HTMLElement>(".record-preview-drawer__body")!;
  pane.scrollTop = 112;
  const darkvisionLink = within(drawer).getByRole("link", { name: "Darkvision" });
  darkvisionLink.focus();
  fireEvent.click(darkvisionLink);
  drawer = await screen.findByRole("dialog", { name: "Darkvision" });
  pane.scrollTop = 265;
  const darknessLink = within(drawer).getByRole("link", { name: "Darkness" });
  darknessLink.focus();
  fireEvent.click(darknessLink);
  drawer = await screen.findByRole("dialog", { name: "Darkness" });
  pane.scrollTop = 650;
  const header = within(drawer)
    .getByRole("heading", { name: "Darkness" })
    .closest("header")!;
  const concentrate = within(header).getByRole("link", { name: "Concentrate" });
  concentrate.focus();
  fireEvent.click(concentrate);
  drawer = await screen.findByRole("dialog", { name: "Concentrate" });
  expect(within(drawer).getByText("Concentrate trait body.")).toBeVisible();
  fireEvent.click(within(drawer).getByRole("button", { name: "Back" }));
  drawer = await screen.findByRole("dialog", { name: "Darkness" });
  const returnedHeader = within(drawer)
    .getByRole("heading", { name: "Darkness" })
    .closest("header")!;
  await waitFor(() =>
    expect(
      within(returnedHeader).getByRole("link", { name: "Concentrate" }),
    ).toHaveFocus(),
  );
  expect(pane.scrollTop).toBe(650);
  expect(screen.getByRole("link", { name: "Unrelated host link" })).not.toHaveFocus();
  fireEvent.click(within(drawer).getByRole("button", { name: "Back" }));
  drawer = await screen.findByRole("dialog", { name: "Darkvision" });
  await waitFor(() =>
    expect(within(drawer).getByRole("link", { name: "Darkness" })).toHaveFocus(),
  );
  expect(pane.scrollTop).toBe(265);
  fireEvent.click(within(drawer).getByRole("button", { name: "Back" }));
  drawer = await screen.findByRole("dialog", { name: "Abrikandilu" });
  await waitFor(() =>
    expect(within(drawer).getByRole("link", { name: "Darkvision" })).toHaveFocus(),
  );
  expect(pane.scrollTop).toBe(112);
  fireEvent.click(
    within(drawer).getByRole("button", { name: "Close reference preview" }),
  );
  await waitFor(() => expect(opener).toHaveFocus());
});
