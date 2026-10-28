import { fireEvent, render, screen } from "@testing-library/react";
import type {
  PreparedContentFieldView,
  SimilarityPartsView,
} from "../../generated/atlas";
import { PreparedContent, PreparedContentBody } from "./PreparedContent";
import { SourceSectionSelectionContext } from "./SourceSectionSelectionContext";
import { RecordReferenceDrawer } from "./RecordReferenceDrawer";
import {
  actorFixture,
  activityFixture,
  detailFixture,
  knownFact,
  summaryFixture,
} from "../../test/fixtures";
import { ActorActivities } from "./ActorReference";
import { RecordPresentation } from "./RecordPresentation";

const location = {
  record: "journals:source",
  embedded_path: [],
  field: "/text/content",
};
const content: PreparedContentFieldView = {
  locator: location,
  source_fingerprint: "source-1",
  role: "description",
  body: {
    kind: "html",
    html: "<p>Original prepared source</p>",
    controls: [],
    inclusions: [],
  },
  section_rendering: {
    selection_version: "structural-source-sections/v4",
    html: '<h2 data-atlas-source-section="0">Shared heading</h2><p data-atlas-source-section="0">First prose <a href="https://example.com">Native reference</a></p><h2 data-atlas-source-section="1">Shared heading</h2><p data-atlas-source-section="1">Second prose</p><p data-atlas-source-section="2">No vector prose</p><div class="prepared-inclusion"><p data-atlas-source-section="0">Foreign included prose</p></div>',
  },
};
const inventory = {
  artifact_generation: "generation-1",
  search_profile: "bge-base",
  seed: { kind: "record", record_key: location.record },
  role_recipe: "roles/v1",
  supports_corresponding_parts: true,
  core: {
    family: "hazard",
    selectors: ["part-0"],
    representation: "identity_and_prose",
    fallback_reason: null,
    recipe: "core-sections/v1",
  },
  parts: [0, 1, 2].map((ordinal) => ({
    kind: "section" as const,
    selector: `part-${ordinal}`,
    label: "Shared heading",
    owner_label: "Source",
    owners: [],
    field: location.field,
    section_ordinal: ordinal,
    role: "description",
    role_label: "Description",
    has_vectors: ordinal !== 2,
    label_only: false,
    navigation: {
      record_key: location.record,
      embedded_path: [],
      field: location.field,
      passage: null,
      source_fingerprint: "source-1",
    },
  })),
} satisfies SimilarityPartsView;
function source(
  selected: string[] | null,
  field = content,
  currentInventory: SimilarityPartsView | null = inventory,
) {
  return (
    <SourceSectionSelectionContext
      value={{
        seed: inventory.seed,
        generation: "generation-1",
        selection: currentInventory ? { inventory: currentInventory, selected } : null,
      }}
    >
      <input aria-label="Draft name" />
      <PreparedContent content={field} onReference={vi.fn()} documentNavigation />
    </SourceSectionSelectionContext>
  );
}

