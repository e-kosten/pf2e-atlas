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
import { RecordReferenceDrawer } from "./RecordReferenceDrawer";

describe("RecordReferenceDrawer", () => {
  afterEach(() => vi.restoreAllMocks());
  it.each(["Escape", "outside click"])(
    "dismisses a pending desktop read with %s",
    async (interaction) => {
      vi.spyOn(Grid, "useBreakpoint").mockReturnValue({ md: true });
      const onClose = vi.fn();
      render(
        <ConfigProvider theme={{ token: { motion: false } }}>
          <RecordReferenceDrawer
            anchor={document.createElement("button")}
            detail={undefined}
            loading
            onClose={onClose}
            onOpenFullPage={vi.fn()}
            onReference={vi.fn()}
          />
        </ConfigProvider>,
      );
      const heading = screen.getByRole("heading", { name: "Reference" });
      await waitFor(() => expect(heading).toHaveFocus());
      if (interaction === "Escape")
        fireEvent.keyDown(heading, { key: "Escape", keyCode: 27 });
      else fireEvent.click(document.querySelector(".ant-drawer-mask")!);
      await waitFor(() => expect(onClose).toHaveBeenCalledOnce());
    },
  );
  it("opens the pending Drawer immediately, delays its spinner and retains its heading when content arrives", async () => {
    const anchor = document.createElement("button");
    document.body.append(anchor);
    const props = {
      anchor,
      onClose: vi.fn(),
      onOpenFullPage: vi.fn(),
      onReference: vi.fn(),
    };
    const { rerender, unmount } = render(
      <RecordReferenceDrawer {...props} detail={undefined} loading />,
      {
        wrapper: ({ children }) => (
          <ConfigProvider theme={{ token: { motion: false } }}>
            {children}
          </ConfigProvider>
        ),
      },
    );
    try {
      const dialog = screen.getByRole("dialog", { name: "Reference" });
      const heading = screen.getByRole("heading", { name: "Reference" });
      expect(dialog).toHaveAttribute("aria-modal", "true");
      expect(screen.getByRole("status")).toHaveAccessibleName("Loading record…");
      expect(dialog.querySelector('.ant-spin[aria-busy="true"]')).toBeNull();
      await waitFor(() =>
        expect(dialog.querySelector('.ant-spin[aria-busy="true"]')).not.toBeNull(),
      );
      rerender(
        <RecordReferenceDrawer
          {...props}
          detail={detailFixture("spells:fear", "Fear")}
          loading={false}
        />,
      );
      expect(screen.getByRole("dialog", { name: "Fear" })).toBe(dialog);
      expect(screen.getByRole("heading", { name: "Fear" })).toBe(heading);
      expect(heading).toHaveFocus();
      expect(screen.queryByRole("status")).toBeNull();
    } finally {
      unmount();
      anchor.remove();
    }
  });

  it("opens a failed initial read so Retry and Close remain available", async () => {
    vi.spyOn(Grid, "useBreakpoint").mockReturnValue({ md: true });
    render(
      <RecordReferenceDrawer
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
  it("dismisses an action tooltip before the containing Drawer on consecutive Escape presses", async () => {
    const onClose = vi.fn();
    render(
      <ConfigProvider theme={{ token: { motion: false } }}>
        <RecordReferenceDrawer
          anchor={null}
          detail={recordDetailFixture()}
          loading={false}
          onClose={onClose}
          onOpenFullPage={vi.fn()}
          onReference={vi.fn()}
        />
      </ConfigProvider>,
    );
    await waitFor(() =>
      expect(screen.getByRole("heading", { name: "Dirge of Doom" })).toHaveFocus(),
    );
    const close = screen.getByRole("button", { name: "Close reference preview" });
    act(() => close.focus());
    await screen.findByRole("tooltip", { name: "Close reference" });
    fireEvent.keyDown(close, { key: "Escape" });
    await waitFor(() => expect(screen.queryByRole("tooltip")).toBeNull());
    expect(onClose).not.toHaveBeenCalled();
    expect(close).toHaveFocus();
    fireEvent.keyDown(close, { key: "Escape" });
    await waitFor(() => expect(onClose).toHaveBeenCalledOnce());
  });
  it.each(["desktop", "narrow"])(
    "%s encounter preview keeps one shared Close beside its actions",
    async (viewport) => {
      vi.spyOn(Grid, "useBreakpoint").mockReturnValue({ md: viewport === "desktop" });
      const onClose = vi.fn();
      render(
        <ConfigProvider theme={{ token: { motion: false } }}>
          <RecordReferenceDrawer
            context="encounter"
            actions={<button>Cast</button>}
            anchor={document.createElement("button")}
            detail={detailFixture("spells:fear", "Fear")}
            loading={false}
            onClose={onClose}
            onOpenFullPage={vi.fn()}
            onReference={vi.fn()}
          />
        </ConfigProvider>,
      );
      const close = await screen.findByRole("button", {
        name: "Close reference preview",
      });
      expect(
        screen.getByRole("button", { name: "Cast" }).closest(".record-preview-toolbar"),
      ).toContainElement(close);
      expect(screen.queryByRole("button", { name: "Close" })).toBeNull();
      fireEvent.click(close);
      await waitFor(() => expect(onClose).toHaveBeenCalledOnce());
    },
  );
  it.each(["desktop", "narrow"])(
    "%s keeps genuine Back while respecting visible-owner context",
    async (viewport) => {
      vi.spyOn(Grid, "useBreakpoint").mockReturnValue({
        md: viewport === "desktop",
        xs: viewport === "narrow",
      });
      const detail = detailFixture("actors:mage", "Mage");
      detail.selected.embedded_path = [
        { collection: "items", identity: { Stable: "fear" } },
      ];
      detail.presentation.identity = { ...detail.record, title: "Fear" };
      const onBack = vi.fn();
      render(
        <ConfigProvider theme={{ token: { motion: false } }}>
          <RecordReferenceDrawer
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
        detail.selected.embedded_path = showTrigger
          ? [{ collection: "/items", identity: { Stable: "selected-owner" } }]
          : [];
        return (
          <StrictMode>
            <ConfigProvider theme={{ token: { motion: false } }}>
              <button>Outside preview</button>
              <RecordReferenceDrawer {...props} detail={detail} />
            </ConfigProvider>
          </StrictMode>
        );
      };
      const { rerender } = render(preview(true));
      await waitFor(() =>
        expect(screen.getByRole("heading", { name: "Next reference" })).toHaveFocus(),
      );
      const trigger = screen.getByRole("link", {
        name: "Return to Follow reference 1",
      });
      act(() => trigger.focus());
      rerender(preview(true, 2));
      expect(trigger).toHaveFocus();
      rerender(preview(false));
      expect(trigger).not.toBeInTheDocument();
      const content = screen.getByRole("heading", { name: "Next reference" });
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
    "%s preview focuses the heading and lets a nested source modal own Escape",
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
        inclusions: [],
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
            <RecordReferenceDrawer
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
        expect(screen.getByRole("heading", { name: "Dirge of Doom" })).toHaveFocus(),
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
      const heading = screen.getByRole("heading", { name: "Dirge of Doom" });
      act(() => heading.focus());
      fireEvent.keyDown(heading, {
        key: "Escape",
        keyCode: 27,
      });
      await waitFor(() => expect(close).toHaveBeenCalledTimes(1));
    },
  );

  it("opens nested references in the existing Drawer session", () => {
    const onReference = vi.fn();
    render(
      <RecordReferenceDrawer
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

  it("preserves the same document, actions and focus when moving between desktop and narrow layouts", async () => {
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
        <RecordReferenceDrawer {...props} />
      </ConfigProvider>
    );
    const { rerender } = render(preview());
    const dialog = await screen.findByRole("dialog", { name: "Dirge of Doom" });
    const heading = screen.getByRole("heading", { name: "Dirge of Doom" });
    await waitFor(() => expect(heading).toHaveFocus());
    const full = screen.getByRole("link", { name: "Open reference full page" });
    act(() => full.focus());
    breakpoint.mockReturnValue({ md: false });
    rerender(preview());
    expect(screen.getByRole("dialog", { name: "Dirge of Doom" })).toBe(dialog);
    expect(dialog).toHaveAttribute("aria-modal", "true");
    expect(screen.getByRole("heading", { name: "Dirge of Doom" })).toBe(heading);
    expect(full).toHaveFocus();
    fireEvent.click(full);
    expect(props.onOpenFullPage).toHaveBeenCalledTimes(1);
    breakpoint.mockReturnValue({ md: true });
    rerender(preview());
    expect(screen.getByRole("dialog", { name: "Dirge of Doom" })).toBe(dialog);
    expect(screen.getByRole("link", { name: "Open reference full page" })).toBe(full);
    expect(full).toHaveFocus();
    fireEvent.click(screen.getByRole("button", { name: "Close reference preview" }));
    await waitFor(() => expect(props.onClose).toHaveBeenCalledTimes(1));
  });
});

function recordDetailFixture(): RecordDetailView {
  return detailFixture("spell:dirge", "Dirge of Doom", "rules:nested");
}
