import type { FilterEditorFieldView, FilterEditorView } from "../generated/atlas";
import { DEFAULT_SEARCH_STATE, type SearchFormState } from "../state/searchState";
import {
  addVisibleFilter,
  additionalFilterGroups,
  additionalVisibleFilterIds,
  booleanForField,
  controlKindForField,
  discoveredOptions,
  rangeForField,
  removeVisibleFilter,
  setRangeForField,
  visibleEditorFilterFields,
} from "./filterControls";
import type { AtlasWorkspaceState } from "./useAtlasWorkspace";

describe("filterControls", () => {
  it("derives default, addable, and removable filters from editor placement", () => {
    const setSearch = vi.fn();
    const workspace = workspaceFor({
      search: {
        ...DEFAULT_SEARCH_STATE,
        visibleFilterIds: ["basic_save"],
        optionFilters: { basic_save: ["true"], pack: ["Actions"] },
      },
      setSearch,
    });

    expect(visibleEditorFilterFields(workspace).map((field) => field.id)).toEqual([
      "kind",
      "level",
      "pack",
    ]);
    expect(additionalVisibleFilterIds(workspace)).toEqual(["basic_save"]);
    expect(additionalFilterGroups(workspace)).toEqual([
      {
        label: "Source",
        options: [
          { value: "publication_title", label: "Publication" },
          { value: "publication_remaster", label: "Remaster" },
        ],
      },
    ]);

    removeVisibleFilter(workspace, "pack");
    expect(setSearch).toHaveBeenLastCalledWith(
      expect.objectContaining({
        hiddenFilterIds: ["pack"],
        optionFilters: { basic_save: ["true"] },
      }),
    );

    const hiddenWorkspace = workspaceFor({
      search: {
        ...DEFAULT_SEARCH_STATE,
        hiddenFilterIds: ["pack"],
      },
      setSearch,
    });
    expect(visibleEditorFilterFields(hiddenWorkspace).map((field) => field.id)).toEqual(
      ["kind", "level"],
    );
    expect(additionalFilterGroups(hiddenWorkspace)[0]?.options).toContainEqual({
      value: "pack",
      label: "Pack",
    });

    addVisibleFilter(hiddenWorkspace, "pack");
    expect(setSearch).toHaveBeenLastCalledWith(
      expect.objectContaining({
        hiddenFilterIds: [],
        visibleFilterIds: [],
      }),
    );
  });

  it("uses backend control shapes and discovered values without static fallback", () => {
    const workspace = workspaceFor({
      filterValuesByField: {
        kind: {
          field_id: "kind",
          matching_record_count: 2n,
          options: [
            {
              value: "spell",
              label: "Spell",
              count: 2n,
              selected: false,
              disabled: false,
              status: "available",
            },
          ],
        },
      },
    });

    expect(controlKindForField(workspace, "level")).toBe("range");
    expect(controlKindForField(workspace, "publication_remaster")).toBe("boolean");
    expect(controlKindForField(workspace, "kind")).toBe("option");
    expect(discoveredOptions(workspace, "kind")).toEqual([
      { value: "spell", label: "Spell (2)", disabled: false },
    ]);
    expect(discoveredOptions(workspace, "rarity")).toEqual([]);
  });

  it("keeps level range in named state and generic ranges in range filters", () => {
    const withLevel = setRangeForField(DEFAULT_SEARCH_STATE, "level", {
      min: 2,
      max: 6,
    });
    expect(withLevel.levelMin).toBe(2);
    expect(withLevel.levelMax).toBe(6);
    expect(rangeForField(withLevel, "level")).toEqual({ min: 2, max: 6 });

    const withBulk = setRangeForField(DEFAULT_SEARCH_STATE, "bulk_value", {
      min: 1,
      max: 3,
    });
    expect(withBulk.rangeFilters.bulk_value).toEqual({ min: 1, max: 3 });
    expect(booleanForField(withBulk, "publication_remaster")).toBeNull();
  });
});

function workspaceFor(
  overrides: {
    search?: SearchFormState;
    setSearch?: (next: SearchFormState) => void;
    filterEditor?: FilterEditorView;
    filterValuesByField?: AtlasWorkspaceState["filterValuesByField"];
  } = {},
): AtlasWorkspaceState {
  return {
    search: overrides.search ?? DEFAULT_SEARCH_STATE,
    setSearch: overrides.setSearch ?? vi.fn(),
    filterEditor: overrides.filterEditor ?? editor(),
    filterValuesByField: overrides.filterValuesByField ?? {},
  } as AtlasWorkspaceState;
}

function editor(): FilterEditorView {
  return {
    matching_record_count: 4n,
    groups: [
      {
        id: "standard",
        label: "Standard",
        fields: [
          field("kind", "Kinds", "always_visible", { kind: "multi_select" }),
          field("level", "Level", "always_visible", {
            kind: "range",
            min_label: "Min",
            max_label: "Max",
            min: 0,
            max: 30,
            step: 1,
          }),
        ],
      },
      {
        id: "source",
        label: "Source",
        fields: [
          field("pack", "Pack", "initially_visible", { kind: "multi_select" }),
          field("publication_title", "Publication", "addable", {
            kind: "multi_select",
          }),
          field("publication_remaster", "Remaster", "addable", {
            kind: "boolean",
            true_label: "Yes",
            false_label: "No",
          }),
        ],
      },
      {
        id: "spells",
        label: "Spells",
        fields: [
          field("basic_save", "Basic Save", "addable", {
            kind: "boolean",
            true_label: "Yes",
            false_label: "No",
          }),
        ],
      },
    ],
  };
}

function field(
  id: string,
  label: string,
  placement: FilterEditorFieldView["placement"],
  control: FilterEditorFieldView["control"],
): FilterEditorFieldView {
  return {
    id,
    label,
    control,
    placement,
    allowed_operators: ["include_any"],
    default_operator: "include_any",
    supports_counts: control.kind !== "range",
  };
}