it("keeps rendered grant references outside selected source sections without losing links or focus", () => {
  const presentation = detailFixture(location.record, "Source").presentation;
  presentation.content = [content];
  const granted = summaryFixture("feats:granted", "Granted feat");
  granted.rarity = "rare";
  granted.traits = [{ kind: "trait", value: "fire", label: "Fire" }];
  const navigation = {
    record_key: granted.record_key,
    embedded_path: [],
    field: null,
    passage: null,
    source_fingerprint: null,
  };
  presentation.grants = {
    entries: [
      { title: granted.title, record: granted, navigation },
      {
        title: "Another grant",
        record: null,
        navigation: { ...navigation, record_key: "feats:another" },
      },
    ],
    unavailable_count: 0,
    truncated: false,
  };
  const onReference = vi.fn();
  function reading(selected: string[] | null) {
    return (
      <div className="comparison-source">
        <SourceSectionSelectionContext
          value={{
            seed: inventory.seed,
            generation: inventory.artifact_generation,
            selection: { inventory, selected },
          }}
        >
          <RecordPresentation presentation={presentation} onReference={onReference} />
        </SourceSectionSelectionContext>
      </div>
    );
  }
  const view = render(reading(null));
  const grants = screen.getByRole("region", { name: "Grants" });
  const link = screen.getByRole("link", { name: /Granted feat/ });
  const fallback = screen.getByRole("link", { name: "Another grant" });
  expect(screen.getByText(/First prose/)).toHaveClass("source-section-selected");
  expect(grants).toContainElement(screen.getByRole("heading", { name: "Grants" }));
  expect(grants).toHaveTextContent("Rare");
  expect(grants).toHaveTextContent("Fire");
  // Whole-root navigation has no redundant compact address.
  expect(link).toHaveAttribute("href", "/records/feats:granted");
  expect(fallback).toHaveAttribute("href", "/records/feats:another");
  link.focus();
  for (const selected of [[], ["part-1"], null]) {
    view.rerender(reading(selected));
    expect(link).toBe(screen.getByRole("link", { name: /Granted feat/ }));
    expect(link).toHaveFocus();
    expect(
      grants.querySelector(".source-section-selected, .source-owner-selected"),
    ).toBeNull();
    expect(link).not.toHaveAttribute("aria-disabled", "true");
  }
  fireEvent.click(link, { ctrlKey: true });
  expect(onReference).not.toHaveBeenCalled();
  fireEvent.click(link);
  expect(onReference).toHaveBeenCalledWith(granted.record_key, link, navigation);
  fireEvent.click(fallback);
  expect(onReference).toHaveBeenLastCalledWith(
    "feats:another",
    fallback,
    presentation.grants.entries[1].navigation,
  );
});

it.each(["gear", "spell"] as const)(
  "reflects selected embedded sections in compact %s links without selecting their facts",
  (kind) => {
    const activity = { ...activityFixture("Embedded entry", location.record), kind };
    const actor = actorFixture();
    actor.activities = knownFact([activity]);
    const onReference = vi.fn();
    const embeddedInventory = {
      ...inventory,
      parts: inventory.parts.map((part) => ({
        ...part,
        owners: activity.navigation.embedded_path,
        navigation: { ...part.navigation!, source_fingerprint: "snapshot" },
      })),
    };
    function reading(selected: string[] | null, current = embeddedInventory) {
      return (
        <SourceSectionSelectionContext
          value={{
            seed: inventory.seed,
            generation: "generation-1",
            selection: { inventory: current, selected },
          }}
        >
          <ActorActivities actor={actor} onReference={onReference} />
        </SourceSectionSelectionContext>
      );
    }
    const view = render(reading(["part-1"]));
    const link = screen.getByRole("link", { name: "Embedded entry" });
    expect(link).toHaveClass("source-owner-selected");
    fireEvent.click(link);
    expect(onReference).toHaveBeenCalledOnce();
    for (const selected of [[], ["part-2"]]) {
      view.rerender(reading(selected));
      expect(link).not.toHaveClass("source-owner-selected");
      expect(link).toBe(screen.getByRole("link", { name: "Embedded entry" }));
    }
    view.rerender(reading(null));
    expect(link).toHaveClass("source-owner-selected");
    for (const current of [
      { ...embeddedInventory, artifact_generation: "generation-2" },
      { ...embeddedInventory, seed: { kind: "record" as const, record_key: "other" } },
      {
        ...embeddedInventory,
        parts: embeddedInventory.parts.map((part) => ({ ...part, owners: [] })),
      },
      {
        ...embeddedInventory,
        parts: embeddedInventory.parts.map((part) => ({
          ...part,
          navigation: { ...part.navigation!, source_fingerprint: "foreign" },
        })),
      },
    ]) {
      view.rerender(reading(null, current));
      expect(link).not.toHaveClass("source-owner-selected");
    }
    view.rerender(<ActorActivities actor={actor} onReference={onReference} />);
    expect(link).not.toHaveClass("source-owner-selected");
  },
);

