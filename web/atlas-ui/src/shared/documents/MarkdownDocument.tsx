import Markdown, { defaultUrlTransform, type Components } from "react-markdown";
import { destinationHref, isPrimaryActivation } from "../../app/navigation";
import { recordDestination } from "../../app/destinations";
import { RecordPreviewDrawer } from "../records/RecordPreviewDrawer";
import { useRecordPreview } from "../records/useRecordPreview";
import { createContext, useContext, type ReactNode } from "react";
import "./documents.css";

const recordPrefix = "atlas-record:";

export function markdownRecordLink(title: string, key: string) {
  const label = title.replace(/[\\[\]*_`<>]/g, "\\$&");
  const target = encodeURIComponent(key)
    .replace("%3A", ":")
    .replace(/[()]/g, (character) => (character === "(" ? "%28" : "%29"));
  return `[${label}](${recordPrefix}${target})`;
}

function urlTransform(url: string) {
  return url.startsWith(recordPrefix) ? url : defaultUrlTransform(url);
}

function recordTarget(href: string) {
  try {
    const key = decodeURIComponent(href.slice(recordPrefix.length));
    return /^[^:\s]+:[^:\s]+$/.test(key) ? key : undefined;
  } catch {
    return undefined;
  }
}

const RecordLinkContext = createContext<
  ((key: string, opener: HTMLElement) => void) | null
>(null);

function MarkdownLink({ href, children }: { href?: string; children?: ReactNode }) {
  const open = useContext(RecordLinkContext);
  if (!href?.startsWith(recordPrefix)) {
    return href ? <a href={href}>{children}</a> : <>{children}</>;
  }
  const key = recordTarget(href);
  if (!key) return <span>{children} (record unavailable)</span>;
  return (
    <a
      href={destinationHref(recordDestination({ recordKey: key }))}
      onClick={(event) => {
        if (!open || !isPrimaryActivation(event)) return;
        event.preventDefault();
        open(key, event.currentTarget);
      }}
    >
      {children}
    </a>
  );
}

const documentComponents: Components = { a: MarkdownLink };

export function MarkdownDocument({ source }: { source: string }) {
  const preview = useRecordPreview();
  return (
    <RecordLinkContext value={preview.open}>
      <div className="markdown-document">
        <Markdown
          skipHtml
          disallowedElements={["img"]}
          urlTransform={urlTransform}
          components={documentComponents}
        >
          {source}
        </Markdown>
      </div>
      <RecordPreviewDrawer preview={preview} />
    </RecordLinkContext>
  );
}

function excerptBlock({ children }: { children?: ReactNode }) {
  return <>{children} </>;
}

const excerptComponents: Components = {
  a: ({ children }) => <>{children}</>,
  blockquote: excerptBlock,
  h1: excerptBlock,
  h2: excerptBlock,
  h3: excerptBlock,
  h4: excerptBlock,
  h5: excerptBlock,
  h6: excerptBlock,
  li: excerptBlock,
  ol: excerptBlock,
  p: excerptBlock,
  pre: excerptBlock,
  ul: excerptBlock,
  hr: () => " ",
  br: () => " ",
};

export function MarkdownExcerpt({ source }: { source: string }) {
  return (
    <Markdown skipHtml disallowedElements={["img"]} components={excerptComponents}>
      {source}
    </Markdown>
  );
}
