import { fireEvent, render, screen } from "@testing-library/react";
import type {
  H8FactView,
  JournalPageEntryView,
  RecordSurfaceView,
  TableResultEntryView,
} from "../../generated/atlas";
import { RecordSurface } from "./RecordSurface";

const missing = { state: "missing" } as const;
const known = <T,>(value: T): H8FactView<T> => ({ state: "known", value });

describe("H8 record surfaces", () => {
  beforeEach(() => history.replaceState(null, "", "/"));

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
    expect(window.location.search).toBe("?child=page-image");

    rerender(<RecordSurface onReference={vi.fn()} surface={surface} />);
    expect(screen.getByText("Media metadata only")).toBeInTheDocument();
    expect(screen.queryByRole("heading", { name: "Artwork" })).toBeNull();
    expect(document.querySelector("img, video, iframe, embed, object")).toBeNull();
  });

  it("links table results through the parent route and keeps unsupported siblings visible", () => {
    const surface = rollTableSurface();
    const onReference = vi.fn();
    history.replaceState(null, "", "/records/roll-tables%3Ahero-points");
    render(<RecordSurface onReference={onReference} surface={surface} />);

    expect(screen.getByText("Draw a card.")).toBeInTheDocument();
    expect(screen.getByText("Result 2 is unavailable")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("link", { name: "Ancestral Might" }));
    expect(onReference).toHaveBeenCalledWith(
      "journals:hero-points",
      "journal-page-ancestral-might",
    );
    fireEvent.click(screen.getByRole("button", { name: "Open result 1" }));
    expect(window.location.pathname).toBe("/records/roll-tables%3Ahero-points");
    expect(window.location.search).toBe("?child=result-one");
  });
});

function journalSurface(): RecordSurfaceView {
  const textPage: JournalPageEntryView = {
    entry_type: "page",
    page: {
      locator: "page-text",
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
      locator: "page-image",
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
        provenance: provenance(),
      },
    },
  };
}

function rollTableSurface(): RecordSurfaceView {
  const result: TableResultEntryView = {
    entry_type: "result",
    result: {
      locator: "result-one",
      identity_stability: "stable_source_id",
      source_id: known("result-one"),
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
              child_locator: "journal-page-ancestral-might",
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
      drawn: known(false),
      image: missing,
      source_metadata: { flags: missing },
    },
  };
  const unsupported: TableResultEntryView = {
    entry_type: "unsupported",
    unsupported: {
      locator: "result-two",
      identity_stability: "stable_source_id",
      source_ordinal: 1,
      exact_source: '{"_id":"result-two","img":"a","img":"b"}',
      reason: "duplicate fixed member `img`",
    },
  };
  return {
    metadata: {
      record_key: "roll-tables:hero-points",
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
        source_metadata: sourceMetadata(),
        provenance: provenance(),
      },
    },
  };
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

function provenance() {
  return {
    source_path: "packs/journals/hero-points.json",
    source_contract_version: "h8.v1",
    source_system_version: "7.4.0",
    source_upstream_commit: "4cbdaa37",
  };
}
