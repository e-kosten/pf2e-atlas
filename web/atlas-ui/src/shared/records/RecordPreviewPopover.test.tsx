import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import type { ReactNode } from "react";
import { recordDetailFixture } from "../../test/recordFixtures";
import { RecordPreviewPopover } from "./RecordPreviewPopover";

const apiMocks = vi.hoisted(() => ({ getRecordDetail: vi.fn() }));

vi.mock("../../api/atlasApi", () => ({
  getRecordDetail: apiMocks.getRecordDetail,
}));

describe("RecordPreviewPopover", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    apiMocks.getRecordDetail.mockImplementation((recordKey: string) =>
      Promise.resolve(recordWithNestedReference(recordKey)),
    );
  });

  it("loads record detail and keeps nested references in the same popover", async () => {
    render(
      <RecordPreviewPopover onOpenFullPage={vi.fn()} recordKey="actors:goblin">
        {(open) => (
          <a aria-expanded={open} href="/records/actors%3Agoblin">
            Open preview
          </a>
        )}
      </RecordPreviewPopover>,
      { wrapper: queryClientWrapper() },
    );

    fireEvent.click(screen.getByRole("link", { name: "Open preview" }));
    await waitFor(() =>
      expect(apiMocks.getRecordDetail).toHaveBeenCalledWith(
        "actors:goblin",
        undefined,
        expect.any(AbortSignal),
      ),
    );
    expect(await screen.findByLabelText("Reference preview")).toBeInTheDocument();
    const header = document.querySelector<HTMLElement>(".preview-popover__header");
    const content = document.querySelector<HTMLElement>(".preview-popover__content");
    expect(header).not.toBeNull();
    expect(content).not.toBeNull();
    expect(within(header!).getByText("Goblin Warrior")).toBeInTheDocument();
    expect(within(content!).queryByText("Goblin Warrior")).not.toBeInTheDocument();

    fireEvent.click(await screen.findByRole("link", { name: "Nested Rule" }));
    await waitFor(() =>
      expect(apiMocks.getRecordDetail).toHaveBeenCalledWith(
        "rules:nested",
        undefined,
        expect.any(AbortSignal),
      ),
    );
    await waitFor(() =>
      expect(within(header!).getByText("Nested Rule")).toBeInTheDocument(),
    );
    expect(within(content!).queryByText("Nested Rule")).not.toBeInTheDocument();
    expect(screen.getAllByLabelText("Reference preview")).toHaveLength(1);
  });

  it("opens the active record on the full-page action", async () => {
    const onOpenFullPage = vi.fn();
    render(
      <RecordPreviewPopover onOpenFullPage={onOpenFullPage} recordKey="actors:goblin">
        {(open) => (
          <button aria-expanded={open} type="button">
            Open preview
          </button>
        )}
      </RecordPreviewPopover>,
      { wrapper: queryClientWrapper() },
    );

    fireEvent.click(screen.getByRole("button", { name: "Open preview" }));
    fireEvent.click(
      await screen.findByRole("button", { name: "Open reference full page" }),
    );

    expect(onOpenFullPage).toHaveBeenCalledWith("actors:goblin");
  });

  it("routes a child reference out of the parent-only preview with its locator", async () => {
    const childLocator = "v1~j~s~706167652d31";
    const onOpenFullPage = vi.fn();
    apiMocks.getRecordDetail.mockImplementation((recordKey: string) =>
      Promise.resolve(recordWithNestedReference(recordKey, childLocator)),
    );
    render(
      <RecordPreviewPopover onOpenFullPage={onOpenFullPage} recordKey="actors:goblin">
        {(open) => (
          <button aria-expanded={open} type="button">
            Open preview
          </button>
        )}
      </RecordPreviewPopover>,
      { wrapper: queryClientWrapper() },
    );

    fireEvent.click(screen.getByRole("button", { name: "Open preview" }));
    fireEvent.click(await screen.findByRole("link", { name: "Nested Rule" }));

    expect(onOpenFullPage).toHaveBeenCalledWith("rules:nested", childLocator);
    expect(apiMocks.getRecordDetail).toHaveBeenCalledTimes(1);
  });
});

function queryClientWrapper() {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return function QueryClientWrapper({ children }: { children: ReactNode }) {
    return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
  };
}

function recordWithNestedReference(recordKey: string, childLocator?: string) {
  const detail = recordDetailFixture({
    recordKey,
    title: recordKey === "rules:nested" ? "Nested Rule" : "Goblin Warrior",
  });
  if (
    recordKey !== "rules:nested" &&
    detail.surface.presentation.presentation_type === "creature"
  ) {
    detail.surface.presentation.body.content = [
      {
        content_key: "nested-reference",
        role: "primary_description",
        authored_order: 0,
        blocks: [
          {
            block_type: "paragraph",
            spans: [
              { span_type: "text", text: "See " },
              {
                span_type: "reference",
                label: "Nested Rule",
                record_key: "rules:nested",
                ...(childLocator ? { child_locator: childLocator } : {}),
                embedded: false,
              },
            ],
          },
        ],
        content_hash: "nested-reference-hash",
        visibility: "public",
        provenance: {
          source_record_key: recordKey,
          relative_source_path: "fixture.json",
          field_family: "fixture.reference",
        },
      },
    ];
  }
  return detail;
}
