import { fireEvent, render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider, useQuery } from "@tanstack/react-query";
import { ConfigProvider } from "antd";
import { type ReactNode } from "react";
import { getDefinitionDetail, getReadiness } from "../../api/atlasApi";
import type { DefinitionDetailView, ReferenceHelpView } from "../../generated/atlas";
import { readinessFixture } from "../../test/fixtures";
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
import { SimilaritySeedLink } from "./SimilaritySeedLink";

vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
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

  destination: { kind: "definition", key: "trait:agile" },
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

    notices: [],

    related_filters: [],
    publications: [],
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
            help={help}
          />
        </ReferenceHelpProvider>
      </ReferenceHelpHost>
    </ReferenceHelpHost>,
    { wrapper: wrapper(queryClient) },
  );
  return { open, queryClient };
}

it("uses supplied trait capability without a separate metadata request", () => {
  const { open } = setup();
  const link = screen.getByRole("link", { name: "Agile" });
  expect(link).toHaveAttribute("href", "/traits/trait:agile");
  expect(getDefinitionDetail).not.toHaveBeenCalled();
  fireEvent.click(link, { ctrlKey: true });
  expect(open).not.toHaveBeenCalled();
  fireEvent.click(link);
  expect(open).toHaveBeenCalledWith(help, link, "current-generation");
});
it("keeps an unavailable or disabled criterion plain without a metadata request", () => {
  render(
    <>
      <SimilaritySeedLink
        seed={{ kind: "trait", trait_key: "trait:missing" }}
        title="Missing"
      />
      <SimilaritySeedLink
        seed={{ kind: "trait", trait_key: "trait:agile" }}
        title="Agile"
        help={help}
        disabled
      />
    </>,
    { wrapper: wrapper(client()) },
  );
  expect(document.body).toHaveTextContent("MissingAgile");
  expect(screen.queryByRole("link")).toBeNull();
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
        help={help}
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
              help={help}
            />
          </ReferenceHelpHost>
        </AppShell>
      </GlobalFeedbackProvider>
    );
  }
  render(<Shell />, { wrapper: wrapper(queryClient) });
  const draft = screen.getByRole("textbox", { name: "Filter draft" });
  fireEvent.click(screen.getByRole("link", { name: "Agile" }));
  expect(await screen.findByRole("alert")).toHaveTextContent(
    "Unable to connect to Atlas",
  );
  expect(getAvailabilityFailure()).toBe(error);
  expect(getDefinitionDetail).toHaveBeenCalledTimes(2);
  expect(screen.getAllByRole("button", { name: "Retry" })).toHaveLength(1);
  expect(screen.queryByText("Definition unavailable")).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Retry" }));
  expect(await screen.findByRole("dialog", { name: "Agile" })).toBeVisible();
  expect(getReadiness).toHaveBeenCalledTimes(2);
  expect(getDefinitionDetail).toHaveBeenCalledTimes(3);
  expect(getAvailabilityFailure()).toBeNull();
  expect(screen.queryByRole("alert")).toBeNull();
  expect(screen.getByRole("textbox", { name: "Filter draft" })).toBe(draft);
  expect(draft).toHaveValue("Unfinished filter text");
  for (const [request] of vi.mocked(getDefinitionDetail).mock.calls)
    expect(request.expected_artifact_generation).toBe("current-generation");
});
