import { DndContext, DragOverlay } from "@dnd-kit/core";
import { SortableContext, verticalListSortingStrategy } from "@dnd-kit/sortable";
import { Button, Form } from "antd";
import { GripVertical } from "lucide-react";
import type {
  PersistentDamagePrecedenceEntryView,
  PersistentDamagePrecedenceView,
  PersistentDamageSourceRefView,
} from "../../generated/atlas";
import {
  SortableRow,
  useSortableList,
  type DragHandleBindings,
} from "../../shared/ui/sortable/SortableList";

const sourceKey = (source: PersistentDamageSourceRefView) =>
  source.kind === "edited" ? "edited" : source.condition_id.toString();
const entryKey = (entry: PersistentDamagePrecedenceEntryView) =>
  sourceKey(entry.source);

export function EncounterPersistentPrecedence({
  precedence,
  order,
  disabled,
  onChange,
}: {
  precedence: PersistentDamagePrecedenceView;
  order?: PersistentDamageSourceRefView[];
  disabled: boolean;
  onChange: (order: PersistentDamageSourceRefView[]) => void;
}) {
  const byKey = new Map(precedence.sources.map((entry) => [entryKey(entry), entry]));
  const entries = (order ?? precedence.sources.map((entry) => entry.source)).flatMap(
    (source) => {
      const entry = byKey.get(sourceKey(source));
      return entry ? [entry] : [];
    },
  );
  const sortable = useSortableList({
    items: entries,
    getKey: entryKey,
    disabled,
    onReorder: async (key, target, placement) => {
      const reordered = entries.filter((entry) => entryKey(entry) !== key);
      const index = reordered.findIndex((entry) => entryKey(entry) === target);
      const moved = byKey.get(key);
      if (!moved || index < 0) return false;
      reordered.splice(index + (placement === "after" ? 1 : 0), 0, moved);
      onChange(reordered.map((entry) => entry.source));
      return true;
    },
  });
  const renderEntry = (
    entry: PersistentDamagePrecedenceEntryView,
    active: boolean,
    bindings?: DragHandleBindings,
  ) => (
    <div className="encounter-persistent-precedence-entry">
      <Button
        {...bindings?.attributes}
        {...bindings?.listeners}
        ref={bindings?.setActivatorNodeRef}
        type="text"
        size="small"
        disabled={sortable.disabled}
        className="encounter-persistent-precedence-handle"
        aria-label={`Reorder ${entry.formula} ${precedence.damage_type}${entry.source.kind === "edited" ? " (this source)" : ""}`}
        icon={<GripVertical size={16} aria-hidden="true" />}
      />
      <span className="encounter-persistent-precedence-label">
        {entry.formula}
        {entry.multiplier === "normal"
          ? ""
          : entry.multiplier === "double"
            ? " ×2"
            : " ×½"}{" "}
        {precedence.damage_type}
        {entry.source.kind === "edited" && (
          <span className="encounter-condition-meta"> · This source</span>
        )}
      </span>
      {active && <span className="encounter-condition-meta">Active</span>}
    </div>
  );
  return (
    <Form.Item label="Damage precedence">
      <DndContext {...sortable.context}>
        <SortableContext
          items={sortable.items.map(entryKey)}
          strategy={verticalListSortingStrategy}
        >
          <div
            role="list"
            aria-label={`${precedence.damage_type} damage precedence`}
            className="encounter-persistent-precedence"
          >
            {sortable.items.map((entry, index) => (
              <SortableRow
                key={entryKey(entry)}
                id={entryKey(entry)}
                disabled={sortable.disabled}
                role="listitem"
              >
                {(bindings) => renderEntry(entry, index === 0, bindings)}
              </SortableRow>
            ))}
          </div>
        </SortableContext>
        <DragOverlay>
          {sortable.activeItem && (
            <div aria-hidden>{renderEntry(sortable.activeItem, false)}</div>
          )}
        </DragOverlay>
      </DndContext>
    </Form.Item>
  );
}
