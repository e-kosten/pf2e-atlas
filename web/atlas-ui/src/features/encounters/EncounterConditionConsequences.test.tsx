import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ConfigProvider } from "antd";
import { previewEncounterDrainedConsequence } from "../../api/atlasApi";
import { EncounterConditionConsequences } from "./EncounterConditionConsequences";

vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  previewEncounterDrainedConsequence: vi.fn(),
}));

it("retains the Drained HP preview through edits and failed refreshes without authorizing Apply", async () => {
  const initial = {
    current_before: 10n,
    current_after: 5n,
    maximum_before: 20n,
    maximum_after: 15n,
  };
  vi.mocked(previewEncounterDrainedConsequence).mockResolvedValue(initial);
  const onResolve = vi.fn();
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <QueryClientProvider client={client}>
        <EncounterConditionConsequences
          encounterRef="encounter"
          encounterRevision={1n}
          row={{
            condition_ref: "drained",
            consequence_key: "consequence",
            participant_key: "actor",
            magnitude_increase: 1n,
          }}
          level={5n}
          disabled={false}
          pending={false}
          onResolve={onResolve}
        />
      </QueryClientProvider>
    </ConfigProvider>,
  );
  fireEvent.click(screen.getByRole("button", { name: "HP loss" }));
  const previous = await screen.findByText("10 → 5");
  const input = screen.getByRole("spinbutton", { name: "Level for Drained" });
  let reject!: (error: Error) => void;
  vi.mocked(previewEncounterDrainedConsequence).mockImplementationOnce(
    () =>
      new Promise((_resolve, fail) => {
        reject = fail;
      }),
  );
  act(() => input.focus());
  fireEvent.change(input, { target: { value: "6" } });
  await waitFor(() =>
    expect(previewEncounterDrainedConsequence).toHaveBeenCalledTimes(2),
  );
  expect(previous).toBeInTheDocument();
  expect(input).toHaveFocus();
  expect(screen.getByRole("button", { name: "Apply" })).toBeDisabled();
  await act(async () => reject(new Error("Unavailable")));
  await screen.findByText(/Previous preview shown/);
  expect(previous).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "Apply" })).toBeDisabled();
  vi.mocked(previewEncounterDrainedConsequence).mockResolvedValue({
    ...initial,
    current_after: 4n,
  });
  fireEvent.click(screen.getByRole("button", { name: "Retry" }));
  await screen.findByText("10 → 4");
  expect(screen.getByRole("button", { name: "Apply" })).toBeEnabled();
  expect(onResolve).not.toHaveBeenCalled();
});