it("dims exact canonical sections live without replacing readable prose, native links, or focus", () => {
  const view = render(source(null));
  const first = screen.getByText(/First prose/);
  const second = screen.getByText("Second prose");
  const link = screen.getByRole("link", { name: "Native reference" });
  expect(first).toHaveClass("source-section-selected");
  expect(second).toHaveClass("source-section-selected");
  expect(screen.getByText("No vector prose")).not.toHaveClass(
    "source-section-selected",
  );
  expect(screen.getByText("Foreign included prose")).not.toHaveClass(
    "source-section-selected",
  );
  const input = screen.getByRole("textbox", { name: "Draft name" });
  input.focus();
  fireEvent.change(input, { target: { value: "Draft" } });
  view.rerender(source(["part-1"]));
  expect(first).toHaveClass("source-section-excluded");
  expect(first).not.toHaveClass("source-section-selected");
  expect(second).toHaveClass("source-section-selected");
  expect(second).not.toHaveClass("source-section-excluded");
  expect(first).toBe(screen.getByText(/First prose/));
  expect(link).toBe(screen.getByRole("link", { name: "Native reference" }));
  expect(link).toHaveAttribute("href", "https://example.com");
  expect(link).not.toHaveAttribute("aria-disabled");
  expect(input).toHaveFocus();
  expect(input).toHaveValue("Draft");
  view.rerender(source([]));
  expect(first).toHaveClass("source-section-excluded");
  expect(second).toHaveClass("source-section-excluded");
  expect(screen.getByText("No vector prose")).not.toHaveClass(
    "source-section-excluded",
  );
  expect(screen.getByText("Foreign included prose")).not.toHaveClass(
    "source-section-excluded",
  );
  view.rerender(source(["part-0", "part-1"]));
  expect(first).not.toHaveClass("source-section-excluded");
  expect(second).not.toHaveClass("source-section-excluded");
});

it("keeps source nodes and focused links stable while selection inventory is pending or invalid", () => {
  const view = render(source(["part-1"]));
  const first = screen.getByText(/First prose/);
  const second = screen.getByText("Second prose");
  const link = screen.getByRole("link", { name: "Native reference" });
  link.focus();
  for (const unavailable of [
    null,
    { ...inventory, artifact_generation: "old-generation" },
    { ...inventory, seed: { kind: "record" as const, record_key: "journals:foreign" } },
  ]) {
    view.rerender(source(["part-1"], content, unavailable));
    expect(first).toBe(screen.getByText(/First prose/));
    expect(second).toBe(screen.getByText("Second prose"));
    expect(link).toBe(screen.getByRole("link", { name: "Native reference" }));
    expect(link).toHaveFocus();
    expect(
      view.container.querySelector(
        ".source-section-selected, .source-section-excluded",
      ),
    ).toBeNull();
  }
  view.rerender(source(["part-1"]));
  expect(first).toBe(screen.getByText(/First prose/));
  expect(first).toHaveClass("source-section-excluded");
  expect(second).toHaveClass("source-section-selected");
  expect(link).toHaveFocus();
});

it.each([
  {
    name: "foreign record",
    field: { ...content, locator: { ...location, record: "journals:foreign" } },
  },
  {
    name: "owned document",
    field: {
      ...content,
      locator: {
        ...location,
        embedded_path: [{ collection: "items", identity: { Stable: "ability" } }],
      },
    },
  },
  {
    name: "different field",
    field: { ...content, locator: { ...location, field: "/other" } },
  },
  {
    name: "source fingerprint",
    field: { ...content, source_fingerprint: "new-source" },
  },
])("does not dim a $name outside the exact inventory address", ({ field }) => {
  const view = render(source([], field));
  expect(
    view.container.querySelector(".source-section-excluded, .source-section-selected"),
  ).toBeNull();
});

