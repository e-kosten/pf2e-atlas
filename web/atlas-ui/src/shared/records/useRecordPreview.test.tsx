import {
  act,
  fireEvent,
  render,
  renderHook,
  screen,
  waitFor,
} from "@testing-library/react";
import { ConfigProvider, Grid } from "antd";
import { useRecordPreview } from "./useRecordPreview";
import { detailFixture } from "../../test/fixtures";
import { RecordPreviewPopover } from "./RecordPreviewPopover";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
function queryWrapper({ children }: { children: import("react").ReactNode }) {
  return (
    <QueryClientProvider client={new QueryClient()}>{children}</QueryClientProvider>
  );
}

vi.mock("./useRecordDetail", () => ({
  useRecordDetail: () => ({
    data: undefined,
    error: null,
    isLoading: false,
    isFetching: false,
  }),
}));

function PreviewFlow() {
  const preview = useRecordPreview();
  const selection = detailFixture("actors:ghoul").selected;
  return (
    <>
      <button
        onClick={(event) =>
          preview.open("actors:ghoul", event.currentTarget, selection)
        }
      >
        Original reference
      </button>
      {preview.recordKey && (
        <>
          <button
            onClick={(event) =>
              preview.open("rules:nested", event.currentTarget, {
                ...selection,
                record_key: "rules:nested",
              })
            }
          >
            Follow reference
          </button>
          <button onClick={preview.close}>Close preview</button>
        </>
      )}
    </>
  );
}

it("returns focus to the original live trigger after following nested references", () => {
  render(<PreviewFlow />, { wrapper: queryWrapper });
  const original = screen.getByRole("button", { name: "Original reference" });
  act(() => original.focus());
  fireEvent.click(original);
  const follow = screen.getByRole("button", { name: "Follow reference" });
  act(() => follow.focus());
  fireEvent.click(follow);
  const close = screen.getByRole("button", { name: "Close preview" });
  act(() => close.focus());
  fireEvent.click(close);
  expect(original).toHaveFocus();
  expect(
    screen.queryByRole("button", { name: "Close preview" }),
  ).not.toBeInTheDocument();
});

it("returns focus to the original trigger after dismissing a narrow reference drawer", async () => {
  const breakpoint = vi
    .spyOn(Grid, "useBreakpoint")
    .mockReturnValue({ xs: true, md: false });
  function NarrowPreviewFlow() {
    const preview = useRecordPreview();
    return (
      <>
        <button onClick={(event) => preview.open("actors:ghoul", event.currentTarget)}>
          Ghoul reference
        </button>
        {preview.recordKey && (
          <RecordPreviewPopover
            anchor={preview.anchor}
            detail={preview.detail}
            loading={preview.loading}
            onClose={preview.close}
            onOpenFullPage={vi.fn()}
            onReference={preview.open}
          />
        )}
      </>
    );
  }
  try {
    render(
      <ConfigProvider theme={{ token: { motion: false } }}>
        <NarrowPreviewFlow />
      </ConfigProvider>,
      { wrapper: queryWrapper },
    );
    const original = screen.getByRole("button", { name: "Ghoul reference" });
    act(() => original.focus());
    fireEvent.click(original);
    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: "Open reference full page" }),
      ).toHaveFocus(),
    );
    const close = screen.getByRole("button", { name: "Close" });
    act(() => close.focus());
    fireEvent.keyDown(close, { key: "Escape", keyCode: 27 });
    await waitFor(() =>
      expect(screen.queryByRole("button", { name: "Close" })).not.toBeInTheDocument(),
    );
    expect(original).toHaveFocus();
  } finally {
    breakpoint.mockRestore();
  }
});

it("bounds preview history and restores exact selected copy, landing, source link and scroll before final opener dismissal", () => {
  const { result } = renderHook(() => useRecordPreview(), { wrapper: queryWrapper });
  const opener = document.createElement("button");
  document.body.append(opener);
  const first = {
    ...detailFixture("actors:ghoul").selected,
    owners: [{ collection: "items", identity: { SnapshotLocal: { index: 7 } } }],
    field: "system.description.value",
    source_fingerprint: "copy-a",
  };
  const host = document.createElement("section");
  host.className = "record-preview-popover__body";
  const source = document.createElement("a");
  source.href = "/records/rules:next";
  host.append(source);
  document.body.append(host);
  act(() => result.current.open(first.record_key, opener, first, first));
  host.scrollTop = 314;
  act(() => result.current.open("rules:next", source));
  act(() => result.current.readingHistory.onBack());
  expect(result.current.selection).toEqual(first);
  expect(result.current.landing).toEqual(first);
  expect(result.current.readingHistory.restore).toEqual({
    scroll: 314,
    focusHref: "/records/rules:next",
  });
  for (let index = 0; index < 30; index++)
    act(() => result.current.open(`rules:${index}`, source));
  for (let index = 0; index < 24; index++)
    act(() => result.current.readingHistory.onBack());
  expect(result.current.recordKey).toBe("rules:5");
  expect(result.current.readingHistory.canGoBack).toBe(false);
  act(() => result.current.close());
  expect(opener).toHaveFocus();
  expect(result.current.readingHistory.canGoBack).toBe(false);
  host.remove();
  opener.remove();
});
