import { fireEvent, render, screen } from "@testing-library/react";
import type {
  PreparedContentFieldView,
  SimilarityPartsView,
} from "../../generated/atlas";
import { PreparedContent, PreparedContentBody } from "./PreparedContent";
import { SourceSectionSelectionContext } from "./SourceSectionSelectionContext";
import { RecordReferenceDrawer } from "./RecordReferenceDrawer";
import { detailFixture } from "../../test/fixtures";

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
const inventory: SimilarityPartsView = {
  artifact_generation: "generation-1",
  search_profile: "bge-base",
  seed: { kind: "record", record_key: location.record },
  role_recipe: "roles/v1",
  supports_corresponding_parts: true,
  parts: [0, 1, 2].map((ordinal) => ({
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
};
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
