import DOMPurify from "dompurify";
import { Alert, Modal } from "antd";
import {
  useMemo,
  useLayoutEffect,
  useRef,
  useState,
  useCallback,
  type ReactPortal,
} from "react";
import { createPortal } from "react-dom";
import { EffectReferenceLink } from "./EffectReferenceLink";
import type {
  PreparedContentFieldView,
  RecordRelationshipView,
  RecordNavigationView,
} from "../../generated/atlas";
import { atlasRoutePath, shouldHandleAtlasRouteClick } from "../../app/routes";
export type RecordReferenceHandler = (
  recordKey: string,
  anchor?: HTMLElement,
  navigation?: RecordNavigationView,
  landing?: RecordNavigationView,
) => void;
const NO_RELATIONSHIPS: RecordRelationshipView[] = [];
export function PreparedContent({
  content,
  relationships = NO_RELATIONSHIPS,
  onReference,
}: {
  content: PreparedContentFieldView;
  relationships?: RecordRelationshipView[];
  onReference: RecordReferenceHandler;
}) {
  const [modal, contextHolder] = Modal.useModal();
  const container = useRef<HTMLDivElement>(null);
  const [effectLinks, setEffectLinks] = useState<ReactPortal[]>([]);
  const referenceHandler = useRef(onReference);
  useLayoutEffect(() => {
    referenceHandler.current = onReference;
  }, [onReference]);
  const openReference = useCallback<RecordReferenceHandler>(
    (...args) => referenceHandler.current(...args),
    [],
  );
  const body = content.body;
  const html = useMemo(
    () =>
      body.kind === "html"
        ? DOMPurify.sanitize(body.html, {
            FORBID_TAGS: ["script", "iframe", "object", "embed", "form"],
            FORBID_ATTR: ["style"],
            ALLOW_DATA_ATTR: true,
          })
        : "",
    [body],
  );
  // Native links must exist before parent reading hosts restore a bookmark.
  useLayoutEffect(() => {
    if (body.kind !== "html") return;
    const markers = container.current?.querySelectorAll<HTMLElement>(
      "[data-atlas-reference],[data-atlas-interaction]",
    );
    const listeners: Array<() => void> = [];
    const effects: ReactPortal[] = [];
    markers?.forEach((originalMarker) => {
      let marker = originalMarker;
      const ref = marker.getAttribute("data-atlas-reference");
      const fact =
        ref === null
          ? null
          : relationships.find(
              (r) =>
                r.ordinal === Number(ref) &&
                JSON.stringify(r.source) === JSON.stringify(content.locator),
            );
      const control =
        ref === null
          ? body.controls.find(
              (c) =>
                c.ordinal === Number(marker.getAttribute("data-atlas-interaction")),
            )?.control
          : null;
      if (fact?.target && fact.effect_label) {
        const host = document.createElement("span");
        host.setAttribute("data-atlas-reference", String(ref));
        marker.replaceWith(host);
        effects.push(
          createPortal(
            <EffectReferenceLink
              label={fact.effect_label}
              target={fact.target}
              onReference={openReference}
            />,
            host,
            String(ref),
          ),
        );
        listeners.push(() => host.replaceWith(originalMarker));
        return;
      }
      if (
        fact?.url &&
        (fact.status === "resolved_url" || fact.status === "unverified_url") &&
        marker.tagName === "A" &&
        DOMPurify.isValidAttribute("a", "href", fact.url)
      ) {
        marker.setAttribute("href", fact.url);
        marker.removeAttribute("role");
        marker.removeAttribute("tabindex");
        return;
      }
      if (!fact?.target && (!control || control.kind === "command")) {
        marker.removeAttribute("href");
        marker.removeAttribute("role");
        marker.removeAttribute("tabindex");
        if (ref !== null) {
          marker.setAttribute("aria-disabled", "true");
          marker.setAttribute("title", "Reference unavailable");
        }
        return;
      }
      const readingTarget = fact?.contextual_target ?? fact?.target;
      if (readingTarget) {
        if (fact?.contextual_target)
          marker.setAttribute("title", "Spell from this item");
        marker.removeAttribute("aria-disabled");
        if (marker.tagName !== "A") {
          const link = document.createElement("a");
          for (const attribute of marker.attributes)
            link.setAttribute(attribute.name, attribute.value);
          link.append(...marker.childNodes);
          marker.replaceWith(link);
          marker = link;
        }
        marker.setAttribute(
          "href",
          atlasRoutePath({
            kind: "record",
            recordKey: readingTarget.record_key,
            selection: readingTarget,
          }),
        );
        marker.removeAttribute("role");
      } else marker.setAttribute("role", "button");
      marker.tabIndex = 0;
      const ownsEvent = (e: Event) => {
        const target = e.target as Element;
        const link = target.closest("a[href]");
        return (
          target.closest("[data-atlas-reference],[data-atlas-interaction]") ===
            marker &&
          (!link || link === marker || !marker.contains(link))
        );
      };
      const activate = (e: Event) => {
        if (!ownsEvent(e)) return;
        const marker = (e.target as Element).closest(
          "[data-atlas-reference],[data-atlas-interaction]",
        );
        if (!marker || !container.current?.contains(marker)) return;
        if (fact?.target && e instanceof MouseEvent && !shouldHandleAtlasRouteClick(e))
          return;
        e.preventDefault();
        const reference = marker.getAttribute("data-atlas-reference");
        if (reference !== null) {
          const ordinal = Number(reference);
          const fact = relationships.find(
            (r) =>
              r.ordinal === ordinal &&
              JSON.stringify(r.source) === JSON.stringify(content.locator),
          );
          if (!fact?.target) return;
          const target = fact.contextual_target ?? fact.target;
          openReference(target.record_key, marker as HTMLElement, target);
          return;
        }
        const ordinal = Number(marker.getAttribute("data-atlas-interaction"));
        const control = body.controls.find((c) => c.ordinal === ordinal)?.control;
        if (!control || control.kind === "command") return;
        const title =
          control.kind === "check"
            ? control.statistic || "Check"
            : control.kind === "damage"
              ? control.formula
              : control.shape || "Template";
        modal.info({
          title,
          content: (
            <dl>
              {Object.entries(control.options).map(([key, value]) => (
                <div key={key}>
                  <dt>{key}</dt>
                  <dd>{value}</dd>
                </div>
              ))}
            </dl>
          ),
        });
      };
      const keyboard = (e: KeyboardEvent) => {
        if (!ownsEvent(e)) return;
        if ((e.key === "Enter" || e.key === " ") && shouldHandleAtlasRouteClick(e)) {
          e.preventDefault();
          marker.click();
        }
      };
      marker.addEventListener("click", activate);
      marker.addEventListener("keydown", keyboard);
      listeners.push(() => {
        marker.removeEventListener("click", activate);
        marker.removeEventListener("keydown", keyboard);
      });
    });
    setEffectLinks(effects);
    return () => listeners.forEach((remove) => remove());
  }, [body, content.locator, relationships, openReference, html, modal]);
  if (body.kind === "unavailable")
    return (
      <Alert
        type="info"
        title={"Content unavailable: " + body.state.replace(/_/g, " ")}
      />
    );
  if (body.kind === "bounded")
    return (
      <Alert type="info" title="Content reading limit" description={body.reason} />
    );
  if (body.kind === "plain") return <p className="prose">{body.text}</p>;
  return (
    <>
      {contextHolder}
      {effectLinks}
      <div
        ref={container}
        className="prepared-content"
        dangerouslySetInnerHTML={{ __html: html }}
      />
    </>
  );
}
