import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider, useQuery } from "@tanstack/react-query";
import { ConfigProvider } from "antd";
import { useState, type ReactNode } from "react";
import {
  AtlasApiError,
  getDefinitionDetail,
  getDefinitionFilterOptions,
  getReadiness,
} from "../../api/atlasApi";
import type { DefinitionDetailView, ReferenceHelpView } from "../../generated/atlas";
import { editorFixture, readinessFixture } from "../../test/fixtures";
import {
  AtlasConnectionError,
  clearAvailabilityFailure,
  getAvailabilityFailure,
} from "../../api/availability";
import { createAtlasQueryClient } from "../../app/queryClient";
import { AppShell } from "../../app/AppShell";
import { GlobalFeedbackProvider } from "../feedback/GlobalFeedback";
import { RouterTestContext } from "../../test/router";
import { ReferenceHelpProvider } from "../records/ReferenceHelp";
import { ReferenceHelpHost } from "../records/ReferenceHelpHost";
import { definitionDetailQueryOptions } from "../records/useDefinitionDetail";
import { retireReferenceQueries } from "../queries/referenceQueries";
import { FilterControls } from "../filters/FilterPanel";
import { DEFAULT_SEARCH_STATE } from "../filters/searchState";
import { SimilaritySeedLink } from "./SimilaritySeedLink";

vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  getDefinitionFilterOptions: vi.fn(),
  getDefinitionDetail: vi.fn(),
  getReadiness: vi.fn(),
}));

beforeEach(() => {
  vi.resetAllMocks();
  clearAvailabilityFailure();
  history.replaceState(null, "", "/search");
});

afterEach(clearAvailabilityFailure);

const help: ReferenceHelpView = {
  definition_key: "trait:agile",
  availability: "available",
  destination: { kind: "definition", key: "trait:agile" },
  related_filters: [],
};
function detail(
  generation = "current-generation",
  selectedHelp = help,
): DefinitionDetailView {
  return {
    artifact_generation: generation,
    definition_key: "trait:agile",
    label: "Agile",
    help: selectedHelp,
    content: selectedHelp.destination
      ? { body: { kind: "plain", text: "Agile rules." }, references: [] }
      : null,
    attribution: null,
    notices: [],
    references: [],
    references_availability: "available",
    references_truncated: false,
  };
}
function wrapper(client: QueryClient) {
  return function QueryWrapper({ children }: { children: ReactNode }) {
    return (
      <RouterTestContext>
        <QueryClientProvider client={client}>
          <ConfigProvider theme={{ token: { motion: false } }}>
            {children}
          </ConfigProvider>
        </QueryClientProvider>
      </RouterTestContext>
    );
  };
}
function client() {
  return new QueryClient({ defaultOptions: { queries: { retry: false } } });
}
function setup() {
  const open = vi.fn();
  const queryClient = client();
  render(
    <ReferenceHelpHost generation="previous-generation">
      <ReferenceHelpHost generation="current-generation">
        <ReferenceHelpProvider onHelp={open}>
          <SimilaritySeedLink
            seed={{ kind: "trait", trait_key: "trait:agile" }}
            title="Agile"
          />
        </ReferenceHelpProvider>
      </ReferenceHelpHost>
    </ReferenceHelpHost>,
    { wrapper: wrapper(queryClient) },
  );
  return { open, queryClient };
}

it("reads the exact retained trait through the supported definition route when complete metadata is unavailable", async () => {
  vi.mocked(getDefinitionFilterOptions).mockRejectedValue(
    new AtlasApiError(409, "Setup required", {
      code: "setup_required",
      message: "Complete definition metadata needs setup.",
    }),
  );
  vi.mocked(getDefinitionDetail).mockResolvedValue(detail());
  const { open } = setup();
  const link = await screen.findByRole("link", { name: "Agile" });
  expect(getDefinitionDetail).toHaveBeenCalledWith(
    {
      definition_key: "trait:agile",
      expected_artifact_generation: "current-generation",
    },
    expect.any(AbortSignal),
  );
  expect(getDefinitionFilterOptions).not.toHaveBeenCalled();
  expect(link).toHaveAttribute("href", "/traits/trait:agile");
  fireEvent.click(link, { ctrlKey: true });
  expect(open).not.toHaveBeenCalled();
  fireEvent.click(link);
  expect(open).toHaveBeenCalledExactlyOnceWith(help, link, "current-generation");
});

