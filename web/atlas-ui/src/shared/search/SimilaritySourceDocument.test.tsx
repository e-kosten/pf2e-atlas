import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { ConfigProvider } from "antd";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type {
  SimilarityComponentView,
  SimilarityPartsView,
  RecordDetailView,
  RecordDetailRequest,
  RecordNavigationView,
} from "../../generated/atlas";
import { detailFixture, spellFixture } from "../../test/fixtures";
import { SimilaritySeedLink } from "./SimilaritySeedLink";
import { parseSearchParams, selectedNavigation } from "../../app/urlState";
import { atlasRouter } from "../../app/router";
import { RouterTestContext } from "../../test/router";
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
const part = {
  kind: "section" as const,
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
} satisfies SimilarityComponentView;
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
      <ConfigProvider theme={{ token: { motion: false } }}>
        <RouterTestContext>{children}</RouterTestContext>
      </ConfigProvider>
    </QueryClientProvider>
  );
}
beforeEach(() => {
  vi.clearAllMocks();
  history.replaceState(null, "", "/search");
});

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
  expect(
    await screen.findByText(
      "The reference collection changed. Refresh before continuing.",
    ),
  ).toBeVisible();
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

it("emphasizes a source only with the matching seed and generation inventory", async () => {
  api.detail.mockResolvedValue(
    content(detailFixture(recordKey, "Brown Mold"), "Source prose"),
  );
  const inventory: SimilarityPartsView = {
    artifact_generation: "test-generation",
    search_profile: "bge-base",
    seed,
    role_recipe: "roles/v1",
    supports_corresponding_parts: true,
    core: {
      family: "hazard",
      selectors: [part.selector],
      representation: "identity_and_prose",
      fallback_reason: null,
      recipe: "core-sections/v1",
    },
    parts: [part],
  };
  const source = (currentInventory: SimilarityPartsView | null) => (
    <SimilaritySourceDocument
      seed={seed}
      generation="test-generation"
      target={null}
      active
      onReloadSections={vi.fn()}
      selection={
        currentInventory ? { inventory: currentInventory, selected: null } : null
      }
    />
  );
  const view = render(source(inventory), { wrapper });
  await screen.findByText("Source prose");
  expect(view.container.querySelector(".comparison-source")).not.toBeNull();
  view.rerender(source({ ...inventory, artifact_generation: "old-generation" }));
  expect(view.container.querySelector(".comparison-source")).toBeNull();
  view.rerender(
    source({ ...inventory, seed: { kind: "record", record_key: "spells:heal" } }),
  );
  expect(view.container.querySelector(".comparison-source")).toBeNull();
  view.rerender(source(null));
  expect(view.container.querySelector(".comparison-source")).toBeNull();
});

