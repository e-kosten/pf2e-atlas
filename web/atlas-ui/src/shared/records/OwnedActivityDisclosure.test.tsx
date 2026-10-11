import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { OwnedActivityDisclosure } from "./OwnedActivityDisclosure";
import { activityFixture, detailFixture, knownFact } from "../../test/fixtures";
import { atlasRoutePath } from "../../app/routes";

const api = vi.hoisted(() => ({ getRecordDetail: vi.fn() }));
vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  getRecordDetail: api.getRecordDetail,
}));

it("loads only the expanded exact owned ability and keeps parent context and a native full-page link", async () => {
  api.getRecordDetail.mockReset();
  const seize = {
    ...activityFixture("Seize", "hazards:nightmare"),
    kind: "ability" as const,
    action_cost: knownFact({ label: "Reaction", glyph: "reaction" as const }),
  };
  const other = {
    ...seize,
    title: "Other ability",
    navigation: {
      ...seize.navigation,
      owners: [{ collection: "items", identity: { Stable: "other" } }],
    },
  };
  const detail = detailFixture("hazards:nightmare", "Seize");
  detail.selected = seize.navigation;
  detail.presentation.body = { kind: "activity", value: seize };
  detail.presentation.content = [
    {
      locator: {
        record: "hazards:nightmare",
        owners: seize.navigation.owners,
        field: "/system/description/value",
      },
      role: "description",
      source_fingerprint: "snapshot",
      body: {
        kind: "html",
        html: "<p><strong>Trigger</strong> A creature approaches.</p><p><strong>Effect</strong> The nightmare seizes it.</p>",
        controls: [],
      },
    },
  ];
  api.getRecordDetail.mockResolvedValue(detail);
  const onReference = vi.fn();
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false, staleTime: Infinity } },
  });
  render(
    <QueryClientProvider client={client}>
      <h2>Waking Nightmare</h2>
      <OwnedActivityDisclosure activity={seize} onReference={onReference} />
      <OwnedActivityDisclosure activity={other} onReference={onReference} />
    </QueryClientProvider>,
  );
  expect(api.getRecordDetail).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button", { name: "Seize Reaction" }));
  await waitFor(() => expect(screen.getByText("Trigger")).toBeVisible());
  expect(screen.getByText("Effect")).toBeVisible();
  expect(screen.queryByRole("heading", { name: "Seize" })).toBeNull();
  expect(screen.getAllByText("Reaction")).toHaveLength(2); // One label for each collapsed ability.
  expect(
    document.querySelector(".actor-ability__detail .detail-panel--inline"),
  ).not.toBeNull();
  expect(screen.getByRole("heading", { name: "Waking Nightmare" })).toBeVisible();
  expect(screen.getByRole("link", { name: "Open Seize full page" })).toHaveAttribute(
    "href",
    atlasRoutePath({
      kind: "record",
      recordKey: seize.navigation.record_key,
      selection: seize.navigation,
    }),
  );
  expect(onReference).not.toHaveBeenCalled();
  expect(api.getRecordDetail).toHaveBeenCalledExactlyOnceWith("hazards:nightmare", {
    ...seize.navigation,
    fields: [],
  });
  fireEvent.click(screen.getByRole("button", { name: "Seize Reaction" }));
  fireEvent.click(screen.getByRole("button", { name: "Seize Reaction" }));
  await waitFor(() => expect(screen.getByText("Trigger")).toBeVisible());
  expect(api.getRecordDetail).toHaveBeenCalledTimes(1);
});
