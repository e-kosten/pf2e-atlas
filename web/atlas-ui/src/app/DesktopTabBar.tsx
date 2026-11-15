import { CloseOutlined, EllipsisOutlined, PlusOutlined } from "@ant-design/icons";
import { Button, Dropdown } from "antd";
import {
  closestCenter,
  DndContext,
  PointerSensor,
  useSensor,
  useSensors,
  type CollisionDetection,
} from "@dnd-kit/core";
import {
  horizontalListSortingStrategy,
  SortableContext,
  useSortable,
} from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import {
  useCallback,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
  type KeyboardEvent,
} from "react";

type TabLabel = { id: string; title: string };

export const desktopTabButtonId = (id: string) => `desktop-tab-${id}`;
export const desktopTabPanelId = (id: string) => `desktop-panel-${id}`;

export function DesktopTabBar({
  tabs,
  activeId,
  select,
  close,
  open,
  moveBy,
  moveTo,
}: {
  tabs: TabLabel[];
  activeId: string;
  select: (id: string) => void;
  close: (id: string) => boolean;
  open: () => void;
  moveBy: (id: string, offset: number) => void;
  moveTo: (id: string, target: string) => void;
}) {
  const viewport = useRef<HTMLDivElement>(null);
  const row = useRef<HTMLDivElement>(null);
  const more = useRef<HTMLSpanElement>(null);
  const [overflowing, setOverflowing] = useState(false);
  const [moreOpen, setMoreOpen] = useState(false);
  const [hiddenIds, setHiddenIds] = useState<string[]>([]);
  const ids = tabs.map((tab) => tab.id);
  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 6 } }),
  );
  const canScroll = useCallback((element: Element) => element === viewport.current, []);

  const collisionDetection: CollisionDetection = (args) => {
    const view = viewport.current;
    const pointer = args.pointerCoordinates;
    if (view && pointer && view.scrollWidth > view.clientWidth) {
      const bounds = view.getBoundingClientRect();
      // At the ends, a wide tab can reach the edge before its center reaches
      // the first or last slot, depending on where the pointer grabbed it.
      if (view.scrollLeft <= 1 && pointer.x <= bounds.left + 8) return [{ id: ids[0] }];
      if (
        view.scrollLeft >= view.scrollWidth - view.clientWidth - 1 &&
        pointer.x >= bounds.right - 8
      )
        return [{ id: ids[ids.length - 1] }];
    }
    return closestCenter(args);
  };

  const reveal = useCallback((id: string) => {
    const view = viewport.current;
    const tab = document.getElementById(desktopTabButtonId(id))?.parentElement;
    if (!view || !tab) return;
    const bounds = view.getBoundingClientRect();
    const card = tab.getBoundingClientRect();
    if (card.left < bounds.left) view.scrollLeft += card.left - bounds.left;
    else if (card.right > bounds.right) view.scrollLeft += card.right - bounds.right;
  }, []);

  useLayoutEffect(() => {
    const view = viewport.current!;
    const list = row.current!;
    const measure = () => {
      const available = view.clientWidth + (more.current?.offsetWidth ?? 0);
      const overflow = list.offsetWidth > available;
      setOverflowing(overflow);
      if (!overflow) setMoreOpen(false);
    };
    const observer = new ResizeObserver(measure);
    observer.observe(view);
    observer.observe(list);
    measure();
    return () => observer.disconnect();
  }, []);

  useLayoutEffect(() => reveal(activeId), [activeId, overflowing, reveal]);

  useEffect(() => {
    const view = viewport.current!;
    function onWheel(event: WheelEvent) {
      // A mouse wheel over the tab strip scrolls through tabs as a trackpad does.
      if (event.ctrlKey || event.deltaX || !event.deltaY) return;
      const unit =
        event.deltaMode === WheelEvent.DOM_DELTA_LINE
          ? 16
          : event.deltaMode === WheelEvent.DOM_DELTA_PAGE
            ? view.clientWidth
            : 1;
      const before = view.scrollLeft;
      view.scrollLeft += event.deltaY * unit;
      if (view.scrollLeft !== before) event.preventDefault();
    }
    view.addEventListener("wheel", onWheel, { passive: false });
    return () => view.removeEventListener("wheel", onWheel);
  }, []);

  function focusTab(id: string) {
    document.getElementById(desktopTabButtonId(id))?.focus({ preventScroll: true });
    reveal(id);
  }

  function onTabKey(event: KeyboardEvent<HTMLButtonElement>, index: number) {
    const { key, altKey, shiftKey, ctrlKey, metaKey } = event;
    const offset = key === "ArrowLeft" ? -1 : key === "ArrowRight" ? 1 : 0;
    if (ctrlKey || metaKey) return;
    const id = ids[index];
    if (offset && altKey && shiftKey) {
      event.preventDefault();
      event.stopPropagation();
      moveBy(id, offset);
      requestAnimationFrame(() => focusTab(id));
    } else if (!altKey && !shiftKey) {
      if (offset || key === "Home" || key === "End") {
        event.preventDefault();
        const next =
          key === "Home"
            ? 0
            : key === "End"
              ? ids.length - 1
              : (index + offset + ids.length) % ids.length;
        focusTab(ids[next]);
      } else if (key === "Delete" || key === "Backspace") {
        event.preventDefault();
        if (close(id) && id !== activeId && ids.length > 2)
          focusTab(ids[index + 1] ?? ids[index - 1]);
      }
    }
  }

  function findHiddenTabs() {
    const bounds = viewport.current!.getBoundingClientRect();
    setHiddenIds(
      ids.filter((id) => {
        const card = document.getElementById(desktopTabButtonId(id))?.parentElement;
        const rect = card?.getBoundingClientRect();
        return rect && (rect.left < bounds.left - 1 || rect.right > bounds.right + 1);
      }),
    );
  }

  return (
    <div className="desktop-tab-bar">
      <div className="desktop-tab-viewport" ref={viewport}>
        <DndContext
          sensors={sensors}
          autoScroll={{
            canScroll,
            threshold: { x: 0.065, y: 0.2 },
            acceleration: 4.5,
            interval: 8,
          }}
          collisionDetection={collisionDetection}
          onDragEnd={({ active, over }) => {
            if (over) moveTo(String(active.id), String(over.id));
          }}
        >
          <SortableContext items={ids} strategy={horizontalListSortingStrategy}>
            <div
              className="desktop-tab-row"
              ref={row}
              role="tablist"
              aria-label="Atlas tabs"
            >
              {tabs.map((tab, index) => (
                <SortableTab
                  key={tab.id}
                  tab={tab}
                  selected={tab.id === activeId}
                  select={() => select(tab.id)}
                  close={() => close(tab.id)}
                  onKeyDown={(event) => onTabKey(event, index)}
                />
              ))}
            </div>
          </SortableContext>
        </DndContext>
      </div>
      {overflowing && (
        <span ref={more} className="desktop-tab-action">
          <Dropdown
            trigger={["click"]}
            placement="bottomRight"
            // eslint-disable-next-line jsx-a11y/no-autofocus -- Focus enters the menu after the user opens it.
            autoFocus
            open={moreOpen}
            onOpenChange={(open) => {
              if (open) findHiddenTabs();
              setMoreOpen(open);
            }}
            menu={{
              items: tabs
                .filter(({ id }) => hiddenIds.includes(id))
                .map(({ id, title }) => ({
                  key: id,
                  label: title,
                  extra: (
                    <Button
                      type="text"
                      size="small"
                      aria-label={`Close ${title} tab`}
                      icon={<CloseOutlined />}
                      onClick={(event) => {
                        event.stopPropagation();
                        if (close(id)) {
                          setMoreOpen(false);
                          more.current
                            ?.querySelector("button")
                            ?.focus({ preventScroll: true });
                        }
                      }}
                      onKeyDown={(event) => {
                        if (event.key === "Enter" || event.key === " ")
                          event.stopPropagation();
                      }}
                    />
                  ),
                })),
              onClick: ({ key }) => {
                setMoreOpen(false);
                select(key);
                focusTab(key);
              },
            }}
          >
            <Button
              aria-label="More tabs"
              aria-haspopup="menu"
              aria-expanded={moreOpen}
              icon={<EllipsisOutlined />}
            />
          </Dropdown>
        </span>
      )}
      <span className="desktop-tab-action">
        <Button aria-label="New tab" icon={<PlusOutlined />} onClick={open} />
      </span>
    </div>
  );
}

