import DOMPurify from "dompurify";
import { Alert, Modal } from "antd";
import { useMemo, useEffect, useRef } from "react";
import type {
  PreparedContentFieldView,
  RecordRelationshipView,
  RecordNavigationView,
} from "../../generated/atlas";
export type RecordReferenceHandler = (
  recordKey: string,
  anchor?: HTMLElement,
  navigation?: RecordNavigationView,
) => void;
export function PreparedContent({
  content,
  relationships = [],
  onReference,
}: {
  content: PreparedContentFieldView;
  relationships?: RecordRelationshipView[];
  onReference: RecordReferenceHandler;
}) {
  const [modal, contextHolder] = Modal.useModal();
  const container = useRef<HTMLDivElement>(null);
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
  useEffect(() => {
    if (body.kind !== "html") return;
    const markers = container.current?.querySelectorAll<HTMLElement>(
      "[data-atlas-reference],[data-atlas-interaction]",
    );
    const listeners: Array<() => void> = [];
    markers?.forEach((marker) => {
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
        return;
      }
      marker.setAttribute("role", "button");
      marker.tabIndex = 0;
      const ownsEvent = (e: Event) => {
        const target = e.target as Element;
        const link = target.closest("a[href]");
        return (
          target.closest("[data-atlas-reference],[data-atlas-interaction]") ===
            marker &&
          (!link || link === marker)
        );
      };
      const activate = (e: Event) => {
        if (!ownsEvent(e)) return;
        const marker = (e.target as Element).closest(
          "[data-atlas-reference],[data-atlas-interaction]",
        );
        if (!marker || !container.current?.contains(marker)) return;
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
          onReference(fact.target.record_key, marker as HTMLElement, fact.target);
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
        if (e.key === "Enter" || e.key === " ") {
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
    return () => listeners.forEach((remove) => remove());
  }, [body, content.locator, relationships, onReference, html, modal]);
  if (body.kind === "unavailable")
    return (
      <Alert
        type="info"
        message={"Content unavailable: " + body.state.replace(/_/g, " ")}
      />
    );
  if (body.kind === "plain") return <p className="prose">{body.text}</p>;
  return (
    <>
      {contextHolder}
      <div
        ref={container}
        className="prepared-content"
        dangerouslySetInnerHTML={{ __html: html }}
      />
    </>
  );
}
