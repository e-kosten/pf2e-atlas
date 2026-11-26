import { Input, Typography } from "antd";
import { useState } from "react";
import { recordDestination } from "../../app/destinations";
import { destinationHref, isPrimaryActivation } from "../../app/navigation";
import type { JournalPresentationView } from "../../generated/atlas";
import type { RecordReferenceHandler } from "./PreparedContent";
import "../../styles/ui/journal-reading.css";

export type JournalContentsEntry = NonNullable<
  JournalPresentationView["pages"]["value"]
>[number];
type ContentsNode = {
  entry: JournalContentsEntry;
  children: ContentsNode[];
  visible: boolean;
};

function contentsTree(entries: JournalContentsEntry[], needle: string) {
  const roots: ContentsNode[] = [];
  const nodes: ContentsNode[] = [];
  for (const entry of entries) {
    const parent = entry.parent_index == null ? undefined : nodes[entry.parent_index];
    // A section title match includes its descendants.
    const visible =
      !!parent?.visible || entry.link.title.toLocaleLowerCase().includes(needle);
    const node: ContentsNode = { entry, children: [], visible };
    nodes.push(node);
    (parent?.children ?? roots).push(node);
  }
  // A matching child keeps its ancestors as context without revealing its siblings.
  for (let i = nodes.length - 1; i >= 0; i--) {
    const node = nodes[i];
    if (node.visible && node.entry.parent_index != null)
      nodes[node.entry.parent_index].visible = true;
  }
  const pages = nodes.filter((node) => !node.entry.is_heading_only);
  return {
    roots,
    total: pages.length,
    visible: pages.filter((node) => node.visible).length,
  };
}

function ContentsList({
  nodes,
  depth = 0,
  onReference,
}: {
  nodes: ContentsNode[];
  depth?: number;
  onReference: RecordReferenceHandler;
}) {
  const Heading = depth === 0 ? "h4" : depth === 1 ? "h5" : "h6";
  return (
    <ul className="journal-contents__pages">
      {nodes
        .filter((node) => node.visible)
        .map(({ entry, children }) => {
          const { title, navigation } = entry.link;
          const label = entry.is_heading_only ? (
            title
          ) : (
            <Typography.Link
              href={destinationHref(
                recordDestination({
                  recordKey: navigation.record_key,
                  selection: navigation,
                }),
              )}
              onClick={(event) => {
                if (!isPrimaryActivation(event)) return;
                event.preventDefault();
                onReference(navigation.record_key, event.currentTarget, navigation);
              }}
            >
              {title}
            </Typography.Link>
          );
          return (
            <li
              key={JSON.stringify(navigation.embedded_path)}
              className={children.length ? "journal-contents__group" : undefined}
            >
              {children.length ? (
                <>
                  <Heading>{label}</Heading>
                  <ContentsList
                    nodes={children}
                    depth={depth + 1}
                    onReference={onReference}
                  />
                </>
              ) : (
                label
              )}
            </li>
          );
        })}
    </ul>
  );
}

export function JournalContents({
  pages,
  onReference,
}: {
  pages: JournalPresentationView["pages"];
  onReference: RecordReferenceHandler;
}) {
  const [query, setQuery] = useState("");
  if (!pages.value?.length) return null;
  const needle = query.trim().toLocaleLowerCase();
  const tree = contentsTree(pages.value, needle);
  return (
    <section className="journal-contents" aria-label="Journal contents">
      <h3>Contents</h3>
      {(pages.value.length > 20 || query) && (
        <Input
          aria-label="Find a page"
          placeholder="Find a page…"
          value={query}
          allowClear
          onChange={(event) => setQuery(event.target.value)}
        />
      )}
      {needle && (
        <p className="muted" role="status">
          {tree.visible} of {tree.total} pages
        </p>
      )}
      {tree.visible ? (
        <ContentsList nodes={tree.roots} onReference={onReference} />
      ) : (
        <p>No matching pages.</p>
      )}
    </section>
  );
}
