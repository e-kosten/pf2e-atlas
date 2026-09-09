import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import type { ReferenceHandler } from "../../shared/records/RecordRichContent";
import { recordSummaryFixture } from "../../test/recordFixtures";
import { ResultTable } from "./ResultTable";
import type { SearchWorkspaceState } from "./useSearchWorkspace";

vi.mock("../../shared/records/RecordSurface", () => ({
  RecordSurface: ({ onReference }: { onReference: ReferenceHandler }) => (
    <button
      onClick={(event) => {
        event.stopPropagation();
        onReference("journals:target", "v1~j~s~706167652d31");
      }}
      type="button"
    >
      Exact child
    </button>
  ),
}));

it("routes a result-card child reference without selecting only its parent", async () => {
  const childLocator = "v1~j~s~706167652d31";
  const selectRecord = vi.fn();
  const workspace = {
    activeResultKey: null,
    focusResult: vi.fn(),
    moveResultSelection: vi.fn(),
    openActiveResult: vi.fn(),
    resultPage: {
      window_id: 1n,
      mode: { kind: "text_search", query: "exact child" },
      page: { number: 1, size: 25, count: 1, total: 1n, has_more: false },
      rows: [
        {
          record: recordSummaryFixture(),
        },
      ],
    },
    resultsLoading: false,
    resultsRefreshing: false,
    selectRecord,
  } as unknown as SearchWorkspaceState;
  history.replaceState(null, "", "/search");

  render(<ResultTable workspace={workspace} />);
  const exactChildButton = screen
    .getAllByRole("button", { name: "Exact child" })
    .find((element) => element.tagName === "BUTTON");
  expect(exactChildButton).toBeDefined();
  fireEvent.click(exactChildButton!);

  await waitFor(() =>
    expect(window.location.pathname).toBe("/records/journals%3Atarget"),
  );
  expect(window.location.search).toBe(`?child=${encodeURIComponent(childLocator)}`);
  expect(selectRecord).not.toHaveBeenCalled();
});
