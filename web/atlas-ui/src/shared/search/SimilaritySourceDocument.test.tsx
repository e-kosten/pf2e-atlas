import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { ConfigProvider } from "antd";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { SimilarityPartView, RecordDetailView } from "../../generated/atlas";
import { detailFixture } from "../../test/fixtures";
import { SimilaritySourceDocument } from "./SimilaritySourceDocument";
import { rootNavigation } from "../records/useReferenceReading";

const api = vi.hoisted(() => ({ detail: vi.fn(), trait: vi.fn() }));
vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  getRecordDetail: api.detail,
  getTraitDetail: api.trait,
}));
const recordKey = "hazards:brown-mold";
const seed = { kind: "record", record_key: recordKey } as const;
const part: SimilarityPartView = {
  selector: "section-reset",
  label: "Reset",
  owner_label: "Brown Mold",
  owners: [],
  field: "/system/details/reset",
  section_ordinal: 0,
  role: "reset",
  role_label: "Reset",
  has_vectors: true,
  label_only: false,
  navigation: {
    ...rootNavigation(recordKey),
    field: "/system/details/reset",
    source_fingerprint: "snapshot-1",
  },
};
function content(detail: RecordDetailView, text: string) {
  detail.presentation.content = [
    {
      locator: {
        record: detail.record.record_key,
        embedded_path: [],
        field: "/system/description/value",
      },
      role: "description",
      source_fingerprint: "snapshot-1",
      body: { kind: "plain", text },
    },
  ];
  return detail;
}
function wrapper({ children }: { children: React.ReactNode }) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return (
    <QueryClientProvider client={client}>
      <ConfigProvider theme={{ token: { motion: false } }}>{children}</ConfigProvider>
    </QueryClientProvider>
  );
}
beforeEach(() => vi.clearAllMocks());

it("starts with the full seed, opens the supplied source address, and restores whole-document reading position", async () => {
  const root = content(detailFixture(recordKey, "Brown Mold"), "Full source document");
  const section = content(
    detailFixture(recordKey, "Brown Mold"),
    "Reset section source",
  );
  section.selected = part.navigation!;
  api.detail.mockImplementation((_key, request) =>
    Promise.resolve(request?.fields?.length ? section : root),
  );
  const view = render(
    <SimilaritySourceDocument
      seed={seed}
      generation="test-generation"
      target={null}
      onReloadSections={vi.fn()}
      active
    />,
    { wrapper },
  );
  await screen.findByText("Full source document");
  const host = screen
    .getByText("Full source document")
    .closest(".reference-reader__scroll")!;
  host.scrollTop = 183;
  fireEvent.scroll(host);
  view.rerender(
    <SimilaritySourceDocument
      seed={seed}
      generation="test-generation"
      target={{ part, visit: 1 }}
      onReloadSections={vi.fn()}
      active
    />,
  );
  await screen.findByText("Reset section source");
  expect(api.detail).toHaveBeenLastCalledWith(
    recordKey,
    expect.objectContaining({ fields: [part.field], source_fingerprint: "snapshot-1" }),
    expect.any(AbortSignal),
  );
  expect(screen.getByRole("heading", { name: "Brown Mold" })).toHaveFocus();
  fireEvent.click(screen.getByRole("button", { name: "Whole document" }));
  await screen.findByText("Full source document");
  await waitFor(() => expect(host.scrollTop).toBe(183));
});

it("isolates stale source responses when the seed changes", async () => {
  let finish!: (detail: RecordDetailView) => void;
  api.detail.mockImplementation((key: string) =>
    key === recordKey
      ? new Promise<RecordDetailView>((resolve) => (finish = resolve))
      : Promise.resolve(content(detailFixture(key, "Heal"), "Healing source")),
  );
  const view = render(
    <SimilaritySourceDocument
      seed={seed}
      generation="test-generation"
      target={null}
      onReloadSections={vi.fn()}
      active
    />,
    { wrapper },
  );
  view.rerender(
    <SimilaritySourceDocument
      seed={{ kind: "record", record_key: "spells:heal" }}
      generation="test-generation"
      target={null}
      onReloadSections={vi.fn()}
      active
    />,
  );
  await screen.findByText("Healing source");
  await act(async () =>
    finish(content(detailFixture(recordKey, "Brown Mold"), "Old source")),
  );
  expect(screen.queryByText("Old source")).toBeNull();
  expect(screen.getByRole("heading", { name: "Heal" })).toBeVisible();
});