it("keeps a supplied bodyless trait seed plain without reporting a query failure", async () => {
  vi.mocked(getDefinitionDetail).mockResolvedValue(
    detail("current-generation", {
      ...help,
      availability: "label_only",
      destination: null,
    }),
  );
  const { open } = setup();
  await waitFor(() => expect(getDefinitionDetail).toHaveBeenCalledOnce());
  expect(screen.getByText("Agile")).toBeVisible();
  expect(screen.queryByRole("link", { name: "Agile" })).toBeNull();
  expect(screen.queryByRole("button", { name: /Retry|Refresh filters/ })).toBeNull();
  expect(open).not.toHaveBeenCalled();
});

it("reports a nonshared task failure and retries the same generation locally", async () => {
  vi.mocked(getDefinitionDetail)
    .mockRejectedValueOnce(
      new AtlasApiError(503, "busy diagnostic", {
        code: "service_busy",
        message: "busy diagnostic",
      }),
    )
    .mockResolvedValue(detail());
  setup();
  fireEvent.click(await screen.findByRole("button", { name: "Retry" }));
  expect(await screen.findByRole("link", { name: "Agile" })).toHaveAttribute(
    "href",
    "/traits/trait:agile",
  );
  expect(getDefinitionDetail).toHaveBeenCalledTimes(2);
  for (const [request] of vi.mocked(getDefinitionDetail).mock.calls)
    expect(request.expected_artifact_generation).toBe("current-generation");
});

it("retires cached trait help on 409 and refreshes the actual filter owner without losing its draft", async () => {
  const queryClient = client();
  const refresh = vi.fn();
  vi.mocked(getDefinitionDetail).mockImplementation(async (request) => {
    if (request.expected_artifact_generation === "next-generation")
      return detail("next-generation");
    throw new AtlasApiError(409, "Reference collection changed", {
      code: "artifact_generation_changed",
      message: "Reference collection changed",
    });
  });
  queryClient.setQueryData(
    definitionDetailQueryOptions("trait:agile", "current-generation").queryKey,
    detail(),
  );
  function FilterOwner() {
    const [generation, setGeneration] = useState("current-generation");
    const [search, setSearch] = useState({
      ...DEFAULT_SEARCH_STATE,
      query: "Unfinished text",
    });
    return (
      <ReferenceHelpHost>
        <FilterControls
          filterState={{
            search,
            setSearch,
            filterEditor: { ...editorFixture(), artifact_generation: generation },
            refreshFilterInformation: async () => {
              refresh();
              await retireReferenceQueries(queryClient);
              setGeneration("next-generation");
            },
          }}
          similarSearch={{
            seed: { kind: "trait", trait_key: "trait:agile" },
            title: "Agile",
            onClear: vi.fn(),
            onSearch: vi.fn(),
          }}
        />
        <output aria-label="Preserved query">{search.query}</output>
      </ReferenceHelpHost>
    );
  }
  render(<FilterOwner />, { wrapper: wrapper(queryClient) });
  expect(screen.getByRole("link", { name: "Agile" })).toBeVisible();
  await act(async () => {
    await queryClient.invalidateQueries({
      queryKey: definitionDetailQueryOptions("trait:agile", "current-generation")
        .queryKey,
    });
  });
  const button = await screen.findByRole("button", { name: "Refresh filters" });
  expect(screen.queryByRole("link", { name: "Agile" })).toBeNull();
  expect(getDefinitionDetail).toHaveBeenLastCalledWith(
    {
      definition_key: "trait:agile",
      expected_artifact_generation: "current-generation",
    },
    expect.any(AbortSignal),
  );
  fireEvent.click(button);
  expect(await screen.findByRole("link", { name: "Agile" })).toBeVisible();
  expect(refresh).toHaveBeenCalledOnce();
  expect(screen.getByLabelText("Preserved query")).toHaveTextContent("Unfinished text");
  expect(getDefinitionDetail).toHaveBeenLastCalledWith(
    { definition_key: "trait:agile", expected_artifact_generation: "next-generation" },
    expect.any(AbortSignal),
  );
});

