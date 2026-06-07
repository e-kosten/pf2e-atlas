import type {
  PresentationBlock,
  PresentationContent,
  PresentationContentBlock,
  PresentationFact,
  PresentationInline,
  PresentationRelationship,
  RecordDetailView,
} from "../generated/atlas";
import type React from "react";

type RecordPresentationProps = {
  detail: RecordDetailView | undefined;
  loading: boolean;
  onReference: (recordKey: string) => void;
};

export function RecordPresentation({
  detail,
  loading,
  onReference,
}: RecordPresentationProps) {
  if (loading) {
    return <div className="detail-empty">Loading record...</div>;
  }
  if (!detail) {
    return <div className="detail-empty">Select a result to inspect it.</div>;
  }

  return (
    <article className="record-detail">
      <header className="record-detail__header">
        <div>
          <p className="eyebrow">{detail.kind}</p>
          <h2>{detail.title}</h2>
          <p className="record-key">{detail.record_key}</p>
        </div>
        <div className="badge-row">
          {detail.presentation.badges.map((badge) => (
            <span className="atlas-badge" key={`${badge.kind}-${badge.value}`}>
              {badge.value}
            </span>
          ))}
        </div>
      </header>

      {detail.presentation.identity.length > 0 && (
        <FactGrid facts={detail.presentation.identity} />
      )}

      {detail.presentation.sections.map((section) => (
        <section className="record-section" key={section.kind}>
          <h3>{section.title}</h3>
          {section.blocks.map((block, index) => (
            <PresentationBlockView
              block={block}
              key={`${section.kind}-${index}`}
              onReference={onReference}
            />
          ))}
        </section>
      ))}
    </article>
  );
}

function PresentationBlockView({
  block,
  onReference,
}: {
  block: PresentationBlock;
  onReference: (recordKey: string) => void;
}) {
  switch (block.kind) {
    case "fact_list":
      return <FactGrid facts={block.content} />;
    case "prose":
      return <p className="prose">{block.content.text}</p>;
    case "relationships":
      return (
        <div className="relationship-list">
          {block.content.map((relationship) => (
            <RelationshipView
              key={`${relationship.kind}-${relationship.label}-${relationship.record_key ?? ""}`}
              relationship={relationship}
              onReference={onReference}
            />
          ))}
        </div>
      );
    case "content":
      return <ContentView content={block.content} onReference={onReference} />;
  }
}

function ContentView({
  content,
  onReference,
}: {
  content: PresentationContent;
  onReference: (recordKey: string) => void;
}) {
  return (
    <>
      {content.blocks.map((block, index) => (
        <ContentBlockView
          block={block}
          key={index}
          onReference={onReference}
        />
      ))}
    </>
  );
}

function ContentBlockView({
  block,
  onReference,
}: {
  block: PresentationContentBlock;
  onReference: (recordKey: string) => void;
}) {
  switch (block.kind) {
    case "heading": {
      const Heading = `h${Math.min(Math.max(block.level + 2, 3), 5)}` as
        | "h3"
        | "h4"
        | "h5";
      return <Heading>{block.text}</Heading>;
    }
    case "paragraph":
      return <p>{renderSpans(block.spans, onReference)}</p>;
    case "list": {
      const List = block.ordered ? "ol" : "ul";
      return (
        <List>
          {block.items.map((item, index) => (
            <li key={index}>
              {item.blocks.map((child, childIndex) => (
                <ContentBlockView
                  block={child}
                  key={childIndex}
                  onReference={onReference}
                />
              ))}
            </li>
          ))}
        </List>
      );
    }
    case "table":
      return (
        <table className="presentation-table">
          {block.caption && <caption>{block.caption}</caption>}
          <tbody>
            {block.rows.map((row, rowIndex) => (
              <tr key={rowIndex}>
                {row.cells.map((cell, cellIndex) => (
                  <td key={cellIndex}>
                    <ContentView content={cell} onReference={onReference} />
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      );
    case "rule":
      return <hr />;
  }
}

function FactGrid({ facts }: { facts: PresentationFact[] }) {
  return (
    <dl className="fact-grid">
      {facts.map((fact) => (
        <div key={`${fact.key}-${fact.value}`}>
          <dt>{fact.label}</dt>
          <dd>{fact.value}</dd>
        </div>
      ))}
    </dl>
  );
}

function RelationshipView({
  relationship,
  onReference,
}: {
  relationship: PresentationRelationship;
  onReference: (recordKey: string) => void;
}) {
  if (relationship.record_key) {
    return (
      <button
        className="relationship-link"
        type="button"
        onClick={() => onReference(relationship.record_key!)}
      >
        {relationship.label}
      </button>
    );
  }
  return <span className="relationship-chip">{relationship.label}</span>;
}

function renderSpans(
  spans: PresentationInline[],
  onReference: (recordKey: string) => void,
): React.ReactNode {
  return spans.map((span, index) => {
    switch (span.kind) {
      case "text":
        return <span key={index}>{span.text}</span>;
      case "strong":
        return <strong key={index}>{renderSpans(span.spans, onReference)}</strong>;
      case "emphasis":
        return <em key={index}>{renderSpans(span.spans, onReference)}</em>;
      case "code":
        return <code key={index}>{span.text}</code>;
      case "line_break":
        return <br key={index} />;
      case "reference":
        if (!span.record_key) {
          return <span key={index}>{span.label}</span>;
        }
        return (
          <button
            className="inline-reference"
            key={index}
            type="button"
            onClick={() => onReference(span.record_key!)}
          >
            {span.label}
          </button>
        );
    }
  });
}
