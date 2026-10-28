import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { Button, Empty, Spin, theme } from "antd";
import { DndContext, DragOverlay } from "@dnd-kit/core";
import { SortableContext, verticalListSortingStrategy } from "@dnd-kit/sortable";
import { GripVertical, Trash2 } from "lucide-react";
import {
  SortableRow,
  useSortableList,
  type DragHandleBindings,
} from "../../shared/ui/sortable/SortableList";
import type { SavedListItemView } from "../../generated/atlas";
import { DangerActionButton } from "../../shared/ui/actions/DangerActionButton";
import { RecordBrowseSummary } from "../../shared/records/RecordBrowseSummary";

export function ListItemsPane({
  error,
  reorderError,
  mutating = false,
  items,
  loading,
  onRemove,
  onReorder,
  onSelect,
  itemHref,
  removingKey,
  selectedRecordKey,
  filtered = false,
  unavailable = false,
}: {
  error?: Error | null;
  reorderError?: Error | null;
  mutating?: boolean;
  items: SavedListItemView[];
  loading: boolean;
  onRemove: (recordKey: string) => void | Promise<unknown>;
  onReorder?: (
    recordKey: string,
    targetRecordKey: string,
    placement: "before" | "after",
  ) => Promise<boolean>;
  onSelect: (recordKey: string) => void;
  itemHref?: (recordKey: string) => string;
  removingKey: string | null;
  selectedRecordKey: string | null;
  filtered?: boolean;
  unavailable?: boolean;
}) {
  const { token } = theme.useToken();
  const sortable = useSortableList({
    items,
    getKey: (item) => item.record_key,
    disabled: loading || mutating || removingKey !== null || filtered || !onReorder,
    onReorder: onReorder ?? (() => Promise.resolve(false)),
  });
  const disabled = loading || mutating || removingKey !== null || sortable.saving;
  const renderItem = (
    item: SavedListItemView,
    reorderEnabled: boolean,
    bindings?: DragHandleBindings,
    overlay = false,
  ) => {
    const selected = item.record_key === selectedRecordKey;
    const title = item.record?.title ?? item.snapshot.title;
    return (
      <div
        className={`reference-result list-member${overlay ? " list-member__drag-preview" : ""}`}
        style={{
          background: selected ? token.colorPrimaryBg : undefined,
          borderInlineStart: `3px solid ${selected ? token.colorPrimaryBorder : "transparent"}`,
        }}
      >
        {reorderEnabled && (
          <Button
            {...bindings?.attributes}
            {...bindings?.listeners}
            ref={bindings?.setActivatorNodeRef}
            type="text"
            size="small"
            disabled={disabled || overlay}
            className="list-member__drag-handle"
            aria-label={`Reorder ${title}`}
            title="Drag to reorder. With a keyboard, press Space then use the arrow keys."
            icon={<GripVertical size={16} aria-hidden="true" />}
          />
        )}
        {item.status !== "unresolved" && item.record ? (
          <RecordBrowseSummary
            record={item.record}
            href={itemHref?.(item.record_key)}
            selected={selected}
            onPreview={() => onSelect(item.record_key)}
          />
        ) : (
          <div className="list-member__unresolved" aria-disabled="true">
            <strong>{title}</strong>
            <span>{item.snapshot.kind}</span>
            <span>Record unavailable in this reference collection</span>
          </div>
        )}
        <DangerActionButton
          className="list-member__remove"
          type="text"
          confirmation="popover"
          aria-label={`Remove ${title}`}
          title="Remove"
          icon={<Trash2 size={15} />}
          disabled={disabled || overlay}
          loading={removingKey === item.record_key}
          confirmTitle="Remove from list?"
          confirmOkText="Remove"
          onConfirm={() => onRemove(item.record_key)}
        />
      </div>
    );
  };

  return (
    <section
      className="list-items"
      aria-label="List items"
      aria-busy={loading || mutating}
      data-sorting={Boolean(sortable.activeItem) || sortable.saving || mutating}
    >
      {error && (
        <ErrorNotice
          error={error}
          title="Unable to remove record"
          description="Use the record’s Remove button to try again."
        />
      )}
      {reorderError && (
        <ErrorNotice error={reorderError} title="Unable to reorder records" />
      )}
      {loading && items.length > 0 && (
        <div className="list-items__refresh" aria-label="Updating saved records">
          <Spin size="small" delay={500} />
        </div>
      )}
      {loading && !items.length ? (
        <Spin className="reference-loading" aria-label="Loading saved records" />
      ) : !items.length ? (
        <Empty
          image={Empty.PRESENTED_IMAGE_SIMPLE}
          description={
            unavailable
              ? "Saved records unavailable"
              : filtered
                ? "No saved records match these filters"
                : "No saved records. Add records from search."
          }
        />
      ) : filtered ? (
        items.map((item) => <div key={item.record_key}>{renderItem(item, false)}</div>)
      ) : (
        <DndContext {...sortable.context}>
          <SortableContext
            items={sortable.items.map((item) => item.record_key)}
            strategy={verticalListSortingStrategy}
          >
            {sortable.items.map((item) => (
              <SortableRow
                key={item.record_key}
                id={item.record_key}
                disabled={sortable.disabled}
              >
                {(bindings) => renderItem(item, Boolean(onReorder), bindings)}
              </SortableRow>
            ))}
          </SortableContext>
          <DragOverlay>
            {sortable.activeItem && (
              <div aria-hidden>
                {renderItem(sortable.activeItem, true, undefined, true)}
              </div>
            )}
          </DragOverlay>
        </DndContext>
      )}
    </section>
  );
}
