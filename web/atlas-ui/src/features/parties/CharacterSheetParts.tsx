import type { SheetFactView, SheetRowView } from "../../generated/atlas";
import { ReferenceHelpLink } from "../../shared/records/ReferenceHelp";
import { rootNavigation } from "../../shared/records/useReferenceReading";

export type SheetPresentation = {
  links: Map<string, string | undefined>;
  generation?: string;
};

export function SheetLabel({
  row,
  links,
  recordKey = row.ability_key ? links.get(row.ability_key) : row.record_key,
  generation,
}: SheetPresentation & { row: SheetRowView; recordKey?: string }) {
  return (
    <ReferenceHelpLink
      help={
        recordKey
          ? {
              definition_key: null,
              destination: { kind: "record", navigation: rootNavigation(recordKey) },
            }
          : null
      }
      generation={generation}
    >
      {row.label}
    </ReferenceHelpLink>
  );
}

export function SheetFacts({
  facts,
  className = "",
}: {
  facts: SheetFactView[];
  className?: string;
}) {
  if (!facts.length) return null;
  return (
    <dl className={`character-sheet__values ${className}`}>
      {facts.map((fact, index) => (
        <div key={`${fact.label}-${index}`}>
          <dt>{fact.label}</dt>
          <dd>{fact.display_value ?? fact.value}</dd>
        </div>
      ))}
    </dl>
  );
}
