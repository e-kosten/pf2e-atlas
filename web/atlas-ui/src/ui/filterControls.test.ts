import type { FilterEditorFieldView, FilterEditorView } from "../generated/atlas";
import { DEFAULT_SEARCH_STATE, type SearchFormState } from "../state/searchState";
import {
  addVisibleFilter,
  additionalFilterGroups,
  additionalVisibleFilterIds,
  booleanForField,
  controlKindForField,
  discoveredOptions,
  metricComparisonForField,
  rangeForField,
  removeVisibleFilter,
  setMetricComparisonForField,
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
        filterClauses: [
          ...DEFAULT_SEARCH_STATE.filterClauses,
          {
            id: "basic_save-include_any",
            field: "basic_save",
            operator: "include_any",
            values: ["true"],
          },
          {
            id: "pack-include_any",
            field: "pack",
            operator: "include_any",
            values: ["Actions"],
          },
        ],
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
      {
        label: "Metrics",
        options: [{ value: "metric", label: "Metric" }],
      },
    ]);

    removeVisibleFilter(workspace, "pack");
    expect(setSearch).toHaveBeenLastCalledWith(
      expect.objectContaining({
        hiddenFilterIds: ["pack"],
        filterClauses: expect.not.arrayContaining([
          expect.objectContaining({ field: "pack" }),
        ]),
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
    expect(controlKindForField(workspace, "metric")).toBe("metric");
    expect(controlKindForField(workspace, "kind")).toBe("option");
    expect(discoveredOptions(workspace, "kind")).toEqual([
      { value: "spell", label: "Spell (2)", disabled: false },
    ]);
    expect(discoveredOptions(workspace, "rarity")).toEqual([]);
  });

  it("keeps range, boolean, and metric values in filter clauses", () => {
    const withLevel = setRangeForField(DEFAULT_SEARCH_STATE, "level", {
      min: 2,
      max: 6,
    });
    expect(rangeForField(withLevel, "level")).toEqual({ min: 2, max: 6 });
    expect(withLevel.filterClauses).toContainEqual(
      expect.objectContaining({
        field: "level",
        operator: "range",
        range: { min: 2, max: 6 },
      }),
    );

    const withBulk = setRangeForField(DEFAULT_SEARCH_STATE, "bulk_value", {
      min: 1,
      max: 3,
    });
    expect(withBulk.filterClauses).toContainEqual(
      expect.objectContaining({
        field: "bulk_value",
        operator: "range",
        range: { min: 1, max: 3 },
      }),
    );
    expect(booleanForField(withBulk, "publication_remaster")).toBeNull();

    const withMetric = setMetricComparisonForField(DEFAULT_SEARCH_STATE, "metric", {
      key: "spell.area.value",
      op: "gte",
      value: 10,
    });
    expect(metricComparisonForField(withMetric, "metric")).toEqual({
      key: "spell.area.value",
      op: "gte",
      value: 10,
    });
    expect(withMetric.filterClauses).toContainEqual(
      expect.objectContaining({
        field: "metric",
        operator: "metric_compare",
        metric: { key: "spell.area.value", op: "gte", value: 10 },
      }),
    );
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
        id: "metrics",
        label: "Metrics",
        fields: [
          field("metric", "Metric", "addable", {
            kind: "metric_comparison",
            key_label: "Metric",
            operator_label: "Operator",
            value_label: "Value",
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
