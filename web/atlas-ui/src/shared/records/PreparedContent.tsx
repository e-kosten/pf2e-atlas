import { sanitizePreparedHtml } from "./PreparedHtml";
import DOMPurify from "dompurify";
import { Alert, Modal, Typography } from "antd";
import { documentHeadings } from "./documentHeadings";
import "../../styles/ui/journal-reading.css";
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
  PreparedFieldBodyView,
  PreparedReferenceView,
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
  documentNavigation = false,
  sectionLanding,
}: {
  documentNavigation?: boolean;
  sectionLanding?: string;
  content: PreparedContentFieldView;
  relationships?: RecordRelationshipView[];
  onReference: RecordReferenceHandler;
}) {
  return (
    <PreparedContentBody
      body={content.body}
      documentNavigation={
        documentNavigation
          ? {
              identity: JSON.stringify(content.locator),
              landing: sectionLanding,
              href: atlasRoutePath({
                kind: "record",
                recordKey: content.locator.record,
                selection: {
                  record_key: content.locator.record,
                  embedded_path: content.locator.embedded_path,
                  spell_rank: undefined,
                  field: null,
                  passage: null,
                  source_fingerprint: null,
                },
              }),
            }
          : undefined
      }
      references={relationships.filter(
        (reference) =>
          JSON.stringify(reference.source) === JSON.stringify(content.locator),
      )}
      onReference={onReference}
    />
  );
}

export function PreparedContentBody({
  body,
  references,
  onReference,
  documentNavigation,
}: {
  documentNavigation?: { identity: string; href: string; landing?: string };
  body: PreparedFieldBodyView;
  references: (PreparedReferenceView | RecordRelationshipView)[];
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
  const sanitized = useMemo(
    () => (body.kind === "html" ? sanitizePreparedHtml(body.html) : ""),
    [body],
  );
  const documentIdentity = documentNavigation?.identity;
  const prepared = useMemo(
    () =>
      documentIdentity
        ? documentHeadings(sanitized, documentIdentity)
        : { html: sanitized, outline: [] },
    [sanitized, documentIdentity],
  );
  const { html, outline } = prepared;
  const sectionLanding = documentNavigation?.landing;
  const landedSection = useRef<string | null>(null);
  useLayoutEffect(() => {
    if (!sectionLanding || landedSection.current === sectionLanding) return;
    const target = [
      ...(container.current?.querySelectorAll<HTMLElement>(".document-heading") ?? []),
    ].find((heading) => heading.id === sectionLanding);
    if (target) {
      landedSection.current = sectionLanding;
      target.scrollIntoView({ block: "start" });
      target.focus({ preventScroll: true });
    }
  }, [html, sectionLanding]);
  useLayoutEffect(() => {
    if (!container.current || !html) return;
    const tables = Array.from(container.current.querySelectorAll("table")).filter(
      (table) => !table.parentElement?.closest("table"),
    );
    const updateOverflow = () => {
      for (const table of tables) {
        const region = table.parentElement!;
        if (region.scrollWidth > region.clientWidth) {
          region.tabIndex = 0;
          region.setAttribute("role", "region");
          region.setAttribute(
            "aria-label",
            table.caption?.textContent?.trim() || "Reference table",
          );
        } else {
          region.removeAttribute("tabindex");
          region.removeAttribute("role");
          region.removeAttribute("aria-label");
        }
      }
    };
    const observer = new ResizeObserver(updateOverflow);
    for (const table of tables) {
      // Keep the authored nodes and reading position on unchanged-content updates.
      let region = table.parentElement!;
      if (!region.classList.contains("prepared-content__table-scroll")) {
        region = document.createElement("div");
        region.className = "prepared-content__table-scroll";
        table.replaceWith(region);
        region.append(table);
      }
      observer.observe(region);
      observer.observe(table);
    }
    updateOverflow();
    return () => observer.disconnect();
  }, [html]);
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
        ref === null ? null : references.find((r) => r.ordinal === Number(ref));
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
          const fact = references.find((r) => r.ordinal === ordinal);
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
          closable: true,
          mask: { closable: true },
          afterClose: () => {
            if (marker instanceof HTMLElement && marker.isConnected)
              marker.focus({ preventScroll: true });
          },
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
  }, [body, references, openReference, html, modal]);
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
      {outline.length > 0 && (
        <nav className="document-outline" aria-label="On this page">
          <strong>On this page</strong>
          <ul>
            {outline.map((section) => (
              <li key={section.id}>
                <Typography.Link
                  href={`${documentNavigation!.href}#${encodeURIComponent(section.id)}`}
                  onClick={(event) => {
                    if (!shouldHandleAtlasRouteClick(event)) return;
                    event.preventDefault();
                    const heading = [
                      ...(container.current?.querySelectorAll<HTMLElement>(
                        ".document-heading",
                      ) ?? []),
                    ].find((heading) => heading.id === section.id);
                    heading?.scrollIntoView({ block: "start" });
                    heading?.focus({ preventScroll: true });
                  }}
                >
                  {section.label}
                </Typography.Link>
              </li>
            ))}
          </ul>
        </nav>
      )}
      <div
        ref={container}
        className="prepared-content"
        dangerouslySetInnerHTML={{ __html: html }}
      />
    </>
  );
}
