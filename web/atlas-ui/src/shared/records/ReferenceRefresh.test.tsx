import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ConfigProvider } from "antd";
import { useState } from "react";
import {
  getDefinitionDetail,
  getRecordDetail,
  AtlasApiError,
} from "../../api/atlasApi";
import type {
  DefinitionDetailView,
  ReferenceHelpView,
  RecordNavigationView,
} from "../../generated/atlas";
import { RouterTestContext } from "../../test/router";
import { detailFixture } from "../../test/fixtures";
import { ReferenceHelpHost } from "./ReferenceHelpHost";
import { ReferenceHelpLink } from "./ReferenceHelp";
import { rootNavigation } from "./useReferenceReading";

vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  getDefinitionDetail: vi.fn(),
  getRecordDetail: vi.fn(),
}));

const changed = () =>
  new AtlasApiError(409, "The reference collection changed.", {
    code: "artifact_generation_changed",
    message: "The reference collection changed.",
    recoverable_action: "refresh_results",
  });
function definition(
  help: ReferenceHelpView,
  generation: string,
  text = "Current language context.",
): DefinitionDetailView {
  return {
    definition_key: help.definition_key!,
    label: "Definition title",
    artifact_generation: generation,
    help,
    content:
      help.destination?.kind === "definition"
        ? { body: { kind: "plain", text }, references: [] }
        : null,
    attribution: null,
    notices: [],
    references: [],
    references_availability: "available",
    references_truncated: false,
  };
}
const standaloneHelp: ReferenceHelpView = {
  definition_key: "language:chthonian",
  availability: "available",
  destination: { kind: "definition", key: "language:chthonian" },
  related_filters: [],
};
function setup(help: ReferenceHelpView) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  function Host() {
    const [draft, setDraft] = useState("Keep unfinished filter");
    return (
      <ReferenceHelpHost generation="g1">
        <div data-testid="host-scroll">
          <label>
            Filter draft
            <input value={draft} onChange={(event) => setDraft(event.target.value)} />
          </label>
          <ReferenceHelpLink help={help}>Open definition</ReferenceHelpLink>
        </div>
      </ReferenceHelpHost>
    );
  }
  render(
    <RouterTestContext>
      <QueryClientProvider client={client}>
        <ConfigProvider theme={{ token: { motion: false } }}>
          <Host />
        </ConfigProvider>
      </QueryClientProvider>
    </RouterTestContext>,
  );
  const host = screen.getByTestId("host-scroll");
  host.scrollTop = 127;
  const opener = screen.getByRole("link", { name: "Open definition" });
  opener.focus();
  fireEvent.click(opener);
  return { client, opener, host };
}
beforeEach(() => vi.clearAllMocks());

it("deliberately refreshes a stale standalone definition into the current generation and preserves its host", async () => {
  let generation = "g1";
  vi.mocked(getDefinitionDetail).mockImplementation(async (request) => {
    if (
      request.expected_artifact_generation &&
      request.expected_artifact_generation !== generation
    )
      throw changed();
    return definition(
      standaloneHelp,
      generation,
      generation === "g1" ? "Old language context." : "New language context.",
    );
  });
  const { client, opener, host } = setup(standaloneHelp);
  await screen.findByText("Old language context.");
  generation = "g2";
  await act(async () => {
    await client.invalidateQueries({ queryKey: ["definition-detail"] });
  });
  const refresh = await screen.findByRole("button", { name: "Refresh definition" });
  fireEvent.click(refresh);
  expect(await screen.findByText("New language context.")).toBeVisible();
  expect(screen.queryByText("Old language context.")).toBeNull();
  expect(
    vi
      .mocked(getDefinitionDetail)
      .mock.calls.some(([request]) => request.expected_artifact_generation === null),
  ).toBe(true);
  expect(
    vi
      .mocked(getDefinitionDetail)
      .mock.calls.some(([request]) => request.expected_artifact_generation === "g2"),
  ).toBe(true);
  expect(screen.getByRole("textbox", { name: "Filter draft" })).toHaveValue(
    "Keep unfinished filter",
  );
  expect(host.scrollTop).toBe(127);
  fireEvent.click(screen.getByRole("button", { name: "Close reference preview" }));
  await waitFor(() => expect(opener).toHaveFocus());
});