it.each(["scoped", "absent", "disabled"] as const)(
  "keeps record seeds in their %s reading scope with native modified activation",
  (scope) => {
    const scoped = vi.fn();
    const global = vi.fn();
    const queryClient = client();
    render(
      <ReferenceHelpHost>
        <ReferenceHelpProvider onReference={global}>
          <ReferenceHelpProvider
            onHelp={vi.fn()}
            onReference={scope === "absent" ? undefined : scoped}
          >
            <SimilaritySeedLink
              seed={{ kind: "record", record_key: "actors:ghoul" }}
              title="Ghoul"
              disabled={scope === "disabled"}
            />
          </ReferenceHelpProvider>
        </ReferenceHelpProvider>
      </ReferenceHelpHost>,
      { wrapper: wrapper(queryClient) },
    );
    const link = screen.getByText("Ghoul");
    if (scope !== "disabled")
      expect(link).toHaveAttribute("href", "/records/actors:ghoul");
    fireEvent.click(link, { ctrlKey: true });
    expect(scoped).not.toHaveBeenCalled();
    fireEvent.click(link);
    if (scope === "scoped")
      expect(scoped).toHaveBeenCalledExactlyOnceWith("actors:ghoul", link);
    else expect(scoped).not.toHaveBeenCalled();
    expect(global).not.toHaveBeenCalled();
    expect(screen.queryByRole("dialog")).toBeNull();
  },
);

it("reads a standalone trait criterion at the current generation without a parent pin", async () => {
  vi.mocked(getDefinitionDetail).mockResolvedValue(detail());
  render(
    <SimilaritySeedLink
      seed={{ kind: "trait", trait_key: "trait:agile" }}
      title="Agile"
    />,
    { wrapper: wrapper(client()) },
  );
  expect(await screen.findByRole("link", { name: "Agile" })).toBeVisible();
  expect(getDefinitionDetail).toHaveBeenCalledWith(
    { definition_key: "trait:agile", expected_artifact_generation: null },
    expect.any(AbortSignal),
  );
});

it("retains an isolated explicit reading handler instead of creating another seed controller", () => {
  const scoped = vi.fn();
  render(
    <ReferenceHelpProvider onReference={scoped}>
      <SimilaritySeedLink
        seed={{ kind: "record", record_key: "actors:ghoul" }}
        title="Ghoul"
      />
    </ReferenceHelpProvider>,
    { wrapper: wrapper(client()) },
  );
  const link = screen.getByRole("link", { name: "Ghoul" });
  fireEvent.click(link);
  expect(scoped).toHaveBeenCalledExactlyOnceWith("actors:ghoul", link);
  expect(screen.queryByRole("dialog")).toBeNull();
});

it("retries an unpinned standalone generation failure without changing the request contract", async () => {
  vi.mocked(getDefinitionDetail)
    .mockRejectedValueOnce(
      new AtlasApiError(409, "Reference collection changed", {
        code: "artifact_generation_changed",
        message: "Reference collection changed",
      }),
    )
    .mockResolvedValue(detail());
  render(
    <SimilaritySeedLink
      seed={{ kind: "trait", trait_key: "trait:agile" }}
      title="Agile"
    />,
    { wrapper: wrapper(client()) },
  );
  fireEvent.click(await screen.findByRole("button", { name: "Retry" }));
  expect(await screen.findByRole("link", { name: "Agile" })).toBeVisible();
  for (const [request] of vi.mocked(getDefinitionDetail).mock.calls)
    expect(request.expected_artifact_generation).toBeNull();
});

