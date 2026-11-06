import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ConfigProvider } from "antd";
import { useState } from "react";
import { RouterTestContext } from "../../test/router";
import { detailFixture } from "../../test/fixtures";
import { getDefinitionDetail, getRecordDetail } from "../../api/atlasApi";
import type { DefinitionDetailView, ReferenceHelpView } from "../../generated/atlas";
import {
  ReferenceHelpLink,
  ReferenceHelpProvider,
  type ReferenceHelpHandoff,
  type ReferenceHelpReturn,
} from "./ReferenceHelp";
import { ReferenceHelpHost } from "./ReferenceHelpHost";
import { rootNavigation } from "./useReferenceReading";
import { definitionHelpMatchesRecord } from "./useDefinitionDetail";
import { recordDetailQueryOptions } from "./recordDetailQuery";

vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  getRecordDetail: vi.fn(),
  getDefinitionDetail: vi.fn(),
}));

const propertyHelp: ReferenceHelpView = {
  definition_key: "sense:darkvision",
  availability: "available",
  destination: {
    kind: "record",
    navigation: rootNavigation("abilities:darkvision"),
    reading_context: "general_definition",
  },
  related_filters: [
    {
      kind: "search",
      label: "Creatures with this sense",
      artifact_generation: "generation-a",
      predicate: {
        kind: "in",
        clause_id: "sense",
        field: "senses.type",
        values: ["darkvision"],
      },
    },
  ],
};
const languageHelp: ReferenceHelpView = {
  definition_key: "language:chthonian",
  availability: "available",
  destination: { kind: "definition", key: "language:chthonian" },
  related_filters: [],
};
const standalone: DefinitionDetailView = {
  artifact_generation: "generation-a",
  definition_key: "language:chthonian",
  label: "Chthonian",
  help: languageHelp,
  content: { body: { kind: "plain", text: "Language context." }, references: [] },
  attribution: null,
  notices: [],
  references: [],
  references_availability: "available",
  references_truncated: false,
};

function wrapper({ children }: { children: React.ReactNode }) {
  return (
    <RouterTestContext>
      <QueryClientProvider
        client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
      >
        <ConfigProvider theme={{ token: { motion: false } }}>{children}</ConfigProvider>
      </QueryClientProvider>
    </RouterTestContext>
  );
}

it("retains real record identity, property context and native activation", () => {
  const open = vi.fn();
  render(
    <ReferenceHelpProvider onHelp={open} generation="generation-a">
      <ReferenceHelpLink help={propertyHelp}>Darkvision</ReferenceHelpLink>
      <ReferenceHelpLink help={{ ...languageHelp, destination: null }}>
        Unknown language
      </ReferenceHelpLink>
    </ReferenceHelpProvider>,
    { wrapper },
  );
  const link = screen.getByRole("link", { name: "Darkvision" });
  expect(link).toHaveAttribute(
    "href",
    "/records/abilities:darkvision?reading_context=general_definition&definition_key=sense%3Adarkvision",
  );
  fireEvent.click(link, { ctrlKey: true });
  fireEvent.click(link, { button: 1 });
  expect(open).not.toHaveBeenCalled();
  fireEvent.click(link);
  expect(open).toHaveBeenCalledWith(propertyHelp, link, "generation-a");
  expect(screen.queryByRole("link", { name: "Unknown language" })).toBeNull();
});

it("uses one mixed Drawer session through nested hosts while preserving the draft and opener", async () => {
  const record = detailFixture("abilities:darkvision", "Darkvision");
  record.artifact_generation = "generation-a";
  record.presentation.identity.rarity = {
    value: "chthonian",
    label: "Language context",
    help: languageHelp,
  };
  vi.mocked(getRecordDetail).mockResolvedValue(record);
  vi.mocked(getDefinitionDetail).mockImplementation(async (request) =>
    request.definition_key === "sense:darkvision"
      ? {
          ...standalone,
          definition_key: "sense:darkvision",
          label: "Darkvision",
          help: propertyHelp,
          content: null,
        }
      : standalone,
  );
  function Host() {
    const [draft, setDraft] = useState("Keep this filter draft");
    return (
      <ReferenceHelpHost generation="generation-a">
        <label>
          Filter draft
          <input value={draft} onChange={(event) => setDraft(event.target.value)} />
        </label>
        <ReferenceHelpHost>
          <ReferenceHelpLink help={propertyHelp}>Open sense</ReferenceHelpLink>
        </ReferenceHelpHost>
        <p>Selected creature</p>
      </ReferenceHelpHost>
    );
  }
  render(<Host />, { wrapper });
  const opener = screen.getByRole("link", { name: "Open sense" });
  opener.focus();
  fireEvent.click(opener);
  const drawer = await screen.findByRole("dialog", { name: "Darkvision" });
  expect(screen.getAllByRole("dialog")).toHaveLength(1);
  expect(vi.mocked(getRecordDetail).mock.calls[0]?.[1]).toMatchObject({
    reading_context: "general_definition",
  });
  expect(
    within(drawer).queryByRole("link", { name: "Creatures with this sense" }),
  ).toBeNull();
  fireEvent.click(within(drawer).getByRole("link", { name: "Language context" }));
  const language = await screen.findByRole("dialog", { name: "Chthonian" });
  expect(screen.getAllByRole("dialog")).toHaveLength(1);
  expect(within(language).getByText("Language context.")).toBeVisible();
  fireEvent.click(within(language).getByRole("button", { name: "Back" }));
  expect(await screen.findByRole("dialog", { name: "Darkvision" })).toBeVisible();
  fireEvent.click(screen.getByRole("button", { name: "Close reference preview" }));
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  expect(opener).toHaveFocus();
  expect(screen.getByRole("textbox", { name: "Filter draft" })).toHaveValue(
    "Keep this filter draft",
  );
  expect(screen.getByText("Selected creature")).toBeVisible();
});