function SortableTab({
  tab: { id, title },
  selected,
  select,
  close,
  onKeyDown,
}: {
  tab: TabLabel;
  selected: boolean;
  select: () => void;
  close: () => void;
  onKeyDown: (event: KeyboardEvent<HTMLButtonElement>) => void;
}) {
  const { listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    id,
  });
  return (
    <div
      ref={setNodeRef}
      className="desktop-tab-card"
      data-selected={selected}
      style={{
        transform: transform
          ? CSS.Translate.toString({ ...transform, y: 0 })
          : undefined,
        transition,
        zIndex: isDragging ? 1 : undefined,
      }}
    >
      <button
        id={desktopTabButtonId(id)}
        className="desktop-tab-label"
        type="button"
        role="tab"
        aria-selected={selected}
        aria-controls={desktopTabPanelId(id)}
        tabIndex={selected ? 0 : -1}
        onClick={select}
        onMouseDown={(event) => {
          if (event.button === 1) event.preventDefault();
        }}
        onAuxClick={(event) => {
          if (event.button === 1) close();
        }}
        onKeyDown={onKeyDown}
        onPointerDown={(event) => listeners?.onPointerDown?.(event)}
      >
        {title}
      </button>
      <Button
        type="text"
        size="small"
        aria-label="Close tab"
        icon={<CloseOutlined />}
        tabIndex={selected ? 0 : -1}
        onClick={close}
      />
    </div>
  );
}
