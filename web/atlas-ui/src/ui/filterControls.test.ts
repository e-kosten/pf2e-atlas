import type { FilterEditorFieldView, FilterEditorView } from "../generated/atlas";
import { DEFAULT_SEARCH_STATE, type SearchFormState } from "../state/searchState";
import {
  addVisibleFilter,
  additionalFilterGroups,
  additionalVisibleFilterIds,
  booleanForField,
  controlKindForField,
  cycleSelectedValueForField,
  discoveredOptions,
  metricComparisonForField,
  rangeForField,
  removeVisibleFilter,
  setMetricComparisonForField,
  setValuesForField,
  setRangeForField,
  valueFilterOperatorPolicy,
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

  it("keeps selected unavailable fields addressable while hiding them from add menu", () => {
    const workspace = workspaceFor({
      search: {
        ...DEFAULT_SEARCH_STATE,
        visibleFilterIds: ["publication_title"],
      },
      filterEditor: {
        matching_record_count: 1n,
        groups: [
          {
            id: "source",
            label: "Source",
            fields: [
              {
                ...field("publication_title", "Publication", "addable", {
                  kind: "multi_select",
                }),
                applicability: "selected_unavailable",
              },
            ],
          },
        ],
      },
    });

    expect(additionalVisibleFilterIds(workspace)).toEqual(["publication_title"]);
    expect(additionalFilterGroups(workspace)).toEqual([]);
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

  it("cycles option fields through neutral, included, and excluded states", () => {
    const policy = valueFilterOperatorPolicy(
      field("kind", "Kinds", "always_visible", {
        kind: "multi_select",
      }),
    );
    const neutralSearch = {
      ...DEFAULT_SEARCH_STATE,
      filterClauses: [],
    };
    const withIncludedKind = cycleSelectedValueForField(
      neutralSearch,
      "kind",
      "spell",
      policy,
    );
    expect(withIncludedKind.filterClauses).toContainEqual(
      expect.objectContaining({
        field: "kind",
        operator: "include_any",
        values: expect.arrayContaining(["spell"]),
      }),
    );

    const withExcludedKind = cycleSelectedValueForField(
      setValuesForField(withIncludedKind, "kind", ["spell", "feat"], policy),
      "kind",
      "spell",
      policy,
    );

    expect(withExcludedKind.filterClauses).toContainEqual(
      expect.objectContaining({
        field: "kind",
        operator: "include_any",
        values: ["feat"],
      }),
    );
    expect(withExcludedKind.filterClauses).toContainEqual(
      expect.objectContaining({
        field: "kind",
        operator: "exclude_any",
        values: ["spell"],
      }),
    );

    const withSpellNeutral = cycleSelectedValueForField(
      withExcludedKind,
      "kind",
      "spell",
      policy,
    );
    expect(withSpellNeutral.filterClauses).not.toContainEqual(
      expect.objectContaining({
        field: "kind",
        operator: "exclude_any",
        values: expect.arrayContaining(["spell"]),
      }),
    );
  });

  it("respects backend operator policy for value fields", () => {
    const includeAllPolicy = valueFilterOperatorPolicy({
      ...field("traits", "Traits", "always_visible", { kind: "multi_select" }),
      allowed_operators: ["include_all", "include_any", "exclude_any"],
      default_operator: "include_all",
    });
    const withIncludedTrait = cycleSelectedValueForField(
      { ...DEFAULT_SEARCH_STATE, filterClauses: [] },
      "traits",
      "attack",
      includeAllPolicy,
    );
    expect(withIncludedTrait.filterClauses).toContainEqual(
      expect.objectContaining({
        field: "traits",
        operator: "include_all",
        values: ["attack"],
      }),
    );

    const includeOnlyPolicy = valueFilterOperatorPolicy({
      ...field("publication_title", "Publication", "addable", {
        kind: "multi_select",
      }),
      allowed_operators: ["include_any"],
      default_operator: "include_any",
    });
    const withIncludedPublication = cycleSelectedValueForField(
      { ...DEFAULT_SEARCH_STATE, filterClauses: [] },
      "publication_title",
      "Player Core",
      includeOnlyPolicy,
    );
    const withPublicationNeutral = cycleSelectedValueForField(
      withIncludedPublication,
      "publication_title",
      "Player Core",
      includeOnlyPolicy,
    );
    expect(withPublicationNeutral.filterClauses).not.toContainEqual(
      expect.objectContaining({
        field: "publication_title",
        operator: "exclude_any",
      }),
    );
    expect(withPublicationNeutral.filterClauses).not.toContainEqual(
      expect.objectContaining({
        field: "publication_title",
        values: expect.arrayContaining(["Player Core"]),
      }),
    );
  });

  it("removes newly included values from existing exclusions", () => {
    const policy = valueFilterOperatorPolicy(
      field("kind", "Kinds", "always_visible", {
        kind: "multi_select",
      }),
    );
    const search = {
      ...DEFAULT_SEARCH_STATE,
      filterClauses: [
        {
          id: "kind-exclude_any",
          field: "kind",
          operator: "exclude_any",
          values: ["spell", "feat"],
        },
      ],
    } satisfies SearchFormState;

    const withIncludedSpell = setValuesForField(search, "kind", ["spell"], policy);

    expect(withIncludedSpell.filterClauses).toContainEqual(
      expect.objectContaining({
        field: "kind",
        operator: "include_any",
        values: ["spell"],
      }),
    );
    expect(withIncludedSpell.filterClauses).toContainEqual(
      expect.objectContaining({
        field: "kind",
        operator: "exclude_any",
        values: ["feat"],
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
  const operator =
    control.kind === "range"
      ? "range"
      : control.kind === "metric_comparison"
        ? "metric_compare"
        : "include_any";
  return {
    id,
    label,
    control,
    placement,
    applicability: "applicable",
    allowed_operators:
      control.kind === "multi_select" ? ["include_any", "exclude_any"] : [operator],
    default_operator: operator,
    supports_counts: control.kind !== "range",
  };
}
