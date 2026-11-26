import type {
  FilterChoiceView,
  FilterEditorFieldView,
  QueryFieldDefinition,
} from "../../generated/atlas";

export function dynamicChoiceGroup(field: QueryFieldDefinition, value: string) {
  return field.choice_groups?.find(
    ({ dynamic_child_prefix }) =>
      dynamic_child_prefix &&
      value.startsWith(dynamic_child_prefix) &&
      value.length > dynamic_child_prefix.length &&
      !/\s/u.test(value),
  );
}

/** Keep a selection from another library editable with its catalog-declared group. */
export function kindChoicesWithSelection(
  field: FilterEditorFieldView,
  selected: string[],
): FilterChoiceView[] {
  const observed = new Set(field.choices.map((choice) => String(choice.value)));
  const missing = [...new Set(selected)].filter((value) => !observed.has(value));
  if (!missing.length) return field.choices;
  return [
    ...field.choices,
    ...missing.map((value) => {
      const group = dynamicChoiceGroup(field.definition, value);
      return {
        value,
        label: group?.dynamic_child_prefix
          ? value.slice(group.dynamic_child_prefix.length)
          : value,
        parent_kind: group?.id,
      };
    }),
  ];
}

type KindChoice = {
  value: string;
  label: string;
  parent?: string;
  searchAliases?: string[];
};

/** Build a two-level display from the catalog. An omitted expansion set opens every group. */
export function kindChoiceTree<T extends KindChoice>(
  choices: T[],
  search: string,
  expansion?: ReadonlySet<string>,
) {
  const children = new Map<string, T[]>();
  const parents = new Map<string, string>();
  for (const choice of choices) {
    if (!choice.parent) continue;
    parents.set(choice.value, choice.parent);
    const siblings = children.get(choice.parent);
    if (siblings) siblings.push(choice);
    else children.set(choice.parent, [choice]);
  }
  const term = search.trim().toLocaleLowerCase();
  const matches = (choice: T) =>
    choice.label.toLocaleLowerCase().includes(term) ||
    choice.searchAliases?.some((alias) => alias.toLocaleLowerCase().includes(term));
  const rows: (T & { expanded?: boolean })[] = [];
  for (const root of choices) {
    if (root.parent) continue;
    const group = children.get(root.value) ?? [];
    const matchingChildren = matches(root) ? group : group.filter(matches);
    if (!matches(root) && !matchingChildren.length) continue;
    const expanded = !!term || !expansion || expansion.has(root.value);
    rows.push(group.length && !term ? { ...root, expanded } : root);
    if (expanded) rows.push(...matchingChildren);
  }
  return { rows, parents };
}
