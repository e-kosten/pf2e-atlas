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
  it.each(["desktop", "narrow"])(
    "%s preview hands removed content focus to its surviving container and preserves other focus",
    async (viewport) => {
      vi.spyOn(Grid, "useBreakpoint").mockReturnValue({
        xl: viewport === "desktop",
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
      const preview = (showTrigger: boolean, version = 1) => (
        <StrictMode>
          <ConfigProvider theme={{ token: { motion: false } }}>
            <button>Outside preview</button>
            <RecordPreviewPopover
              {...props}
              content={
                showTrigger ? (
                  <button key="follow">Follow reference {version}</button>
                ) : (
                  <h2>Next reference</h2>
                )
              }
            />
          </ConfigProvider>
        </StrictMode>
      );
      const { rerender } = render(preview(true));
      await waitFor(() =>
        expect(
          screen.getByRole("link", { name: "Open reference full page" }),
        ).toHaveFocus(),
      );
      const trigger = screen.getByRole("button", { name: "Follow reference 1" });
      act(() => trigger.focus());
      rerender(preview(true, 2));
      expect(trigger).toHaveFocus();
      rerender(preview(false));
      expect(trigger).not.toBeInTheDocument();
      const content = screen.getByLabelText(
        viewport === "desktop" ? "Reference preview" : "Reference content",
      );
      expect(content).toHaveFocus();
      rerender(preview(true));
      act(() => screen.getByRole("button", { name: "Follow reference 1" }).focus());
      const header = screen.getByRole("link", { name: "Open reference full page" });
      act(() => header.focus());
      rerender(preview(false));
      expect(header).toHaveFocus();
      rerender(preview(true));
      act(() => screen.getByRole("button", { name: "Follow reference 1" }).focus());
      const outside = screen.getByRole("button", { name: "Outside preview" });
      act(() => outside.focus());
      rerender(preview(false));
      expect(outside).toHaveFocus();
      rerender(preview(true));
      const blurTrigger = screen.getByRole("button", { name: "Follow reference 1" });
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
        name: "Close reference preview",
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
    const breakpoint = vi.spyOn(Grid, "useBreakpoint").mockReturnValue({ xl: true });
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
    breakpoint.mockReturnValue({ lg: true, xl: false });
    rerender(preview());
    expect(await screen.findByRole("dialog", { name: "Reference" })).toHaveAttribute(
      "aria-modal",
      "true",
    );
    expect(screen.getByRole("heading", { name: "Dirge of Doom" })).toBeInTheDocument();
    await waitFor(() =>
      expect(
        screen.getByRole("link", { name: "Open reference full page" }),
      ).toHaveFocus(),
    );
    fireEvent.click(screen.getByRole("link", { name: "Open reference full page" }));
    expect(props.onOpenFullPage).toHaveBeenCalledTimes(1);
    breakpoint.mockReturnValue({ xl: true });
    rerender(preview());
    expect(
      await screen.findByRole("dialog", { name: "Reference preview" }),
    ).toBeInTheDocument();
    expect(screen.queryByRole("dialog", { name: "Reference" })).not.toBeInTheDocument();
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
