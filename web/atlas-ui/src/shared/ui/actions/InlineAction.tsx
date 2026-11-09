import { Button, Tooltip } from "antd";
import {
  useEffect,
  useRef,
  useState,
  type ComponentPropsWithRef,
  type ReactNode,
} from "react";
import "../../../styles/ui/inline-action.css";

type InlineActionProps = Omit<
  ComponentPropsWithRef<typeof Button>,
  "aria-label" | "children" | "icon" | "type" | "size"
> & {
  label: string;
  icon: ReactNode;
  tooltip?: boolean;
};

function InlineAction({
  label,
  icon,
  tooltip = true,
  title,
  className,
  onClick,
  ...buttonProps
}: InlineActionProps) {
  const [captionOpen, setCaptionOpen] = useState(false);
  const trigger = useRef<HTMLElement | null>(null);
  useEffect(() => {
    if (!captionOpen || !tooltip) return;
    const dismiss = (event: KeyboardEvent) => {
      if (event.key !== "Escape") return;
      event.preventDefault();
      event.stopPropagation();
      setCaptionOpen(false);
    };
    const owner = trigger.current?.ownerDocument ?? document;
    const outside = (event: PointerEvent) => {
      if (!trigger.current?.contains(event.target as Node)) setCaptionOpen(false);
    };
    owner.addEventListener("keydown", dismiss, true);
    owner.addEventListener("pointerdown", outside, true);
    return () => {
      owner.removeEventListener("keydown", dismiss, true);
      owner.removeEventListener("pointerdown", outside, true);
    };
  }, [captionOpen, tooltip]);
  const action = (
    <Button
      {...buttonProps}
      className={["inline-icon-action", className].filter(Boolean).join(" ")}
      type="text"
      aria-label={label}
      icon={icon}
      title={tooltip ? undefined : title}
      onFocus={(event) => {
        trigger.current = event.currentTarget;
        buttonProps.onFocus?.(event);
      }}
      onMouseEnter={(event) => {
        trigger.current = event.currentTarget;
        buttonProps.onMouseEnter?.(event);
      }}
      onClick={(event) => {
        trigger.current = event.currentTarget;
        if (onClick || buttonProps.href !== undefined) {
          setCaptionOpen(false);
          onClick?.(event);
        } else if (tooltip) {
          event.currentTarget.focus({ preventScroll: true });
          setCaptionOpen(true);
        }
      }}
    />
  );
  return tooltip ? (
    <Tooltip
      title={title ?? label}
      trigger={["hover", "focus"]}
      open={captionOpen}
      onOpenChange={setCaptionOpen}
    >
      {action}
    </Tooltip>
  ) : (
    action
  );
}

/** Ant's native control and overlay events pass through; this owner supplies inline geometry. */
export function InlineIconButton(props: Omit<InlineActionProps, "href">) {
  return <InlineAction {...props} />;
}
export function InlineIconLink(props: InlineActionProps & { href: string }) {
  return <InlineAction {...props} />;
}

/** Attach annotations to the final literal fragment without duplicating linked elements. */
export function InlineContext({
  children,
  actions,
}: {
  children: ReactNode;
  actions: ReactNode;
}) {
  if (!actions) return children;
  if (typeof children === "string") {
    const match = /^(.*\s)?(\S+\s*)$/su.exec(children);
    const leading = match?.[1] ?? "";
    const trailing = match?.[2] ?? children;
    return (
      <>
        {leading}
        <span className="inline-context-tail">
          <span className="inline-context-content">{trailing}</span>
          <span className="inline-context-actions">{actions}</span>
        </span>
      </>
    );
  }
  return (
    <span className="inline-context-tail inline-context-tail--rich">
      <span className="inline-context-content">{children}</span>
      <span className="inline-context-actions">{actions}</span>
    </span>
  );
}

const attachments = new WeakMap<Element, { target: HTMLElement; group: HTMLElement }>();

function restoreMovedFocus(owner: Document, focused: Element | null) {
  if (
    owner.activeElement === owner.body &&
    focused instanceof HTMLElement &&
    focused.isConnected
  ) {
    focused.focus({ preventScroll: true });
  }
}

/** Attach a portal to authored DOM using the same layout as InlineContext. */
export function attachInlineAction(target: HTMLElement, host: HTMLElement) {
  const doc = target.ownerDocument;
  const focused = doc.activeElement;
  const existing = target.parentElement?.nextElementSibling;
  if (existing && attachments.get(existing)?.target === target) {
    if (host.parentElement !== existing) {
      detachInlineAction(host);
      existing.append(host);
      restoreMovedFocus(doc, focused);
    }
    return;
  }

  detachInlineAction(host);
  const group = doc.createElement("span");
  const content = doc.createElement("span");
  const actions = doc.createElement("span");
  group.className = "inline-context-tail inline-context-tail--rich";
  content.className = "inline-context-content";
  actions.className = "inline-context-actions";
  attachments.set(actions, { target, group });
  target.before(group);
  group.append(content, actions);
  content.append(target);
  actions.append(host);
  restoreMovedFocus(doc, focused);
}

export function detachInlineAction(host: HTMLElement) {
  const actions = host.parentElement;
  const attachment = actions && attachments.get(actions);
  host.remove();
  if (!actions || !attachment || actions.childElementCount) return;

  attachments.delete(actions);
  const { target, group } = attachment;
  const doc = target.ownerDocument;
  const focused = doc.activeElement;
  group.replaceWith(target);
  restoreMovedFocus(doc, focused);
}
