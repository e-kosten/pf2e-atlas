import { Typography } from "antd";
import { useRef } from "react";
import type React from "react";
import type {
  CreatureSurfaceContentBlockView,
  CreatureSurfaceContentInlineView,
  CreatureSurfaceContentView,
} from "../../generated/atlas";

export type ReferenceHandler = (
  recordKey: string,
  anchorRect?: DOMRect,
  triggerElement?: HTMLElement,
) => void;

export function RichContent({
  compact = false,
  content,
  onReference,
}: {
  compact?: boolean;
  content: CreatureSurfaceContentView;
  onReference: ReferenceHandler;
}) {
  const blocks = compact ? content.blocks.slice(0, 1) : content.blocks;
  return (
    <div className="creature-sheet__rich-content">
      {blocks.map((block, index) => (
        <RichBlock
          block={block}
          key={`${content.content_key}:${index}`}
          onReference={onReference}
        />
      ))}
    </div>
  );
}

function RichBlock({
  block,
  onReference,
}: {
  block: CreatureSurfaceContentBlockView;
  onReference: ReferenceHandler;
}) {
  switch (block.block_type) {
    case "heading": {
      const Heading =
        `h${Math.min(6, Math.max(4, block.level + 2))}` as keyof React.JSX.IntrinsicElements;
      return <Heading>{block.text}</Heading>;
    }
    case "paragraph":
      return (
        <p>
          {block.spans.map((span, index) => (
            <RichInline key={index} onReference={onReference} span={span} />
          ))}
        </p>
      );
    case "list": {
      const List = block.ordered ? "ol" : "ul";
      return (
        <List>
          {block.items.map((item, index) => (
            <li key={index}>
              {item.blocks.map((child, childIndex) => (
                <RichBlock block={child} key={childIndex} onReference={onReference} />
              ))}
            </li>
          ))}
        </List>
      );
    }
    case "table":
      return (
        <div className="creature-sheet__table-scroll">
          <table>
            {block.caption && <caption>{block.caption}</caption>}
            <tbody>
              {block.rows.map((row, rowIndex) => (
                <tr key={rowIndex}>
                  {row.cells.map((cell, cellIndex) => (
                    <td key={cellIndex}>
                      {cell.map((child, childIndex) => (
                        <RichBlock
                          block={child}
                          key={childIndex}
                          onReference={onReference}
                        />
                      ))}
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      );
    case "divider":
      return <hr />;
  }
}

function RichInline({
  onReference,
  span,
}: {
  onReference: ReferenceHandler;
  span: CreatureSurfaceContentInlineView;
}): React.ReactNode {
  switch (span.span_type) {
    case "text":
      return span.text;
    case "strong":
      return (
        <strong>
          {span.spans.map((child, index) => (
            <RichInline key={index} onReference={onReference} span={child} />
          ))}
        </strong>
      );
    case "emphasis":
      return (
        <em>
          {span.spans.map((child, index) => (
            <RichInline key={index} onReference={onReference} span={child} />
          ))}
        </em>
      );
    case "code":
      return <code>{span.text}</code>;
    case "reference":
      return (
        <RecordReference
          label={span.label}
          onReference={onReference}
          recordKey={span.record_key}
        />
      );
    case "check":
      return <span className="creature-sheet__check">{span.display}</span>;
    case "line_break":
      return <br />;
  }
}

export function RecordReference({
  expanded,
  label,
  onReference,
  openOnFocus = false,
  recordKey,
}: {
  expanded?: boolean;
  label: string;
  onReference: ReferenceHandler;
  openOnFocus?: boolean;
  recordKey: string | undefined;
}) {
  const pointerActivation = useRef(false);
  if (!recordKey) {
    return <span>{label}</span>;
  }
  const openReference = (element: HTMLElement) => {
    onReference(recordKey, element.getBoundingClientRect(), element);
  };
  return (
    <Typography.Link
      aria-expanded={openOnFocus ? Boolean(expanded) : undefined}
      aria-haspopup={openOnFocus ? "dialog" : undefined}
      href={`/records/${encodeURIComponent(recordKey)}`}
      onBlur={() => {
        pointerActivation.current = false;
      }}
      onClick={(event) => {
        event.preventDefault();
        pointerActivation.current = false;
        openReference(event.currentTarget);
      }}
      onFocus={(event) => {
        if (openOnFocus && !pointerActivation.current) {
          openReference(event.currentTarget);
        }
      }}
      onPointerDown={() => {
        if (openOnFocus) {
          pointerActivation.current = true;
        }
      }}
    >
      {label}
    </Typography.Link>
  );
}

export function narrativeContent(content: CreatureSurfaceContentView[] | undefined) {
  return (content ?? []).filter(
    (document) =>
      document.role === "primary_description" ||
      document.role === "summary" ||
      document.role === "supplemental_rules" ||
      document.role === "generated_narrative" ||
      document.role === "journal_page",
  );
}

export function contentLabel(content: CreatureSurfaceContentView) {
  if (content.label && content.label !== "Public Notes") {
    return content.label;
  }
  return content.role === "primary_description" ? "Description" : "More lore";
}