it("keeps a disabled trait criterion plain without requesting or showing recovery controls", () => {
  render(
    <SimilaritySeedLink
      seed={{ kind: "trait", trait_key: "trait:agile" }}
      title="Agile"
      disabled
    />,
    { wrapper: wrapper(client()) },
  );
  expect(screen.getByText("Agile")).toBeVisible();
  expect(screen.queryByRole("link", { name: "Agile" })).toBeNull();
  expect(screen.queryByRole("button", { name: /Retry|Refresh filters/ })).toBeNull();
  expect(getDefinitionDetail).not.toHaveBeenCalled();
});

it("opens retained trait reading in the shared Drawer and reuses the same pinned detail query", async () => {
  vi.mocked(getDefinitionDetail).mockResolvedValue(detail());
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, staleTime: 20_000 } },
  });
  render(
    <ReferenceHelpHost generation="current-generation">
      <SimilaritySeedLink
        seed={{ kind: "trait", trait_key: "trait:agile" }}
        title="Agile"
      />
    </ReferenceHelpHost>,
    { wrapper: wrapper(queryClient) },
  );
  fireEvent.click(await screen.findByRole("link", { name: "Agile" }));
  expect(await screen.findByRole("dialog", { name: "Agile" })).toBeVisible();
  expect(screen.getAllByRole("dialog")).toHaveLength(1);
  expect(getDefinitionDetail).toHaveBeenCalledOnce();
});

it("lets the single shell Retry recover an active failed definition read through the production query client", async () => {
  const error = new AtlasConnectionError(false, new Error("offline"));
  vi.mocked(getDefinitionDetail)
    .mockRejectedValueOnce(error)
    .mockRejectedValueOnce(error)
    .mockResolvedValue(detail());
  vi.mocked(getReadiness).mockResolvedValue(readinessFixture());
  const queryClient = createAtlasQueryClient();
  // Keep the production retry policy; only remove the backoff delay in this test.
  queryClient.setQueryDefaults(["definition-detail"], { retryDelay: 0 });
  function Shell() {
    const readiness = useQuery({ queryKey: ["readiness"], queryFn: getReadiness });
    return (
      <GlobalFeedbackProvider>
        <AppShell
          activeView="search"
          colorScheme="light"
          resolvedColorScheme="light"
          onColorSchemeChange={vi.fn()}
          readiness={readiness.data}
          readinessError={readiness.error}
          checkingReadiness={readiness.isFetching}
          onCheckReadiness={() => void readiness.refetch()}
        >
          <input aria-label="Filter draft" defaultValue="Unfinished filter text" />
          <ReferenceHelpHost generation="current-generation">
            <SimilaritySeedLink
              seed={{ kind: "trait", trait_key: "trait:agile" }}
              title="Agile"
            />
          </ReferenceHelpHost>
        </AppShell>
      </GlobalFeedbackProvider>
    );
  }
  render(<Shell />, { wrapper: wrapper(queryClient) });
  const draft = screen.getByRole("textbox", { name: "Filter draft" });
  expect(await screen.findByRole("alert")).toHaveTextContent(
    "Unable to connect to Atlas",
  );
  expect(getAvailabilityFailure()).toBe(error);
  expect(getDefinitionDetail).toHaveBeenCalledTimes(2);
  expect(screen.getAllByRole("button", { name: "Retry" })).toHaveLength(1);
  expect(screen.queryByText("Definition unavailable")).toBeNull();
  expect(screen.queryByRole("link", { name: "Agile" })).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Retry" }));
  expect(await screen.findByRole("link", { name: "Agile" })).toHaveAttribute(
    "href",
    "/traits/trait:agile",
  );
  expect(getReadiness).toHaveBeenCalledTimes(2);
  expect(getDefinitionDetail).toHaveBeenCalledTimes(3);
  expect(getAvailabilityFailure()).toBeNull();
  expect(screen.queryByRole("alert")).toBeNull();
  expect(screen.getByRole("textbox", { name: "Filter draft" })).toBe(draft);
  expect(draft).toHaveValue("Unfinished filter text");
  for (const [request] of vi.mocked(getDefinitionDetail).mock.calls)
    expect(request.expected_artifact_generation).toBe("current-generation");
});
