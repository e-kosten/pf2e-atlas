import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { App, ConfigProvider } from "antd";
import { ReferencePreviewProvider } from "../../shared/records/ReferencePreviewProvider";
import { RouterTestContext } from "../../test/router";
import { detailFixture } from "../../test/fixtures";
import { getRecordDetail, getRemasterRelationships } from "../../api/atlasApi";
import { remasterInformationFixture } from "../../test/remasterFixtures";
import { ReferencePreviewContext } from "../../shared/records/ReferencePreviewContext";
import { UnavailableRecordRemasterAction } from "./UnavailableRecordRemasterAction";
import { rootNavigation } from "../../shared/records/useReferenceReading";
vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  getRemasterRelationships: vi.fn(),
  getRecordDetail: vi.fn(),
}));
it("reads an unavailable exact saved key on demand and opens its successor without replacing saved identity or notes", async () => {
  const read = vi.mocked(getRemasterRelationships);
  read.mockResolvedValue({
    artifact_generation: "current",
    subject: {
      kind: "record",
      selection: rootNavigation("rules:absent"),
      title: null,
      available: false,
    },
    relationships: { kind: "content", content: remasterInformationFixture() },
  });
  const open = vi.fn();
  render(
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <ConfigProvider theme={{ token: { motion: false } }}>
        <App>
          <ReferencePreviewContext value={{ open, openTrait: vi.fn() }}>
            <span>Saved key: rules:absent</span>
            <textarea aria-label="Saved note" defaultValue="Keep this chosen record" />
            <UnavailableRecordRemasterAction
              recordKey="rules:absent"
              title="Retained title"
            />
          </ReferencePreviewContext>
        </App>
      </ConfigProvider>
    </QueryClientProvider>,
  );
  expect(read).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button", { name: "Remaster information" }));
  const successor = await screen.findByRole("link", { name: "New name" });
  expect(read).toHaveBeenCalledExactlyOnceWith(
    {
      subject: { kind: "record", selection: rootNavigation("rules:absent") },
      limit: 20,
    },
    expect.any(AbortSignal),
  );
  fireEvent.click(successor);
  expect(open).toHaveBeenCalledWith(
    "rules:new",
    successor,
    rootNavigation("rules:new"),
  );
  expect(screen.getByText("Saved key: rules:absent")).toBeInTheDocument();
  expect(screen.getByRole("textbox", { name: "Saved note", hidden: true })).toHaveValue(
    "Keep this chosen record",
  );
});

it.each(["Escape", "Close"])(
  "returns focus to the saved-row opener after %s without fetching while closed",
  async (method) => {
    const read = vi.mocked(getRemasterRelationships);
    read.mockReset().mockResolvedValue({
      artifact_generation: "current",
      subject: {
        kind: "record",
        selection: rootNavigation("rules:absent"),
        title: null,
        available: false,
      },
      relationships: { kind: "content", content: remasterInformationFixture() },
    });
    render(
      <QueryClientProvider
        client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
      >
        <ConfigProvider theme={{ token: { motion: false } }}>
          <App>
            <ReferencePreviewContext value={{ open: vi.fn(), openTrait: vi.fn() }}>
              <UnavailableRecordRemasterAction
                recordKey="rules:absent"
                title="Retained title"
              />
            </ReferencePreviewContext>
          </App>
        </ConfigProvider>
      </QueryClientProvider>,
    );
    const opener = screen.getByRole("button", { name: "Remaster information" });
    opener.focus();
    expect(read).not.toHaveBeenCalled();
    fireEvent.click(opener);
    const dialog = await screen.findByRole("dialog", {
      name: "Remaster information · Retained title",
    });
    await screen.findByRole("link", { name: "New name" });
    if (method === "Escape") fireEvent.keyDown(dialog, { key: "Escape", keyCode: 27 });
    else fireEvent.click(within(dialog).getByRole("button", { name: "Close" }));
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
    await waitFor(() => expect(opener).toHaveFocus());
    expect(read).toHaveBeenCalledTimes(1);
  },
);

it("returns from the real successor drawer to the information modal, then to the saved row", async () => {
  vi.mocked(getRemasterRelationships)
    .mockReset()
    .mockResolvedValue({
      artifact_generation: "current",
      subject: {
        kind: "record",
        selection: rootNavigation("rules:absent"),
        title: null,
        available: false,
      },
      relationships: { kind: "content", content: remasterInformationFixture() },
    });
  vi.mocked(getRecordDetail).mockResolvedValue(detailFixture("rules:new", "New name"));
  render(
    <RouterTestContext>
      <QueryClientProvider
        client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
      >
        <ConfigProvider theme={{ token: { motion: false } }}>
          <App>
            <ReferencePreviewProvider>
              <UnavailableRecordRemasterAction
                recordKey="rules:absent"
                title="Retained title"
              />
            </ReferencePreviewProvider>
          </App>
        </ConfigProvider>
      </QueryClientProvider>
    </RouterTestContext>,
  );
  const opener = screen.getByRole("button", { name: "Remaster information" });
  opener.focus();
  fireEvent.click(opener);
  const information = await screen.findByRole("dialog", {
    name: "Remaster information · Retained title",
  });
  const successor = await screen.findByRole("link", { name: "New name" });
  successor.focus();
  fireEvent.click(successor);
  const drawer = await screen.findByRole("dialog", { name: "New name" });
  expect(screen.getAllByRole("dialog")).toHaveLength(2);
  fireEvent.click(
    within(drawer).getByRole("button", { name: "Close reference preview" }),
  );
  await waitFor(() =>
    expect(screen.queryByRole("dialog", { name: "New name" })).toBeNull(),
  );
  await waitFor(() => expect(successor).toHaveFocus());
  expect(information).toBeVisible();
  fireEvent.keyDown(information, { key: "Escape", keyCode: 27 });
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  await waitFor(() => expect(opener).toHaveFocus());
});