it.each(["changed", "unreadable"] as const)(
  "revalidates a record-backed property's %s mapping before displaying refreshed content",
  async (outcome) => {
    let generation = "g1";
    const old = rootNavigation("abilities:old");
    const replacement = rootNavigation("abilities:new");
    const original: ReferenceHelpView = {
      definition_key: "sense:darkvision",
      availability: "available",
      destination: {
        kind: "record",
        navigation: old,
        reading_context: "general_definition",
      },
      related_filters: [],
    };
    vi.mocked(getDefinitionDetail).mockImplementation(async (request) => {
      if (
        request.expected_artifact_generation &&
        request.expected_artifact_generation !== generation
      )
        throw changed();
      const help: ReferenceHelpView =
        generation === "g1"
          ? original
          : {
              ...original,
              availability: outcome === "unreadable" ? "label_only" : "available",
              destination:
                outcome === "unreadable"
                  ? null
                  : {
                      kind: "record",
                      navigation: replacement,
                      reading_context: "general_definition",
                    },
            };
      return definition(help, generation);
    });
    vi.mocked(getRecordDetail).mockImplementation(async (key) => {
      const detail = detailFixture(
        key,
        key === old.record_key ? "Old entry" : "New entry",
        "rules:nested",
      );
      detail.artifact_generation = generation;
      detail.presentation.content[0].body = {
        kind: "plain",
        text: key === old.record_key ? "Old authored body." : "New authored body.",
      };
      return detail;
    });
    const { client } = setup(original);
    await screen.findByText("Old authored body.");
    generation = "g2";
    await act(async () => {
      await client.invalidateQueries({
        predicate: (query) => query.meta?.reference === true,
      });
    });
    fireEvent.click(await screen.findByRole("button", { name: "Refresh definition" }));
    if (outcome === "changed") {
      expect(await screen.findByText("New authored body.")).toBeVisible();
      expect(
        screen.getByRole("link", { name: "Open reference full page" }),
      ).toHaveAttribute(
        "href",
        "/records/abilities:new?reading_context=general_definition&definition_key=sense%3Adarkvision",
      );
    } else {
      expect(
        await screen.findByText("This definition is no longer available."),
      ).toBeVisible();
      expect(
        screen.queryByRole("link", { name: "Open reference full page" }),
      ).toBeNull();
      expect(
        screen.getByRole("button", { name: "Open reference full page" }),
      ).toBeDisabled();
    }
    expect(screen.queryByText("Old authored body.")).toBeNull();
    expect(
      vi
        .mocked(getDefinitionDetail)
        .mock.calls.every(([request]) => request.definition_key === "sense:darkvision"),
    ).toBe(true);
    expect(
      vi
        .mocked(getRecordDetail)
        .mock.calls.filter(([key]) => key === replacement.record_key)
        .every(([, request]) => request?.reading_context === "general_definition"),
    ).toBe(true);
  },
);

it("keeps the complete direct-record address when refresh is refused", async () => {
  let generation = "g1";
  const selection: RecordNavigationView = {
    record_key: "actors:source",
    embedded_path: [{ collection: "/items", identity: { Stable: "owned" } }],
    field: "/system/description/value",
    passage: { kind: "identity" },
    spell_rank: 3,
    source_fingerprint: "old-source",
  };
  const help: ReferenceHelpView = {
    definition_key: null,
    availability: "available",
    destination: {
      kind: "record",
      navigation: selection,
      reading_context: "general_definition",
    },
    related_filters: [],
  };
  let refuse = false;
  vi.mocked(getRecordDetail).mockImplementation(async () => {
    if (refuse) throw new Error("The original source address is unavailable.");
    const detail = detailFixture(selection.record_key, "Owned entry");
    detail.artifact_generation = generation;
    detail.selected = selection;
    return detail;
  });
  const { client } = setup(help);
  await screen.findByRole("heading", { name: "Owned entry" });
  generation = "g2";
  await act(async () => {
    await client.invalidateQueries({ queryKey: ["record-detail"] });
  });
  refuse = true;
  fireEvent.click(await screen.findByRole("button", { name: "Refresh record" }));
  expect(await screen.findByText("Unable to load this record")).toBeVisible();
  for (const [key, request] of vi.mocked(getRecordDetail).mock.calls) {
    expect(key).toBe(selection.record_key);
    expect(request).toMatchObject({
      embedded_path: selection.embedded_path,
      fields: [selection.field],
      passage: selection.passage,
      spell_rank: 3,
      source_fingerprint: "old-source",
    });
  }
});

it("allows a failed deliberate refresh to retry against the current generation", async () => {
  let generation = "g1";
  let failedRefresh = false;
  vi.mocked(getDefinitionDetail).mockImplementation(async (request) => {
    if (
      request.expected_artifact_generation &&
      request.expected_artifact_generation !== generation
    )
      throw changed();
    if (request.expected_artifact_generation === null && !failedRefresh) {
      failedRefresh = true;
      throw new Error("Temporary refresh failure");
    }
    return definition(
      standaloneHelp,
      generation,
      generation === "g1" ? "Old language context." : "Recovered language context.",
    );
  });
  const { client } = setup(standaloneHelp);
  await screen.findByText("Old language context.");
  generation = "g2";
  await act(async () => {
    await client.invalidateQueries({ queryKey: ["definition-detail"] });
  });
  fireEvent.click(await screen.findByRole("button", { name: "Refresh definition" }));
  await screen.findByText("Unable to load this definition");
  fireEvent.click(screen.getByRole("button", { name: "Refresh definition" }));
  expect(await screen.findByText("Recovered language context.")).toBeVisible();
});
