import {
  closestCenter,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
  type DndContextProps,
} from "@dnd-kit/core";
import { arrayMove, sortableKeyboardCoordinates, useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import {
  useEffect,
  useRef,
  useState,
  type HTMLAttributes,
  type ReactNode,
} from "react";

export type DragHandleBindings = Pick<
  ReturnType<typeof useSortable>,
  "attributes" | "listeners" | "setActivatorNodeRef"
>;

export function useSortableList<T>({
  items,
  getKey,
  disabled,
  onReorder,
}: {
  items: T[];
  getKey: (item: T) => string;
  disabled: boolean;
  onReorder: (
    key: string,
    targetKey: string,
    placement: "before" | "after",
  ) => Promise<boolean>;
}) {
  const [activeKey, setActiveKey] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const pending = useRef(false);
  const keyboardHandle = useRef<HTMLElement | null>(null);
  useEffect(() => {
    if (disabled || saving || activeKey !== null) return;
    const handle = keyboardHandle.current;
    if (!handle) return;
    const frame = requestAnimationFrame(() => {
      keyboardHandle.current = null;
      // Disabling a pending handle can move focus to body. Restore keyboard
      // continuity once enabled, unless the user chose a different control.
      if (
        handle.isConnected &&
        (document.activeElement === document.body || document.activeElement === handle)
      ) {
        handle.focus({ preventScroll: true });
      }
    });
    return () => cancelAnimationFrame(frame);
  }, [disabled, saving, activeKey]);
  // Retain the dropped order through the write and subsequent canonical read.
  // A replacement snapshot retires this purely visual ordering.
  const [droppedOrder, setDroppedOrder] = useState<{
    base: T[];
    keys: string[];
  } | null>(null);
  const itemsByKey = new Map(items.map((item) => [getKey(item), item]));
  const displayedItems =
    droppedOrder?.base === items
      ? droppedOrder.keys.flatMap((key) => {
          const item = itemsByKey.get(key);
          return item === undefined ? [] : [item];
        })
      : items;
  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 6 } }),
    useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates }),
  );
  const context: DndContextProps = {
    sensors,
    collisionDetection: closestCenter,
    onDragStart: ({ active, activatorEvent }) => {
      keyboardHandle.current =
        activatorEvent instanceof KeyboardEvent &&
        document.activeElement instanceof HTMLElement
          ? document.activeElement
          : null;
      setActiveKey(String(active.id));
    },
    onDragCancel: () => {
      keyboardHandle.current = null;
      setActiveKey(null);
    },
    onDragEnd: async ({ active, over }) => {
      setActiveKey(null);
      if (disabled || pending.current || !over || active.id === over.id) return;
      const keys = items.map(getKey);
      const from = keys.indexOf(String(active.id));
      const to = keys.indexOf(String(over.id));
      if (from < 0 || to < 0) return;
      pending.current = true;
      setSaving(true);
      setDroppedOrder({ base: items, keys: arrayMove(keys, from, to) });
      try {
        if (
          !(await onReorder(
            String(active.id),
            String(over.id),
            from < to ? "after" : "before",
          ))
        ) {
          setDroppedOrder(null);
        }
      } catch {
        // The mutation owner presents the failure; restore canonical order.
        setDroppedOrder(null);
      } finally {
        pending.current = false;
        setSaving(false);
      }
    },
  };
  return {
    context,
    items: displayedItems,
    activeItem: items.find((item) => getKey(item) === activeKey),
    saving,
    disabled: disabled || saving,
  };
}

export function SortableRow({
  id,
  disabled,
  children,
  ...props
}: Omit<HTMLAttributes<HTMLDivElement>, "children" | "id"> & {
  id: string;
  disabled: boolean;
  children: (bindings: DragHandleBindings) => ReactNode;
}) {
  const {
    attributes,
    listeners,
    setNodeRef,
    setActivatorNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id, disabled });
  return (
    <div
      {...props}
      ref={setNodeRef}
      data-sortable-key={id}
      style={{
        ...props.style,
        transform: CSS.Transform.toString(transform),
        transition,
        opacity: isDragging ? 0 : undefined,
      }}
    >
      {children({ attributes, listeners, setActivatorNodeRef })}
    </div>
  );
}
