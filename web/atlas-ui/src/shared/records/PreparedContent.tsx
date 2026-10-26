import { destinationHref, isPrimaryActivation } from "../../app/navigation";
import { recordDestination } from "../../app/destinations";
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
import { ContentRoll } from "../dice/ContentRoll";
import type {
  PreparedContentFieldView,
  PreparedFieldBodyView,
  PreparedReferenceView,
  RecordRelationshipView,
  RecordNavigationView,
} from "../../generated/atlas";
export type RecordReferenceHandler = (
  recordKey: string,
  anchor?: HTMLElement,
  navigation?: RecordNavigationView,
  landing?: RecordNavigationView,
) => void;
const NO_RELATIONSHIPS: RecordRelationshipView[] = [];
// Plain check/area text inside a reference remains part of that reference.
const ACTIVATION_MARKER =
  '[data-atlas-reference],[data-atlas-interaction][role="button"]';
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
              href: destinationHref(
                recordDestination({
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
              ),
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
  documentNavigation?: {
    identity: string;
    href: string;
    landing?: string;
  };
  body: PreparedFieldBodyView;
  references: (PreparedReferenceView | RecordRelationshipView)[];
  onReference: RecordReferenceHandler;
}) {
  const [modal, contextHolder] = Modal.useModal();
  const container = useRef<HTMLDivElement>(null);
  const [effectLinks, setEffectLinks] = useState<ReactPortal[]>([]);
  const rollHosts = useRef(new Map<string, HTMLElement>());
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
    const activeRolls = new Set<string>();
    markers?.forEach((originalMarker) => {
      let marker = originalMarker;
      const namespace = marker
        .closest("[data-atlas-inclusion]")
        ?.getAttribute("data-atlas-inclusion");
      const inclusion =
        namespace == null
          ? undefined
          : body.inclusions.find((entry) => entry.namespace === Number(namespace));
      const markerReferences =
        namespace == null ? references : (inclusion?.references ?? []);
      const markerControls =
        namespace == null ? body.controls : (inclusion?.controls ?? []);
      const ref = marker.getAttribute("data-atlas-reference");
      const fact =
        ref === null ? null : markerReferences.find((r) => r.ordinal === Number(ref));
      const control =
        ref === null
          ? markerControls.find(
              (c) =>
                c.ordinal === Number(marker.getAttribute("data-atlas-interaction")),
            )?.control
          : null;
      if (control?.kind === "check" || control?.kind === "template") {
        marker.removeAttribute("href");
        marker.removeAttribute("role");
        marker.removeAttribute("tabindex");
        return;
      }
      if (fact?.self_action_reference) {
        marker.removeAttribute("href");
        marker.removeAttribute("role");
        marker.removeAttribute("tabindex");
        marker.removeAttribute("aria-disabled");
        marker.removeAttribute("title");
        return;
      }
      if (
        (control?.kind === "damage" || control?.kind === "command") &&
        control.literal
      ) {
        marker.removeAttribute("role");
        marker.removeAttribute("tabindex");
        if (control.roll) {
          const identity = `${namespace ?? "host"}:${JSON.stringify(control.roll)}`;
          activeRolls.add(identity);
          let host = rollHosts.current.get(identity);
          if (!host) {
            host = document.createElement("span");
            rollHosts.current.set(identity, host);
          }
          const formula = Array.from(
            marker.querySelectorAll<HTMLElement>("[data-atlas-roll-formula]"),
          ).find((node) => node.closest("[data-atlas-interaction]") === marker);
          (formula ?? marker).after(host);
          effects.push(
            createPortal(
              <ContentRoll request={control.roll} presentation={control.literal} />,
              host,
              identity,
            ),
          );
        }
        return;
      }
      if (fact?.target && fact.effect_label) {
        const host = document.createElement("span");
        host.setAttribute("data-atlas-reference", String(ref));
        marker.replaceWith(host);
        effects.push(
          createPortal(
            <EffectReferenceLink
              label={fact.effect_label}
              target={fact.contextual_target ?? fact.target}
              onReference={openReference}
            />,
            host,
            `${namespace ?? "host"}:${ref}`,
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
        if (
          fact?.contextual_target &&
          (fact.contextual_target.record_key !== fact.target?.record_key ||
            JSON.stringify(fact.contextual_target.embedded_path) !==
              JSON.stringify(fact.target?.embedded_path))
        )
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
          destinationHref(
            recordDestination({
              recordKey: readingTarget.record_key,
              selection: readingTarget,
            }),
          ),
        );
        marker.removeAttribute("role");
      } else marker.setAttribute("role", "button");
      marker.tabIndex = 0;
      const ownsEvent = (e: Event) => {
        const target = e.target as Element;
        if (target.closest("button")) return false;
        const link = target.closest("a[href]");
        return (
          target.closest(ACTIVATION_MARKER) === marker &&
          (!link || link === marker || !marker.contains(link))
        );
      };
      const activate = (e: Event) => {
        if (!ownsEvent(e)) return;
        const marker = (e.target as Element).closest(ACTIVATION_MARKER);
        if (!marker || !container.current?.contains(marker)) return;
        if (fact?.target && e instanceof MouseEvent && !isPrimaryActivation(e)) return;
        e.preventDefault();
        const reference = marker.getAttribute("data-atlas-reference");
        if (reference !== null) {
          const ordinal = Number(reference);
          const fact = markerReferences.find((r) => r.ordinal === ordinal);
          if (!fact?.target) return;
          const target = fact.contextual_target ?? fact.target;
          openReference(target.record_key, marker as HTMLElement, target);
          return;
        }
        const ordinal = Number(marker.getAttribute("data-atlas-interaction"));
        const control = markerControls.find((c) => c.ordinal === ordinal)?.control;
        if (control?.kind !== "damage") return;
        const title = control.formula;
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
        if ((e.key === "Enter" || e.key === " ") && isPrimaryActivation(e)) {
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
    for (const [identity, host] of rollHosts.current) {
      if (!activeRolls.has(identity)) {
        host.remove();
        rollHosts.current.delete(identity);
      }
    }
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
  const reading = (
    <>
      {outline.length > 0 && (
        <nav className="document-outline" aria-label="On this page">
          <strong>On this page</strong>
          <ul>
            {outline.map((section) => (
              <li key={section.id}>
                <Typography.Link
                  href={`${documentNavigation!.href}#${encodeURIComponent(section.id)}`}
                  onClick={(event) => {
                    if (!isPrimaryActivation(event)) return;
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
  return (
    <>
      {contextHolder}
      {effectLinks}
      {documentNavigation ? (
        <div
          className={`document-reading${outline.length ? " document-reading--outlined" : ""}`}
        >
          {reading}
        </div>
      ) : (
        reading
      )}
    </>
  );
}
