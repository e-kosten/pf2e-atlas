import { renderHook } from "@testing-library/react";
import { detailFixture } from "../../test/fixtures";
import { usePreviewReadingPosition } from "./usePreviewReadingPosition";
import type { RecordPreviewContentProps } from "./recordPreviewTypes";

it.each(["popover", "drawer"] as const)(
  "restores the actual %s scroll host only after addressed content arrives and preserves background refresh position",
  (host) => {
    const outer = document.createElement("div");
    const content = document.createElement("section");
    const link = document.createElement("a");
    link.href = "/records/rules:nested";
    content.append(link);
    outer.append(content);
    document.body.append(outer);
    const scroll = host === "drawer" ? outer : content;
    const previous: NonNullable<RecordPreviewContentProps["readingHistory"]> = {
      address: "exact-owner-field-fingerprint",
      restoringHistory: true,
      canGoBack: false,
      onBack: vi.fn(),
      restore: { scroll: 480, focusHref: "/records/rules:nested" },
    };
    const next = { ...previous, address: "next-record", restore: { scroll: 0 } };
    const { result, rerender, unmount } = renderHook(
      ({ detail, history }) => usePreviewReadingPosition(detail, history, host),
      {
        initialProps: {
          detail: undefined as ReturnType<typeof detailFixture> | undefined,
          history: previous,
        },
      },
    );
    result.current.ref.current = content;
    rerender({ detail: detailFixture("rules:prior"), history: previous });
    expect(scroll.scrollTop).toBe(480);
    expect(link).toHaveFocus();
    rerender({ detail: undefined, history: next });
    expect(scroll.scrollTop).toBe(480);
    rerender({ detail: detailFixture("rules:next"), history: next });
    expect(scroll.scrollTop).toBe(0);
    scroll.scrollTop = 215;
    rerender({ detail: detailFixture("rules:next"), history: next });
    expect(scroll.scrollTop).toBe(215);
    rerender({ detail: detailFixture("rules:prior"), history: previous });
    expect(scroll.scrollTop).toBe(480);
    expect(link).toHaveFocus();
    unmount();
    outer.remove();
  },
);