it("separates ordinary and general requests and rejects changed semantic mappings", () => {
  const navigation = rootNavigation("abilities:darkvision");
  expect(
    recordDetailQueryOptions(navigation.record_key, navigation).queryKey,
  ).not.toEqual(
    recordDetailQueryOptions(
      navigation.record_key,
      navigation,
      undefined,
      "general_definition",
    ).queryKey,
  );
  expect(
    definitionHelpMatchesRecord(propertyHelp, navigation, "general_definition"),
  ).toBe(true);
  expect(definitionHelpMatchesRecord(propertyHelp, navigation)).toBe(false);
  expect(
    definitionHelpMatchesRecord(
      propertyHelp,
      {
        ...navigation,
        embedded_path: [{ collection: "/items", identity: { Stable: "owned" } }],
      },
      "general_definition",
    ),
  ).toBe(false);
  expect(
    definitionHelpMatchesRecord(
      propertyHelp,
      rootNavigation("abilities:other"),
      "general_definition",
    ),
  ).toBe(false);
});

it("hands primary reading to its owner before opening while modified links remain native", () => {
  const open = vi.fn();
  const finish = vi.fn();
  let startReading: Parameters<ReferenceHelpHandoff>[1] | undefined;
  const handoff = vi.fn<ReferenceHelpHandoff>((_anchor, start) => {
    startReading = start;
  });
  render(
    <ReferenceHelpProvider onHelp={open} generation="generation-a">
      <ReferenceHelpLink help={languageHelp} handoff={handoff}>
        Read Chthonian
      </ReferenceHelpLink>
    </ReferenceHelpProvider>,
    { wrapper },
  );
  const link = screen.getByRole("link", { name: "Read Chthonian" });
  fireEvent.click(link, { ctrlKey: true });
  fireEvent.click(link, { button: 1 });
  expect(handoff).not.toHaveBeenCalled();
  expect(open).not.toHaveBeenCalled();
  fireEvent.click(link);
  expect(handoff).toHaveBeenCalledExactlyOnceWith(link, expect.any(Function));
  expect(open).not.toHaveBeenCalled();
  act(() => startReading!(finish));
  expect(open).toHaveBeenCalledExactlyOnceWith(
    languageHelp,
    link,
    "generation-a",
    finish,
  );
});

it.each(["return", "navigate"] as const)(
  "finishes a suspended owner exactly once with %s after nested reading",
  async (reason) => {
    history.replaceState(null, "", "/search");
    const record = detailFixture("abilities:darkvision", "Darkvision");
    record.artifact_generation = "generation-a";
    record.presentation.identity.rarity = {
      value: "chthonian",
      label: "Language context",
      help: languageHelp,
    };
    vi.mocked(getRecordDetail).mockResolvedValue(record);
    vi.mocked(getDefinitionDetail).mockImplementation(async (request) =>
      request.definition_key === "sense:darkvision"
        ? {
            ...standalone,
            definition_key: "sense:darkvision",
            label: "Darkvision",
            help: propertyHelp,
            content: null,
          }
        : standalone,
    );
    let owner: HTMLElement;
    let opener: HTMLElement;
    const finish = vi.fn<ReferenceHelpReturn>((result) => {
      if (result === "return") {
        owner.hidden = false;
        opener.focus({ preventScroll: true });
      }
    });
    const handoff: ReferenceHelpHandoff = (anchor, start) => {
      opener = anchor;
      owner = anchor.closest<HTMLElement>("[data-picker-owner]")!;
      owner.hidden = true;
      start(finish);
    };
    render(
      <ReferenceHelpHost generation="generation-a">
        <div data-picker-owner>
          <label>
            Unfinished filter
            <input defaultValue="Keep exact draft" />
          </label>
          <ReferenceHelpLink help={propertyHelp} handoff={handoff}>
            Read sense
          </ReferenceHelpLink>
        </div>
      </ReferenceHelpHost>,
      { wrapper },
    );
    const link = screen.getByRole("link", { name: "Read sense" });
    link.focus();
    const focus = vi.spyOn(link, "focus");
    fireEvent.click(link);
    let drawer = await screen.findByRole("dialog", { name: "Darkvision" });
    fireEvent.click(within(drawer).getByRole("link", { name: "Language context" }));
    drawer = await screen.findByRole("dialog", { name: "Chthonian" });
    expect(finish).not.toHaveBeenCalled();
    if (reason === "return")
      fireEvent.click(
        within(drawer).getByRole("button", { name: "Close reference preview" }),
      );
    else
      fireEvent.click(
        within(drawer).getByRole("link", { name: "Open reference full page" }),
      );
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
    expect(finish).toHaveBeenCalledExactlyOnceWith(reason);
    if (reason === "return") {
      expect(link).toHaveFocus();
      expect(focus).toHaveBeenCalledTimes(1);
      expect(screen.getByRole("textbox", { name: "Unfinished filter" })).toHaveValue(
        "Keep exact draft",
      );
    } else {
      expect(owner!.hidden).toBe(true);
      expect(focus).not.toHaveBeenCalled();
      expect(link).not.toHaveFocus();
      expect(location.pathname).toBe("/definitions/language:chthonian");
    }
  },
);
