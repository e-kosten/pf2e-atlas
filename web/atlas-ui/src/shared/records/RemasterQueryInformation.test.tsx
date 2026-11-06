import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { AtlasApiError, getRemasterRelationships } from "../../api/atlasApi";
import { remasterInformationFixture } from "../../test/remasterFixtures";
import { RemasterQueryInformation } from "./RemasterQueryInformation";
import { SearchRemasterContext } from "../../features/search/SearchRemasterContext";
import { ReferencePreviewContext } from "./ReferencePreviewContext";
vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  getRemasterRelationships: vi.fn(),
}));
const read = vi.mocked(getRemasterRelationships);
const request = {
  subject: { kind: "term" as const, term: "Removed term" },
  expected_artifact_generation: "generation-1",
  limit: 20,
};
function setup() {
  const refresh = vi.fn(),
    open = vi.fn();
  render(
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <input aria-label="Preserved draft" defaultValue="Unsaved filter draft" />
      <p>0 matching records</p>
      <RemasterQueryInformation
        request={request}
        onReference={open}
        onReload={refresh}
        quietEmpty
      />
    </QueryClientProvider>,
  );
  return { refresh, open };
}
beforeEach(() => {
  read.mockReset();
});
it("shows exact committed-term source context outside counts and keeps the draft", async () => {
  read.mockResolvedValue({
    artifact_generation: "generation-1",
    subject: {
      kind: "term",
      query: "Removed term",
      normalized_term: "removed term",
      scope_label: "Source statements about this term; independent of record filters.",
    },
    relationships: { kind: "content", content: remasterInformationFixture() },
  });
  const { open } = setup();
  const link = (await screen.findAllByRole("link", { name: "New name" }))[0];
  expect(read).toHaveBeenCalledWith(request, expect.any(AbortSignal));
  expect(screen.getByText(/independent of record filters/)).toBeVisible();
  expect(screen.getByText("0 matching records")).toBeVisible();
  expect(screen.getByRole("textbox", { name: "Preserved draft" })).toHaveValue(
    "Unsaved filter draft",
  );
  fireEvent.click(link);
  expect(open).toHaveBeenCalledOnce();
});
it("keeps errors local with deliberate generation recovery", async () => {
  read.mockRejectedValue(
    new AtlasApiError(409, "Changed", {
      code: "artifact_generation_changed",
      message: "Changed",
      recoverable_action: "refresh_results",
    }),
  );
  const { refresh } = setup();
  fireEvent.click(await screen.findByRole("button", { name: "Refresh results" }));
  expect(refresh).toHaveBeenCalledOnce();
  expect(read).toHaveBeenCalledTimes(1);
  expect(screen.getByText("0 matching records")).toBeVisible();
});
it("does not request optional context for overlong queries", async () => {
  render(
    <QueryClientProvider client={new QueryClient()}>
      <ReferencePreviewContext value={{ open: vi.fn(), openTrait: vi.fn() }}>
        <SearchRemasterContext
          query={"x".repeat(257)}
          generation="generation-1"
          onReload={vi.fn()}
        />
      </ReferencePreviewContext>
    </QueryClientProvider>,
  );
  await waitFor(() => expect(read).not.toHaveBeenCalled());
});

it("keeps unknown terms quiet without claiming they are records", async () => {
  read.mockResolvedValue({
    artifact_generation: "generation-1",
    subject: {
      kind: "term",
      query: "Removed term",
      normalized_term: "removed term",
      scope_label: "Independent source context",
    },
    relationships: { kind: "empty" },
  });
  setup();
  await waitFor(() => expect(read).toHaveBeenCalledOnce());
  await waitFor(() =>
    expect(screen.queryByText("Checking remaster information…")).toBeNull(),
  );
  expect(
    screen.queryByRole("region", {
      name: "Source statements about this term or entry",
    }),
  ).toBeNull();
  expect(screen.getByText("0 matching records")).toBeVisible();
});
