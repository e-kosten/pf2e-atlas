import { useLayoutEffect, useRef, type ComponentProps } from "react";
import { recordDestination } from "../../app/destinations";
import { destinationHref, isPrimaryActivation } from "../../app/navigation";
import type { ProgressionInlineLinkView } from "../../generated/atlas";
import { PreparedContent } from "./PreparedContent";

type PrerequisiteContentProps = ComponentProps<typeof PreparedContent> & {
  links?: ProgressionInlineLinkView[];
};

export function PrerequisiteContent({ links, ...props }: PrerequisiteContentProps) {
  if (!links?.some((link) => link.field === props.content.locator.field)) {
    return <PreparedContent {...props} />;
  }
  return <LinkedPrerequisite {...props} links={links} />;
}

const INTERACTIVE_CONTENT =
  "a, button, [data-atlas-reference], [data-atlas-interaction], [aria-hidden=true]";

function LinkedPrerequisite({ links, ...props }: PrerequisiteContentProps) {
  const { content, onReference } = props;
  const root = useRef<HTMLDivElement>(null);
  const activate = useRef(onReference);
  useLayoutEffect(() => {
    activate.current = onReference;
  }, [onReference]);

  useLayoutEffect(() => {
    if (!root.current) return;
    const walker = document.createTreeWalker(root.current, NodeFilter.SHOW_TEXT);
    const nodes: Text[] = [];
    for (let node = walker.nextNode(); node; node = walker.nextNode()) {
      if (!node.parentElement?.closest(INTERACTIVE_CONTENT)) nodes.push(node as Text);
    }
    const occurrences = new Map<
      Text,
      { offset: number; link: ProgressionInlineLinkView }[]
    >();
    for (const link of links ?? []) {
      if (link.field !== content.locator.field || !link.text) continue;
      const matches: { node: Text; offset: number }[] = [];
      for (const node of nodes) {
        for (
          let position = node.data.indexOf(link.text);
          position >= 0;
          position = node.data.indexOf(link.text, position + link.text.length)
        ) {
          if (isWholeTextMatch(node.data, link.text, position)) {
            matches.push({ node, offset: position });
          }
        }
      }
      if (matches.length !== 1) continue;
      const { node, offset } = matches[0];
      const current = occurrences.get(node) ?? [];
      if (
        current.some(
          (other) =>
            offset < other.offset + other.link.text.length &&
            other.offset < offset + link.text.length,
        )
      )
        continue;
      occurrences.set(node, [...current, { offset, link }]);
    }

    const anchors: HTMLAnchorElement[] = [];
    for (const [node, matches] of occurrences) {
      const fragment = document.createDocumentFragment();
      let cursor = 0;
      for (const { offset, link } of matches.sort((a, b) => a.offset - b.offset)) {
        fragment.append(node.data.slice(cursor, offset));
        const anchor = document.createElement("a");
        const target = link.navigation;
        anchor.href = destinationHref(
          recordDestination({ recordKey: target.record_key, selection: target }),
        );
        anchor.textContent = link.text;
        anchor.addEventListener("click", (event) => {
          if (!isPrimaryActivation(event)) return;
          event.preventDefault();
          activate.current(target.record_key, anchor, target);
        });
        fragment.append(anchor);
        anchors.push(anchor);
        cursor = offset + link.text.length;
      }
      fragment.append(node.data.slice(cursor));
      node.replaceWith(fragment);
    }
    return () => {
      const parents = new Set<Node>();
      for (const anchor of anchors) {
        if (anchor.parentNode) parents.add(anchor.parentNode);
        anchor.replaceWith(anchor.textContent ?? "");
      }
      for (const parent of parents) parent.normalize();
    };
  }, [content, links]);

  return (
    <div ref={root}>
      <PreparedContent {...props} />
    </div>
  );
}

function isWholeTextMatch(source: string, match: string, offset: number) {
  const word = /[\p{L}\p{N}]/u;
  return (
    !(word.test(match[0]) && word.test(source[offset - 1] ?? "")) &&
    !(
      word.test(match[match.length - 1]) &&
      word.test(source[offset + match.length] ?? "")
    )
  );
}
