import { detailFixture } from "../../test/fixtures";
import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { ConfigProvider, Grid } from "antd";
import { StrictMode } from "react";
import type { RecordDetailView } from "../../generated/atlas";
import { RecordPreviewPopover } from "./RecordPreviewPopover";

describe("RecordPreviewPopover", () => {
  afterEach(() => vi.restoreAllMocks());
  it.each(["Escape", "outside click"])(
    "dismisses a pending desktop read with %s",
    (interaction) => {
      vi.spyOn(Grid, "useBreakpoint").mockReturnValue({ md: true });
      const onClose = vi.fn();
      render(
        <RecordPreviewPopover
          anchor={document.createElement("button")}
          detail={undefined}
          loading
          onClose={onClose}
          onOpenFullPage={vi.fn()}
          onReference={vi.fn()}
        />,
      );
      if (interaction === "Escape") fireEvent.keyDown(document, { key: "Escape" });
      else fireEvent.pointerDown(document.body);
      expect(onClose).toHaveBeenCalledOnce();
    },
  );
  it("delays the trigger spinner and opens the content-sized popup when the record arrives", async () => {
    vi.spyOn(Grid, "useBreakpoint").mockReturnValue({ md: true });
    const anchor = document.createElement("button");
    document.body.append(anchor);
    const props = {
      anchor,
      onClose: vi.fn(),
      onOpenFullPage: vi.fn(),
      onReference: vi.fn(),
    };
    const { rerender, unmount } = render(
      <RecordPreviewPopover {...props} detail={undefined} loading />,
      {
        wrapper: ({ children }) => (
          <ConfigProvider theme={{ token: { motion: false } }}>
            {children}
          </ConfigProvider>
        ),
      },
    );
    expect(screen.getByRole("status", { name: "Loading reference" })).toBeVisible();
    const spinner = screen
      .getByRole("status", { name: "Loading reference" })
      .querySelector("[aria-busy]");
    expect(spinner).toHaveAttribute("aria-busy", "false");
    await waitFor(() => expect(spinner).toHaveAttribute("aria-busy", "true"));
    expect(screen.queryByRole("dialog", { name: "Reference preview" })).toBeNull();
    rerender(
      <RecordPreviewPopover
        {...props}
        detail={detailFixture("spells:fear", "Fear")}
        loading={false}
      />,
    );
    await waitFor(() =>
      expect(screen.getByRole("dialog", { name: "Reference preview" })).toBeVisible(),
    );
    expect(screen.queryByRole("status", { name: "Loading reference" })).toBeNull();
    unmount();
    anchor.remove();
  });

  it("opens a failed initial read so Retry and Close remain available", async () => {
    vi.spyOn(Grid, "useBreakpoint").mockReturnValue({ md: true });
    render(
      <RecordPreviewPopover
        anchor={document.createElement("button")}
        detail={undefined}
        loading={false}
        error={new Error("Reference could not load")}
        onRetry={vi.fn()}
        onClose={vi.fn()}
        onOpenFullPage={vi.fn()}
        onReference={vi.fn()}
      />,
      {
        wrapper: ({ children }) => (
          <ConfigProvider theme={{ token: { motion: false } }}>
            {children}
          </ConfigProvider>
        ),
      },
    );
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Retry" })).toBeVisible(),
    );
    expect(
      screen.getByRole("button", { name: "Close reference preview" }),
    ).toBeVisible();
  });
  it.each(["desktop", "narrow"])(
    "%s keeps genuine Back while respecting visible-owner context",
    async (viewport) => {
      vi.spyOn(Grid, "useBreakpoint").mockReturnValue({
        md: viewport === "desktop",
        xs: viewport === "narrow",
      });
      const detail = detailFixture("actors:mage", "Mage");
      detail.selected.owners = [{ collection: "items", identity: { Stable: "fear" } }];
      detail.presentation.identity = { ...detail.record, title: "Fear" };
      const onBack = vi.fn();
      render(
        <ConfigProvider theme={{ token: { motion: false } }}>
          <RecordPreviewPopover
            visibleOwnerRecordKey="actors:mage"
            anchor={document.createElement("button")}
            detail={detail}
            loading={false}
            onClose={vi.fn()}
            onOpenFullPage={vi.fn()}
            onReference={vi.fn()}
            readingHistory={{
              address: "fear",
              restoringHistory: false,
              canGoBack: true,
              onBack,
              restore: { scroll: 0 },
            }}
          />
        </ConfigProvider>,
      );
      expect(screen.queryByRole("link", { name: "Return to Mage" })).toBeNull();
      fireEvent.click(await screen.findByRole("button", { name: "Back" }));
      expect(onBack).toHaveBeenCalledOnce();
    },
  );
  it.each(["desktop", "narrow"])(
    "%s preview hands removed content focus to its surviving container and preserves other focus",
    async (viewport) => {
      vi.spyOn(Grid, "useBreakpoint").mockReturnValue({
        md: viewport === "desktop",
        xs: viewport === "narrow",
      });
      const props = {
        anchor: document.createElement("button"),
        detail: undefined,
        loading: false,
        fullPageHref: "/records/rules:next",
        onClose: vi.fn(),
        onOpenFullPage: vi.fn(),
        onReference: vi.fn(),
      };
      const preview = (showTrigger: boolean, version = 1) => {
        const detail = detailFixture("rules:current", `Follow reference ${version}`);
        detail.presentation.identity = { ...detail.record, title: "Next reference" };
        detail.selected.owners = showTrigger
          ? [{ collection: "/items", identity: { Stable: "selected-owner" } }]
          : [];
        return (
          <StrictMode>
            <ConfigProvider theme={{ token: { motion: false } }}>
              <button>Outside preview</button>
              <RecordPreviewPopover {...props} detail={detail} />
            </ConfigProvider>
          </StrictMode>
        );
      };
      const { rerender } = render(preview(true));
      await waitFor(() =>
        expect(
          screen.getByRole("link", { name: "Open reference full page" }),
        ).toHaveFocus(),
      );
      const trigger = screen.getByRole("link", {
        name: "Return to Follow reference 1",
      });
      act(() => trigger.focus());
      rerender(preview(true, 2));
      expect(trigger).toHaveFocus();
      rerender(preview(false));
      expect(trigger).not.toBeInTheDocument();
      const content =
        viewport === "desktop"
          ? document.querySelector(".record-preview-popover__body")!
          : screen.getByLabelText("Reference content");
      expect(content).toHaveFocus();
      rerender(preview(true));
      act(() =>
        screen.getByRole("link", { name: "Return to Follow reference 1" }).focus(),
      );
      const header = screen.getByRole("link", { name: "Open reference full page" });
      act(() => header.focus());
      rerender(preview(false));
      expect(header).toHaveFocus();
      rerender(preview(true));
      act(() =>
        screen.getByRole("link", { name: "Return to Follow reference 1" }).focus(),
      );
      const outside = screen.getByRole("button", { name: "Outside preview" });
      act(() => outside.focus());
      rerender(preview(false));
      expect(outside).toHaveFocus();
      rerender(preview(true));
      const blurTrigger = screen.getByRole("link", {
        name: "Return to Follow reference 1",
      });
      act(() => blurTrigger.focus());
      act(() => blurTrigger.blur());
      expect(document.body).toHaveFocus();
      rerender(preview(false));
      expect(document.body).toHaveFocus();
    },
  );

  it.each(["desktop", "narrow"])(
    "%s preview focuses actions and lets a nested source modal own Escape",
    async (viewport) => {
      vi.spyOn(Grid, "useBreakpoint").mockReturnValue({
        xs: viewport === "narrow",
        sm: viewport === "desktop",
        md: viewport === "desktop",
        xl: viewport === "desktop",
      });
      const close = vi.fn();
      const detail = recordDetailFixture();
      detail.presentation.content[0].body = {
        kind: "html",
        html: '<span data-atlas-interaction="0">Fortitude check</span>',
        controls: [
          {
            ordinal: 0,
            control: { kind: "check", statistic: "Fortitude", options: { dc: "15" } },
          },
        ],
      };
      render(
        <StrictMode>
          <ConfigProvider theme={{ token: { motion: false } }}>
            <RecordPreviewPopover
              anchor={document.createElement("button")}
              detail={detail}
              loading={false}
              onClose={close}
              onOpenFullPage={vi.fn()}
              onReference={vi.fn()}
            />{" "}
          </ConfigProvider>
        </StrictMode>,
      );
      await waitFor(() =>
        expect(
          screen.getByRole("link", { name: "Open reference full page" }),
        ).toHaveFocus(),
      );
      const marker = screen.getByRole("button", { name: "Fortitude check" });
      marker.focus();
      fireEvent.keyDown(marker, { key: "Enter" });
      await waitFor(() => expect(screen.getAllByRole("dialog")).toHaveLength(2));
      const modal = screen.getByRole("dialog", { name: "Fortitude" });
      expect(modal).toHaveTextContent("Fortitude");
      expect(close).not.toHaveBeenCalled();
      const modalAction = within(modal).getByRole("button", { name: "OK" });
      act(() => modalAction.focus());
      fireEvent.keyDown(modalAction, { key: "Escape", keyCode: 27 });
      expect(close).not.toHaveBeenCalled();
      await waitFor(() => expect(modal).not.toBeInTheDocument());
      expect(marker).toHaveFocus();
      const closeAction = screen.getByRole("button", {
        name: viewport === "desktop" ? "Close reference preview" : "Close",
      });
      act(() => closeAction.focus());
      fireEvent.keyDown(closeAction, {
        key: "Escape",
      });
      await waitFor(() => expect(close).toHaveBeenCalledTimes(1));
    },
  );

  it("opens nested references without re-anchoring the active popover", () => {
    const onReference = vi.fn();
    render(
      <RecordPreviewPopover
        anchor={document.createElement("button")}
        detail={recordDetailFixture()}
        loading={false}
        onClose={vi.fn()}
        onOpenFullPage={vi.fn()}
        onReference={onReference}
      />,
    );

    fireEvent.click(screen.getByText("Nested Rule"));

    expect(onReference).toHaveBeenCalledWith(
      "rules:nested",
      expect.any(HTMLElement),
      expect.objectContaining({ record_key: "rules:nested" }),
    );
  });

  it("preserves the selected reference and actions when moving between desktop and narrow layouts", async () => {
    const breakpoint = vi.spyOn(Grid, "useBreakpoint").mockReturnValue({ md: true });
    const props = {
      anchor: document.createElement("button"),
      detail: recordDetailFixture(),
      loading: false,
      onClose: vi.fn(),
      onOpenFullPage: vi.fn(),
      onReference: vi.fn(),
    };
    const preview = () => (
      <ConfigProvider theme={{ token: { motion: false } }}>
        <RecordPreviewPopover {...props} />
      </ConfigProvider>
    );
    const { rerender } = render(preview());
    expect(
      await screen.findByRole("dialog", { name: "Reference preview" }),
    ).toBeInTheDocument();
    breakpoint.mockReturnValue({ md: false });
    rerender(preview());
    expect(
      await screen.findByRole("dialog", { name: "Dirge of Doom" }),
    ).toHaveAttribute("aria-modal", "true");
    expect(
      within(screen.getByRole("dialog", { name: "Dirge of Doom" })).getByText(
        "Dirge of Doom",
      ),
    ).toBeVisible();
    await waitFor(() =>
      expect(
        screen.getByRole("link", { name: "Open reference full page" }),
      ).toHaveFocus(),
    );
    fireEvent.click(screen.getByRole("link", { name: "Open reference full page" }));
    expect(props.onOpenFullPage).toHaveBeenCalledTimes(1);
    breakpoint.mockReturnValue({ md: true });
    rerender(preview());
    expect(
      await screen.findByRole("dialog", { name: "Reference preview" }),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("dialog", { name: "Dirge of Doom" }),
    ).not.toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Dirge of Doom" })).toBeInTheDocument();
    await waitFor(() =>
      expect(
        screen.getByRole("link", { name: "Open reference full page" }),
      ).toHaveFocus(),
    );
    fireEvent.click(screen.getByRole("button", { name: "Close reference preview" }));
    expect(props.onClose).toHaveBeenCalledTimes(1);
  });
});

function recordDetailFixture(): RecordDetailView {
  return detailFixture("spell:dirge", "Dirge of Doom", "rules:nested");
}