it("ignores inventory from another seed or generation", () => {
  const view = render(
    source([], content, { ...inventory, artifact_generation: "generation-2" }),
  );
  expect(
    view.container.querySelector(".source-section-excluded, .source-section-selected"),
  ).toBeNull();
  view.rerender(
    source([], content, {
      ...inventory,
      seed: { kind: "record", record_key: "journals:foreign" },
    }),
  );
  expect(
    view.container.querySelector(".source-section-excluded, .source-section-selected"),
  ).toBeNull();
});

it("leaves ordinary readers and unsupported sidecar versions on original prepared content", () => {
  const view = render(<PreparedContent content={content} onReference={vi.fn()} />);
  expect(screen.getByText("Original prepared source")).toBeVisible();
  expect(screen.queryByText("Second prose")).toBeNull();
  view.rerender(
    source([], {
      ...content,
      section_rendering: {
        ...content.section_rendering!,
        selection_version: "future/v5",
      },
    }),
  );
  expect(screen.getByText("Original prepared source")).toBeVisible();
  expect(
    view.container.querySelector(".source-section-excluded, .source-section-selected"),
  ).toBeNull();
});

it("clears source selection for a nested reference Drawer, including the same source record", () => {
  const detail = detailFixture(location.record, "Source");
  detail.presentation.content = [content];
  render(
    <SourceSectionSelectionContext
      value={{
        seed: inventory.seed,
        generation: "test-generation",
        selection: {
          inventory: { ...inventory, artifact_generation: "test-generation" },
          selected: [],
        },
      }}
    >
      <RecordReferenceDrawer
        anchor={null}
        detail={detail}
        loading={false}
        error={null}
        onRetry={vi.fn()}
        onClose={vi.fn()}
        onOpenFullPage={vi.fn()}
        onReference={vi.fn()}
        onRankChange={vi.fn()}
      />
    </SourceSectionSelectionContext>,
  );
  expect(screen.getByText("Original prepared source")).toBeVisible();
  expect(screen.queryByText("Second prose")).toBeNull();
});

it("treats a trait definition with several headings as its one canonical part", () => {
  const traitInventory: SimilarityPartsView = {
    ...inventory,
    seed: { kind: "trait", trait_key: "trait:fire" },
    parts: [{ ...inventory.parts[0], field: "/definition", navigation: null }],
  };
  render(
    <SourceSectionSelectionContext
      value={{
        seed: traitInventory.seed,
        generation: "generation-1",
        selection: { inventory: traitInventory, selected: [] },
      }}
    >
      <PreparedContentBody
        body={{ kind: "plain", text: "Original trait" }}
        references={[]}
        onReference={vi.fn()}
        sourceLocation={{ trait_key: "trait:fire" }}
        sectionRendering={{
          selection_version: "trait-definition-centroid/v1",
          html: '<h2 data-atlas-source-section="0">First trait heading</h2><p data-atlas-source-section="0">Trait definition</p><h2 data-atlas-source-section="0">Second trait heading</h2>',
        }}
      />
    </SourceSectionSelectionContext>,
  );
  expect(screen.getByText("Trait definition")).toHaveClass("source-section-excluded");
  expect(screen.getByRole("heading", { name: "Second trait heading" })).toHaveClass(
    "source-section-excluded",
  );
});

it("never paints an identity component as a selected prose section", () => {
  const current: SimilarityPartsView = {
    ...inventory,
    parts: [
      ...inventory.parts,
      {
        kind: "identity",
        selector: "identity-root",
        label: "Name and identity",
        owner_label: "Source",
        has_vectors: true,
        navigation: {
          record_key: location.record,
          embedded_path: [],
          field: null,
          passage: null,
          source_fingerprint: "source-1",
        },
      },
    ],
  };
  const view = render(source(["identity-root"], content, current));
  const first = screen.getByText(/First prose/);
  expect(first).not.toHaveClass("source-section-selected");
  expect(first).toHaveClass("source-section-excluded");
  view.rerender(source(["identity-root", "part-0"], content, current));
  expect(first).toHaveClass("source-section-selected");
  expect(screen.getByText("Second prose")).toHaveClass("source-section-excluded");
});