it("rejects a source from a different artifact generation", async () => {
  api.detail.mockResolvedValue(
    content(detailFixture(recordKey, "Brown Mold"), "Wrong generation source"),
  );
  render(
    <SimilaritySourceDocument
      seed={seed}
      generation="new-generation"
      target={null}
      onReloadSections={vi.fn()}
      active
    />,
    { wrapper },
  );
  expect(await screen.findByText("The reference collection changed.")).toBeVisible();
  expect(screen.queryByText("Wrong generation source")).toBeNull();
});

it("renders the existing prepared trait definition with its source identity", async () => {
  api.trait.mockResolvedValue({
    trait_key: "trait:fire",
    label: "Fire",
    artifact_generation: "test-generation",
    content: { body: { kind: "plain", text: "Fire definition" }, references: [] },
  });
  const view = render(
    <SimilaritySourceDocument
      seed={{ kind: "trait", trait_key: "trait:fire" }}
      generation="test-generation"
      target={null}
      onReloadSections={vi.fn()}
      active
    />,
    { wrapper },
  );
  expect(await screen.findByRole("heading", { name: "Fire" })).toBeVisible();
  expect(screen.getByText("Fire definition")).toBeVisible();
  const traitPart = { ...part, navigation: null, label: "Definition" };
  view.rerender(
    <SimilaritySourceDocument
      seed={{ kind: "trait", trait_key: "trait:fire" }}
      generation="test-generation"
      target={{ part: traitPart, visit: 1 }}
      onReloadSections={vi.fn()}
      active
    />,
  );
  expect(screen.getByRole("heading", { name: "Fire" })).toHaveFocus();
  expect(api.trait).toHaveBeenCalledWith(
    { trait_key: "trait:fire", expected_artifact_generation: "test-generation" },
    expect.any(AbortSignal),
  );
});

it.each(["record", "trait"] as const)(
  "recovers a changed %s source through the shared section reload instead of retrying an obsolete generation",
  async (kind) => {
    const { AtlasApiError } = await import("../../api/atlasApi");
    const error = new AtlasApiError(409, "The reference collection changed.", {
      code: "artifact_generation_changed",
      message: "The reference collection changed.",
      recoverable_action: "refresh_results",
    });
    api.detail.mockRejectedValue(error);
    api.trait.mockRejectedValue(error);
    const reload = vi.fn();
    render(
      <SimilaritySourceDocument
        seed={kind === "record" ? seed : { kind: "trait", trait_key: "trait:fire" }}
        generation="old-generation"
        target={null}
        active
        onReloadSections={reload}
      />,
      { wrapper },
    );
    fireEvent.click(await screen.findByRole("button", { name: "Reload sections" }));
    expect(reload).toHaveBeenCalledOnce();
    expect(screen.queryByRole("button", { name: "Retry" })).toBeNull();
    expect(kind === "record" ? api.detail : api.trait).toHaveBeenCalledOnce();
  },
);

it("keeps typing focus when a slow section read finishes after the user moves to another control", async () => {
  let finish!: (detail: RecordDetailView) => void;
  const root = content(detailFixture(recordKey, "Brown Mold"), "Full source document");
  const section = content(
    detailFixture(recordKey, "Brown Mold"),
    "Delayed section source",
  );
  section.selected = part.navigation!;
  api.detail.mockImplementation((_key, request) =>
    request?.fields?.length
      ? new Promise<RecordDetailView>((resolve) => (finish = resolve))
      : Promise.resolve(root),
  );
  const renderSource = (
    target: import("./SimilaritySourceDocument").SimilaritySourceTarget,
  ) => (
    <>
      <button>Reset source</button>
      <input aria-label="Match decay draft" />
      <SimilaritySourceDocument
        seed={seed}
        generation="test-generation"
        target={target}
        active
        onReloadSections={vi.fn()}
      />
    </>
  );
  const view = render(renderSource(null), { wrapper });
  await screen.findByText("Full source document");
  const opener = screen.getByRole("button", { name: "Reset source" });
  opener.focus();
  view.rerender(renderSource({ part, visit: 1, opener }));
  const input = screen.getByRole("textbox", { name: "Match decay draft" });
  input.focus();
  fireEvent.change(input, { target: { value: "0.75" } });
  await act(async () => finish(section));
  await screen.findByText("Delayed section source");
  expect(input).toHaveFocus();
  expect(input).toHaveValue("0.75");
});
