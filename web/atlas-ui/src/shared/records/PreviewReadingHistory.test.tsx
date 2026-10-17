import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { ConfigProvider, Grid } from "antd";
import { StrictMode } from "react";
import { getOwnedReading, getRecordDetail } from "../../api/atlasApi";
import type { OwnedReadingBatchView } from "../../generated/atlas";
import { SearchProfileProvider } from "../../test/SearchProfileProvider";
import {
  activityFixture,
  actorFixture,
  detailFixture,
  knownFact,
} from "../../test/fixtures";
import { RecordPreviewPopover } from "./RecordPreviewPopover";
import { useRecordPreview } from "./useRecordPreview";

vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  getRecordDetail: vi.fn(),
  getOwnedReading: vi.fn(),
}));

it.each([
  ["popover", "strike", "landed"],
  ["drawer", "strike", "landed"],
  ["popover", "ability", "landed"],
  ["drawer", "ability", "landed"],
  ["popover", "ability", "selected"],
  ["drawer", "ability", "selected"],
] as const)(
  "restores %s history after the %s mounts in %s reading, retaining position and final opener",
  async (host, kind, reading) => {
    vi.spyOn(Grid, "useBreakpoint").mockReturnValue({
      md: host === "popover",
      xs: host === "drawer",
    });
    const activity = { ...activityFixture("Jaws", "actors:ghoul"), kind };
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
    const batch: OwnedReadingBatchView = {
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
    const getAbilities = vi.mocked(getOwnedReading);
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
          {preview.recordKey && (
            <RecordPreviewPopover
              anchor={preview.anchor}
              detail={preview.detail}
              landing={preview.landing}
              readingHistory={preview.readingHistory}
              loading={preview.loading}
              error={preview.error}
              onRetry={preview.retry}
              onClose={preview.close}
              onOpenFullPage={vi.fn()}
              onReference={preview.open}
            />
          )}
        </>
      );
    }
    const selector =
      host === "drawer"
        ? ".record-preview-drawer__body"
        : ".record-preview-popover__body";
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
      // not turn the same owned landing into another first-arrival scroll.
      fireEvent.scroll(scroll);
      expect(scroll.scrollTop).toBe(425);
      nested.focus();
      fireEvent.click(nested);
      await waitFor(() =>
        expect(
          screen.getByRole(host === "drawer" ? "dialog" : "heading", {
            name: "Linked rule",
          }),
        ).toBeVisible(),
      );
      let finish: (() => void) | undefined;
      if (kind === "ability" && reading === "landed") {
        client.removeQueries({ queryKey: ["owned-reading"] });
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
      const closeName = host === "drawer" ? "Close" : "Close reference preview";
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
