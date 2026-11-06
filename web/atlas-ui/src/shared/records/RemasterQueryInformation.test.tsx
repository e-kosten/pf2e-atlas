import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { AtlasApiError, getRemasterRelationships } from "../../api/atlasApi";
import { rootNavigation } from "./useReferenceReading";
import { remasterInformationFixture } from "../../test/remasterFixtures";
import { RemasterQueryInformation } from "./RemasterQueryInformation";
vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  getRemasterRelationships: vi.fn(),
}));
const read = vi.mocked(getRemasterRelationships);
const request = {
  subject: { kind: "record" as const, selection: rootNavigation("rules:absent") },
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
      <p>Saved key: rules:absent</p>
      <RemasterQueryInformation
        request={request}
        onReference={open}
        onReload={refresh}
      />
    </QueryClientProvider>,
  );
  return { refresh, open };
}
beforeEach(() => {
  read.mockReset();
});
it("reads explicit saved identity without changing its notes", async () => {
  read.mockResolvedValue({
    artifact_generation: "generation-1",
    subject: {
      kind: "record",
      selection: rootNavigation("rules:absent"),
      title: null,
      available: false,
    },
    relationships: { kind: "content", content: remasterInformationFixture() },
  });
  const { open } = setup();
  const link = (await screen.findAllByRole("link", { name: "New name" }))[0];
  expect(read).toHaveBeenCalledWith(request, expect.any(AbortSignal));
  expect(screen.getByText("Saved key: rules:absent")).toBeVisible();
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
  fireEvent.click(await screen.findByRole("button", { name: "Reload information" }));
  expect(refresh).toHaveBeenCalledOnce();
  expect(read).toHaveBeenCalledTimes(1);
  expect(screen.getByText("Saved key: rules:absent")).toBeVisible();
});
it("explains empty information for the retained saved identity", async () => {
  read.mockResolvedValue({
    artifact_generation: "generation-1",
    subject: {
      kind: "record",
      selection: rootNavigation("rules:absent"),
      title: null,
      available: false,
    },
    relationships: { kind: "empty" },
  });
  setup();
  await waitFor(() => expect(read).toHaveBeenCalledOnce());
  await waitFor(() =>
    expect(screen.queryByText("Checking remaster information…")).toBeNull(),
  );
  expect(
    screen.getByText("No remaster information found for this entry."),
  ).toBeVisible();
  expect(screen.getByText("Saved key: rules:absent")).toBeVisible();
});
