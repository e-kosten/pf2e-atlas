import { Button, Dropdown, Typography } from "antd";
import { Ellipsis } from "lucide-react";
import { useRef, useState, type KeyboardEvent } from "react";
import type { RecordNavigationView } from "../../generated/atlas";
import { atlasRoutePath, shouldHandleAtlasRouteClick } from "../../app/routes";
import type { RecordReferenceHandler } from "./PreparedContent";

/** Optional general-entry lookup, separate from reading the selected item. */
export function RecordProvenanceAction({
  navigation,
  title,
  onReference,
}: {
  navigation: RecordNavigationView;
  title?: string;
  onReference: RecordReferenceHandler;
}) {
  const [open, setOpen] = useState(false);
  const trigger = useRef<HTMLButtonElement>(null);
  const closeOnEscape = (event: KeyboardEvent<HTMLElement>) => {
    if (event.key !== "Escape" || !open) return;
    event.preventDefault();
    event.stopPropagation();
    setOpen(false);
    trigger.current?.focus({ preventScroll: true });
  };
  return (
    <Dropdown
      trigger={["click"]}
      placement="bottomRight"
      // eslint-disable-next-line jsx-a11y/no-autofocus
      autoFocus
      open={open}
      onOpenChange={setOpen}
      menu={{
        "aria-label": "Record information",
        onKeyDownCapture: closeOnEscape,
        onClick: ({ domEvent }) => {
          setOpen(false);
          // Menu keyboard activation targets its item rather than the nested native link.
          if (domEvent.type === "keydown" && trigger.current)
            onReference(navigation.record_key, trigger.current, navigation);
        },
        items: [
          {
            key: "general-entry",
            label: (
              <Typography.Link
                href={atlasRoutePath({
                  kind: "record",
                  recordKey: navigation.record_key,
                  selection: navigation,
                })}
                onClick={(event) => {
                  if (!shouldHandleAtlasRouteClick(event)) return;
                  event.preventDefault();
                  setOpen(false);
                  onReference(
                    navigation.record_key,
                    trigger.current ?? event.currentTarget,
                    navigation,
                  );
                }}
              >
                {title ? `General entry: ${title}` : "General entry"}
              </Typography.Link>
            ),
          },
        ],
      }}
    >
      <Button
        ref={trigger}
        type="text"
        icon={<Ellipsis size={16} aria-hidden="true" />}
        aria-label="Record information"
        title="Record information"
        aria-haspopup="menu"
        aria-expanded={open}
        onKeyDown={(event) => {
          closeOnEscape(event);
          if (event.key === "ArrowDown") {
            event.preventDefault();
            setOpen(true);
          }
        }}
      />
    </Dropdown>
  );
}
