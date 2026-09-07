import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen } from "@testing-library/react";
import { getRecordDetail } from "../../api/atlasApi";
import { recordDetailFixture } from "../../test/recordFixtures";
import { ReferenceFilterChip } from "./ReferenceFilterChip";
vi.mock("../../api/atlasApi", () => ({ getRecordDetail: vi.fn() }));

it("uses the exact server record label and offers an accessible removal", async () => {
  const detail = recordDetailFixture({ recordKey: "spells:seed", title: "Fireball" });
  vi.mocked(getRecordDetail).mockResolvedValue(detail);
  const remove = vi.fn();
  render(
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <ReferenceFilterChip
        relationship={{ direction: "incoming", record_key: "spells:seed" }}
        onRemove={remove}
      />
    </QueryClientProvider>,
  );
  expect(await screen.findByText("References: Fireball")).toBeInTheDocument();
  expect(getRecordDetail).toHaveBeenCalledWith(
    "spells:seed",
    undefined,
    expect.any(AbortSignal),
  );
  expect(screen.queryByText(/spells:seed/)).not.toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "Remove reference filter" }));
  expect(remove).toHaveBeenCalledOnce();
});

it("does not attribute a mismatched response label or expose a missing raw key", async () => {
  vi.mocked(getRecordDetail).mockResolvedValue(
    recordDetailFixture({ recordKey: "spells:other", title: "Wrong label" }),
  );
  render(
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <ReferenceFilterChip
        relationship={{ direction: "outgoing", record_key: "spells:missing" }}
        onRemove={vi.fn()}
      />
    </QueryClientProvider>,
  );
  expect(
    await screen.findByText("Referenced by: Record not found"),
  ).toBeInTheDocument();
  expect(screen.queryByText(/Wrong label|spells:missing/)).not.toBeInTheDocument();
});
