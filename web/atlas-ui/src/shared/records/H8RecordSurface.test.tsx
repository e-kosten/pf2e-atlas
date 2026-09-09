import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import type {
  H8FactView,
  JournalPageEntryView,
  RecordSurfaceView,
  TableResultEntryView,
} from "../../generated/atlas";
import { RecordSurface } from "./RecordSurface";
import { rollTable } from "../../api/atlasApi";

vi.mock("../../api/atlasApi", () => ({ rollTable: vi.fn() }));

const rollTableMock = vi.mocked(rollTable);

const missing = { state: "missing" } as const;
const known = <T,>(value: T): H8FactView<T> => ({ state: "known", value });
const pageTextLocator = "v1~j~s~706167652d74657874";
const pageImageLocator = "v1~j~s~706167652d696d616765";
const resultOneLocator = "v1~t~s~4531636a674171465a497a436a447555";
const resultTwoLocator = "v1~t~s~726573756c742d74776f";
const ancestralMightLocator =
  "v1~j~s~6a6f75726e616c2d706167652d616e6365737472616c2d6d69676874";

describe("H8 record surfaces", () => {
  beforeEach(() => {
    history.replaceState(null, "", "/");
    rollTableMock.mockReset();
  });

  it("uses the parent journal route for stable page navigation without rendering media", () => {
    const surface = journalSurface();
    history.replaceState(null, "", "/records/journals%3Ahero-points");
    const { rerender } = render(
      <RecordSurface onReference={vi.fn()} surface={surface} />,
    );

    expect(screen.getByRole("heading", { name: "Hero Points" })).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { level: 2, name: "Introduction" }),
    ).toBeInTheDocument();
    expect(screen.getByText("Welcome to the deck.")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Artwork" }));
    expect(window.location.pathname).toBe("/records/journals%3Ahero-points");
    expect(window.location.search).toBe(`?child=${pageImageLocator}`);

    rerender(<RecordSurface onReference={vi.fn()} surface={surface} />);
    expect(screen.getByText("Media metadata only")).toBeInTheDocument();
    expect(screen.queryByRole("heading", { name: "Artwork" })).toBeNull();
    expect(document.querySelector("img, video, iframe, embed, object")).toBeNull();
  });

  it("links table results through the parent route and keeps unsupported siblings visible", () => {
    const surface = rollTableSurface();
    const onReference = vi.fn();
    history.replaceState(null, "", "/records/rollable-tables%3AzgZoI7h0XjjJrrNK");
    const { rerender } = render(
      <RecordSurface onReference={onReference} surface={surface} />,
    );

    expect(screen.getByText("Draw a card.")).toBeInTheDocument();
    expect(screen.getByText("Result 2 is unavailable")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("link", { name: "Ancestral Might" }));
    expect(onReference).toHaveBeenCalledWith(
      "journals:hero-points",
      ancestralMightLocator,
    );
    const openResult = screen.getByRole("link", { name: "Open result 1" });
    expect(openResult).toHaveAttribute(
      "href",
      `/records/rollable-tables%3AzgZoI7h0XjjJrrNK?child=${resultOneLocator}`,
    );
    fireEvent.click(openResult);
    expect(window.location.pathname).toBe(
      "/records/rollable-tables%3AzgZoI7h0XjjJrrNK",
    );
    expect(window.location.search).toBe(`?child=${resultOneLocator}`);

    rerender(<RecordSurface onReference={onReference} surface={surface} />);
    expect(screen.getByText("Result 1 selected")).toHaveAttribute(
      "aria-current",
      "page",
    );
    expect(screen.getByLabelText("Selected result 1")).toHaveFocus();
    expect(screen.queryByRole("link", { name: "Open result 1" })).toBeNull();
  });

  it("presents every read-only drawn source state without collapsing false", () => {
    const { rerender } = render(
      <RecordSurface onReference={vi.fn()} surface={rollTableSurface(known(false))} />,
    );
    expect(screen.getByText("Drawn: No")).toBeInTheDocument();

    rerender(
      <RecordSurface onReference={vi.fn()} surface={rollTableSurface(known(true))} />,
    );
    expect(screen.getByText("Drawn: Yes")).toBeInTheDocument();

    rerender(
      <RecordSurface onReference={vi.fn()} surface={rollTableSurface(missing)} />,
    );
    expect(screen.getByText("Drawn: missing")).toBeInTheDocument();

    rerender(
      <RecordSurface
        onReference={vi.fn()}
        surface={rollTableSurface({ state: "null" })}
      />,
    );
    expect(screen.getByText("Drawn: null")).toBeInTheDocument();

    rerender(
      <RecordSurface
        onReference={vi.fn()}
        surface={rollTableSurface({
          state: "unsupported",
          value: {
            shape: "string",
            exact_value: '"yes"',
            reason: "source_field_drift",
          },
        })}
      />,
    );
    expect(screen.getByText("Drawn: unsupported")).toBeInTheDocument();
  });

  it("rolls displayRoll-false tables from backend capability and opens every ordered outcome", async () => {
    const surface = rollableTableSurface();
    if (surface.presentation.presentation_type !== "roll_table") {
      throw new Error("roll-table fixture");
    }
    const result =
      surface.presentation.body.results.state === "known"
        ? surface.presentation.body.results.value.find(
            (entry) => entry.entry_type === "result",
          )
        : undefined;
    if (!result || result.entry_type !== "result") throw new Error("result fixture");
    rollTableMock.mockResolvedValue({
      state: "available",
      table_key: "rollable-tables:zgZoI7h0XjjJrrNK",
      formula: "1d2",
      total: 1,
      outcomes: [result.result],
    });

    render(<RecordSurface onReference={vi.fn()} surface={surface} />);
    const button = screen.getByRole("button", { name: "Roll" });
    fireEvent.click(button);
    expect(button).toBeDisabled();

    await waitFor(() => expect(rollTableMock).toHaveBeenCalledTimes(1));
    expect(rollTableMock).toHaveBeenCalledWith(
      "rollable-tables:zgZoI7h0XjjJrrNK",
      expect.any(AbortSignal),
    );
    expect(await screen.findByText("Rolled 1 on 1d2")).toBeInTheDocument();
    const link = screen.getByRole("link", { name: "Open rolled result 1" });
    fireEvent.click(link);
    expect(window.location.search).toBe(`?child=${resultOneLocator}`);
  });

  it("shows typed roll unavailability without parsing the source formula", () => {
    const surface = rollTableSurface();
    if (surface.presentation.presentation_type !== "roll_table") {
      throw new Error("roll-table fixture");
    }
    surface.presentation.body.roll = {
      state: "unavailable",
      unavailable: {
        reason: "unsupported_result",
        subject: {
          subject_type: "result",
          locator: resultTwoLocator,
          source_ordinal: 1,
        },
        message: "Roll is unavailable because result 2 is unsupported.",
      },
    };

    render(<RecordSurface onReference={vi.fn()} surface={surface} />);
    expect(screen.getByText("Roll unavailable")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Roll" })).toBeNull();
  });

  it("distinguishes a valid result gap from backend operation unavailability", async () => {
    rollTableMock
      .mockResolvedValueOnce({
        state: "available",
        table_key: "rollable-tables:zgZoI7h0XjjJrrNK",
        formula: "1d2",
        total: 2,
        outcomes: [],
      })
      .mockResolvedValueOnce({
        state: "unavailable",
        table_key: "rollable-tables:zgZoI7h0XjjJrrNK",
        unavailable: {
          reason: "range_unsupported",
          subject: {
            subject_type: "result",
            locator: resultOneLocator,
            source_ordinal: 0,
          },
          message: "Roll is unavailable because result 1 has an unsupported range.",
        },
      });
    render(<RecordSurface onReference={vi.fn()} surface={rollableTableSurface()} />);

    fireEvent.click(screen.getByRole("button", { name: "Roll" }));
    expect(await screen.findByText("No results match total 2.")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Roll" }));
    expect(await screen.findByText("Roll unavailable")).toBeInTheDocument();
    expect(
      screen.getByText(
        "Roll is unavailable because result 1 has an unsupported range.",
      ),
    ).toBeInTheDocument();
  });

  it("aborts and discards a pending response when the parent record changes", async () => {
    let resolveRoll:
      | ((value: Awaited<ReturnType<typeof rollTable>>) => void)
      | undefined;
    rollTableMock.mockImplementation(
      () =>
        new Promise((resolve) => {
          resolveRoll = resolve;
        }),
    );
    const first = rollableTableSurface();
    const second = rollableTableSurface();
    second.metadata.record_key = "rollable-tables:other";
    const { rerender } = render(
      <RecordSurface onReference={vi.fn()} surface={first} />,
    );
    fireEvent.click(screen.getByRole("button", { name: "Roll" }));
    const signal = rollTableMock.mock.calls[0]?.[1];
    rerender(<RecordSurface onReference={vi.fn()} surface={second} />);
    expect(signal?.aborted).toBe(true);
    resolveRoll?.({
      state: "available",
      table_key: "rollable-tables:zgZoI7h0XjjJrrNK",
      formula: "1d2",
      total: 1,
      outcomes: [],
    });
    await Promise.resolve();
    expect(screen.queryByText("Rolled 1 on 1d2")).toBeNull();
  });
});

function journalSurface(): RecordSurfaceView {
  const textPage: JournalPageEntryView = {
    entry_type: "page",
    page: {
      locator: pageTextLocator,
      identity_stability: "stable_source_id",
      source_id: known("page-text"),
      source_ordinal: 0,
      name: known("Introduction"),
      page_kind: known("text"),
      sort: known(0),
      title: known({ show: known(true), level: known(2) }),
      text: known({
        content: known([
          {
            block_type: "paragraph",
            spans: [{ span_type: "text", text: "Welcome to the deck." }],
          },
        ]),
        format: known(1),
        markdown: missing,
      }),
      source: missing,
      image_source: known("{}"),
      image_caption: missing,
      video: missing,
      source_system: missing,
      source_metadata: { ownership: missing, flags: missing, stats: missing },
    },
  };
  const imagePage: JournalPageEntryView = {
    entry_type: "page",
    page: {
      locator: pageImageLocator,
      identity_stability: "stable_source_id",
      source_id: known("page-image"),
      source_ordinal: 1,
      name: known("Artwork"),
      page_kind: known("image"),
      sort: known(10),
      title: known({ show: known(false), level: known(2) }),
      text: missing,
      source: known("systems/pf2e/assets/hero.webp"),
      image_source: known('{"caption":"Hero Point card art"}'),
      image_caption: known("Hero Point card art"),
      video: missing,
      source_system: missing,
      source_metadata: { ownership: missing, flags: missing, stats: missing },
    },
  };
  return {
    metadata: {
      record_key: "journals:hero-points",
      title: "Hero Points",
      kind: "journal",
      kind_label: "Journal",
    },
    profile: "record_detail",
    presentation: {
      presentation_type: "journal",
      body: {
        source_id: "journal-id",
        pages: known([textPage, imagePage]),
        source_metadata: sourceMetadata(),
        provenance: provenance("packs/journals/hero-points.json"),
      },
    },
  };
}

function rollTableSurface(
  drawn: H8FactView<boolean> = known(false),
): RecordSurfaceView {
  const result: TableResultEntryView = {
    entry_type: "result",
    result: {
      locator: resultOneLocator,
      identity_stability: "stable_source_id",
      source_id: known("E1cjgAqFZIzCjDuU"),
      source_ordinal: 0,
      result_kind: known("text"),
      text: known([
        {
          block_type: "paragraph",
          spans: [
            { span_type: "text", text: "Gain a Hero Point from " },
            {
              span_type: "reference",
              label: "Ancestral Might",
              record_key: "journals:hero-points",
              child_locator: ancestralMightLocator,
              embedded: false,
            },
            { span_type: "text", text: "." },
          ],
        },
      ]),
      collection: missing,
      document_id: missing,
      weight: known("1"),
      range: known({ first: 1, last: 1 }),
      drawn,
      image: missing,
      source_metadata: { flags: missing },
    },
  };
  const unsupported: TableResultEntryView = {
    entry_type: "unsupported",
    unsupported: {
      locator: resultTwoLocator,
      identity_stability: "stable_source_id",
      source_id: known("result-two"),
      source_ordinal: 1,
      exact_source: '{"_id":"result-two","img":"a","img":"b"}',
      reason: "duplicate fixed member `img`",
    },
  };
  return {
    metadata: {
      record_key: "rollable-tables:zgZoI7h0XjjJrrNK",
      title: "Hero Point Deck",
      kind: "roll_table",
      kind_label: "Roll Table",
    },
    profile: "record_detail",
    presentation: {
      presentation_type: "roll_table",
      body: {
        source_id: "table-id",
        description: known([
          {
            block_type: "paragraph",
            spans: [{ span_type: "text", text: "Draw a card." }],
          },
        ]),
        results: known([result, unsupported]),
        formula: known("1d2"),
        replacement: known(true),
        display_roll: known(false),
        image: missing,
        roll: {
          state: "unavailable",
          unavailable: {
            reason: "unsupported_result",
            subject: {
              subject_type: "result",
              locator: resultTwoLocator,
              source_ordinal: 1,
            },
            message: "Roll is unavailable because result 2 is unsupported.",
          },
        },
        source_metadata: sourceMetadata(),
        provenance: provenance("packs/rollable-tables/hero-point-deck.json"),
      },
    },
  };
}

function rollableTableSurface(): RecordSurfaceView {
  const surface = rollTableSurface();
  if (surface.presentation.presentation_type !== "roll_table") {
    throw new Error("roll-table fixture");
  }
  if (surface.presentation.body.results.state !== "known") {
    throw new Error("known result fixture");
  }
  surface.presentation.body.results = known(
    surface.presentation.body.results.value.filter(
      (entry) => entry.entry_type === "result",
    ),
  );
  surface.presentation.body.roll = {
    state: "available",
    formula: "1d2",
    sides: 2,
  };
  return surface;
}

function sourceMetadata() {
  return {
    folder: missing,
    sort: known(0),
    ownership: missing,
    flags: missing,
    stats: missing,
  };
}

function provenance(sourcePath: string) {
  return {
    source_path: sourcePath,
    source_contract_version: "h8.v1",
    source_system_version: "7.4.0",
    source_upstream_commit: "4cbdaa37",
  };
}