it.each([
  { caller: "seed link", embedded: false },
  { caller: "seed link", embedded: true },
  { caller: "trait source", embedded: false },
  { caller: "trait source", embedded: true },
] as const)(
  "preserves requested spell rank and exact identity from $caller (embedded=$embedded)",
  async ({ caller, embedded }) => {
    const spellKey = "spells:fireball";
    const target: RecordNavigationView = embedded
      ? {
          ...rootNavigation(spellKey),
          embedded_path: [
            { collection: "/items", identity: { Stable: "nested-spell" } },
          ],
          field: "/system/description/value",
          source_fingerprint: "spell-snapshot",
        }
      : rootNavigation(spellKey);
    // Returning from an embedded spell exercises the Drawer's landing handoff.
    const landing: RecordNavigationView | undefined =
      caller === "seed link" && !embedded
        ? {
            ...rootNavigation(spellKey),
            embedded_path: [
              { collection: "/items", identity: { Stable: "nested-spell" } },
            ],
            source_fingerprint: "spell-snapshot",
          }
        : undefined;
    const reference = {
      ...detailFixture("rules:host", "Host", spellKey).relationships[0],
      target: landing ?? target,
    };
    api.detail.mockImplementation((key: string, request?: RecordDetailRequest) => {
      const detail = detailFixture(key, "Fireball");
      const rank = request?.spell_rank ?? 3;
      detail.selected = {
        ...rootNavigation(key),
        embedded_path: request?.embedded_path ?? [],
        field: request?.fields?.[0] ?? null,
        passage: request?.passage ?? null,
        source_fingerprint: request?.source_fingerprint ?? null,
        spell_rank: rank,
      };
      detail.presentation.body = { kind: "spell", value: spellFixture() };
      detail.presentation.body.value.rank = {
        ...detail.presentation.body.value.rank,
        value: rank,
      };
      detail.presentation.content = [
        {
          locator: {
            record: key,
            embedded_path: [],
            field: "/system/description/value",
          },
          role: "description",
          source_fingerprint: "spell-snapshot",
          body: {
            kind: "html",
            html: '<p><span data-atlas-reference="0">Embedded spell</span></p>',
            controls: [],
            inclusions: [],
          },
        },
      ];
      detail.relationships = [
        {
          ...reference,
          source: detail.presentation.content[0].locator,
        },
      ];
      return Promise.resolve(detail);
    });
    api.trait.mockResolvedValue({
      trait_key: "trait:fire",
      label: "Fire",
      artifact_generation: "test-generation",
      content: {
        body: {
          kind: "html",
          html: '<p><span data-atlas-reference="0">Spell reference</span></p>',
          controls: [],
          inclusions: [],
        },
        references: [reference],
      },
    });
    if (caller === "seed link") {
      render(
        <SimilaritySeedLink
          seed={{ kind: "record", record_key: spellKey }}
          title="Source seed"
        />,
        { wrapper },
      );
      fireEvent.click(screen.getByRole("link", { name: "Source seed" }));
      await screen.findByRole("button", { name: /Change rank/ });
      if (embedded || landing) {
        fireEvent.click(screen.getByRole("link", { name: "Embedded spell" }));
        await waitFor(() =>
          expect(api.detail).toHaveBeenLastCalledWith(
            spellKey,
            expect.objectContaining({
              embedded_path: (landing ?? target).embedded_path,
            }),
            expect.any(AbortSignal),
          ),
        );
        if (landing) {
          fireEvent.click(
            await screen.findByRole("link", { name: "Return to Fireball" }),
          );
        }
      }
    } else {
      render(
        <SimilaritySourceDocument
          seed={{ kind: "trait", trait_key: "trait:fire" }}
          generation="test-generation"
          target={null}
          active
          onReloadSections={vi.fn()}
        />,
        { wrapper },
      );
      fireEvent.click(await screen.findByRole("link", { name: "Spell reference" }));
    }
    await screen.findByRole("link", { name: "Open reference full page" });
    const openFullPage = () =>
      screen.getByRole("link", { name: "Open reference full page" });
    await waitFor(() => {
      const search = new URL(openFullPage().getAttribute("href")!, "http://localhost")
        .search;
      expect(new URLSearchParams(search).has("rank")).toBe(false);
      expect(
        selectedNavigation(parseSearchParams(search), spellKey) ??
          rootNavigation(spellKey),
      ).toEqual(target);
      expect(selectedNavigation(parseSearchParams(search), spellKey, "focus")).toEqual(
        landing,
      );
    });
    fireEvent.click(screen.getByRole("button", { name: /Change rank/ }));
    fireEvent.click(await screen.findByRole("menuitem", { name: "Rank 5" }));
    await waitFor(() =>
      expect(api.detail).toHaveBeenLastCalledWith(
        spellKey,
        expect.objectContaining({ spell_rank: 5, embedded_path: target.embedded_path }),
        expect.any(AbortSignal),
      ),
    );
    await waitFor(() =>
      expect(
        selectedNavigation(
          parseSearchParams(
            new URL(openFullPage().getAttribute("href")!, "http://localhost").search,
          ),
          spellKey,
        ),
      ).toEqual({ ...target, spell_rank: 5 }),
    );
    fireEvent.click(openFullPage());
    await waitFor(() => {
      expect(atlasRouter.latestLocation.pathname).toBe("/records/" + spellKey);
      expect(selectedNavigation(atlasRouter.latestLocation.search, spellKey)).toEqual({
        ...target,
        spell_rank: 5,
      });
      expect(
        selectedNavigation(atlasRouter.latestLocation.search, spellKey, "focus"),
      ).toEqual(landing);
    });
  },
);

it("keeps unknown trait failures contextual and retries without exposing diagnostics", async () => {
  api.trait
    .mockRejectedValueOnce(new Error("private-path-and-decoder-details"))
    .mockResolvedValue({
      trait_key: "trait:fire",
      label: "Fire",
      artifact_generation: "test-generation",
      content: {
        body: { kind: "plain", text: "Recovered definition" },
        references: [],
      },
    });
  render(
    <SimilaritySourceDocument
      seed={{ kind: "trait", trait_key: "trait:fire" }}
      generation="test-generation"
      target={null}
      active
      onReloadSections={vi.fn()}
    />,
    { wrapper },
  );
  expect(await screen.findByText("Unable to load this trait")).toBeVisible();
  expect(screen.queryByText("private-path-and-decoder-details")).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Retry" }));
  expect(await screen.findByText("Recovered definition")).toBeVisible();
  expect(screen.queryByText("Unable to load this trait")).toBeNull();
});
