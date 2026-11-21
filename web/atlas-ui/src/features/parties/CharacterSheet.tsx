import { Button, Collapse, Tabs, Typography } from "antd";
import { useQuery } from "@tanstack/react-query";
import { useId, useMemo, useState, type ReactNode } from "react";
import { getCharacterImportDetails, getCharacterSheet } from "../../api/atlasApi";
import type {
  CharacterSheetView,
  ImportDetailsView,
  ImportFieldView,
  PcAbilitySectionView,
  PcReferenceView,
  SheetFactView,
  SheetGroupView,
  SheetRowView,
  SheetSectionKind,
} from "../../generated/atlas";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { referenceQueryMeta } from "../../shared/queries/referenceQueries";
import { ReferenceHelpLink } from "../../shared/records/ReferenceHelp";
import { FactRow } from "../../shared/records/presentationFacts";
import { ValueExplanation } from "../../shared/ui/ValueExplanation";
import { rootNavigation } from "../../shared/records/useReferenceReading";
import { LoadingStatus } from "../../shared/ui/LoadingStatus";

type SheetTab = SheetSectionKind | "abilities" | "details" | "review";
const sectionLabels: Record<SheetSectionKind, string> = {
  stats: "Stats",
  progression: "Progression",
  magic: "Magic",
  inventory: "Inventory",
  companions: "Companions",
};

export function SavedCharacterSheet({
  character,
  defaultTab,
  abilityRoute,
  abilities,
  active = true,
  refreshCharacter,
}: {
  character: PcReferenceView;
  refreshCharacter: () => Promise<number | undefined>;
  defaultTab: "stats" | "abilities";
  abilityRoute?: { active: boolean; close: () => void };
  abilities: (active: boolean) => ReactNode;
  active?: boolean;
}) {
  return (
    <CharacterSheet
      defaultTab={defaultTab}
      abilityRoute={abilityRoute}
      abilitySections={character.sections}
      abilities={(visible) => abilities(visible && active)}
      saved={character}
      refreshCharacter={refreshCharacter}
      active={active}
    />
  );
}

export function CharacterSheet({
  defaultTab,
  abilityRoute,
  abilitySections,
  abilities,
  sheet: draftSheet,
  draftDetails,
  saved,
  active = true,
  reviewSummary,
  generation,
  refreshCharacter,
}: {
  defaultTab: "stats" | "abilities";
  abilityRoute?: { active: boolean; close: () => void };
  abilitySections: PcAbilitySectionView[];
  abilities: (active: boolean) => ReactNode;
  sheet?: CharacterSheetView;
  draftDetails?: (signal: AbortSignal) => Promise<ImportDetailsView>;
  saved?: PcReferenceView;
  refreshCharacter?: () => Promise<number | undefined>;
  active?: boolean;
  reviewSummary?: ReactNode;
  generation?: string;
}) {
  const [selected, setSelected] = useState<SheetTab>(() =>
    abilityRoute?.active ? "abilities" : defaultTab,
  );
  const selectedTab = abilityRoute?.active ? "abilities" : selected;
  const draftId = useId();
  const characterKey = saved?.character_key;
  const revision = saved?.character_revision;
  const sheetQuery = useQuery({
    queryKey: ["character-sheet", characterKey, revision],
    queryFn: async ({ signal }) => {
      const response = await getCharacterSheet(characterKey!, signal);
      if (
        response.character_key !== characterKey ||
        response.character_revision !== revision
      ) {
        throw new Error(
          "The character changed. Review the latest character to continue.",
        );
      }
      return response;
    },
    enabled:
      active &&
      Boolean(saved?.imported) &&
      selectedTab !== "abilities" &&
      selectedTab !== "details",
    retry: false,
    meta: referenceQueryMeta,
  });
  const detailsQuery = useQuery({
    queryKey: characterKey
      ? ["character-import-details", characterKey, revision]
      : ["draft-import-details", draftId, generation],
    queryFn: ({ signal }) =>
      draftDetails
        ? draftDetails(signal)
        : getCharacterImportDetails(characterKey!, revision!, signal),
    enabled: active && selectedTab === "details",
    retry: false,
    gcTime: draftDetails ? 0 : undefined,
  });
  const sheet = draftSheet ?? sheetQuery.data?.sheet;
  const links = useMemo(
    () =>
      new Map(
        abilitySections.flatMap((section) =>
          section.abilities.map((ability) => [ability.ability_key, ability.record_key]),
        ),
      ),
    [abilitySections],
  );

  async function retryRead(refetch: () => Promise<unknown>) {
    if (!refreshCharacter || (await refreshCharacter()) === revision) await refetch();
  }

  function renderSection(kind: SheetSectionKind) {
    if (kind !== selectedTab && !sheet) return null;
    if (sheetQuery.error)
      return (
        <ErrorNotice
          error={sheetQuery.error}
          title="Could not load character sheet"
          action={
            <Button onClick={() => void retryRead(sheetQuery.refetch)}>Retry</Button>
          }
        />
      );
    if (!sheet && saved?.imported && sheetQuery.isPending)
      return <LoadingStatus label="Loading character sheet…" />;
    if (!sheet)
      return (
        <p>
          {saved?.imported
            ? "This character has an abilities reference. Use New import from Pathbuilder on the character page to add the full sheet."
            : "Import a build from Pathbuilder to add a character sheet."}
        </p>
      );
    const groups = sheet.sections.find((section) => section.kind === kind)?.groups;
    return (
      <>
        {kind === "stats" && (
          <div className="character-sheet__snapshot">
            {saved && saved.name !== sheet.name && <strong>{sheet.name}</strong>}
            <Typography.Text type="secondary">
              Imported build · Level {sheet.level}
            </Typography.Text>
          </div>
        )}
        {groups?.length ? (
          <SheetGroups
            kind={kind}
            groups={groups}
            links={links}
            generation={generation}
          />
        ) : (
          <p>This export has no {sectionLabels[kind].toLocaleLowerCase()} to show.</p>
        )}
      </>
    );
  }

  const stats = {
    key: "stats",
    label: sectionLabels.stats,
    children: <div className="character-sheet__pane">{renderSection("stats")}</div>,
  };
  const lookup = {
    key: "abilities",
    label: "Abilities",
    children: abilities(selectedTab === "abilities"),
  };
  const sections: SheetSectionKind[] = [
    "inventory",
    "magic",
    "progression",
    "companions",
  ];
  return (
    <Tabs
      className="character-sheet"
      size="small"
      activeKey={selectedTab}
      onChange={(tab) => {
        setSelected(tab as SheetTab);
        if (abilityRoute?.active && tab !== "abilities") abilityRoute.close();
      }}
      items={[
        ...(defaultTab === "stats" ? [stats, lookup] : [lookup, stats]),
        ...(saved?.imported === false ? [] : sections).map((kind) => ({
          key: kind,
          label: sectionLabels[kind],
          children: <div className="character-sheet__pane">{renderSection(kind)}</div>,
        })),
        ...(reviewSummary
          ? [
              {
                key: "review",
                label: "Import summary",
                children: <div className="character-sheet__pane">{reviewSummary}</div>,
              },
            ]
          : []),
        ...(saved?.imported || draftDetails
          ? [
              {
                key: "details",
                label: "Additional import details",
                children: (
                  <div className="character-sheet__pane">
                    {detailsQuery.error && (
                      <ErrorNotice
                        error={detailsQuery.error}
                        title="Could not load import details"
                        action={
                          <Button onClick={() => void retryRead(detailsQuery.refetch)}>
                            Retry
                          </Button>
                        }
                      />
                    )}
                    {detailsQuery.isFetching && !detailsQuery.data && (
                      <LoadingStatus label="Loading import details…" />
                    )}
                    {detailsQuery.data && <ImportDetails details={detailsQuery.data} />}
                  </div>
                ),
              },
            ]
          : []),
      ]}
    />
  );
}

function SheetGroups({
  kind,
  groups,
  links,
  generation,
}: {
  kind: SheetSectionKind;
  groups: SheetGroupView[];
  links: Map<string, string | undefined>;
  generation?: string;
}) {
  return (
    <div className={`character-sheet__groups character-sheet__groups--${kind}`}>
      {groups.map((group, index) => {
        const compact = group.rows.every((row) => row.summary_value !== undefined);
        const items = group.rows.map((row, index) => {
          const key = row.ability_key ? links.get(row.ability_key) : row.record_key;
          const content = (
            <SheetRow row={row} recordKey={key} generation={generation} />
          );
          const rowKey = `${row.label}-${index}`;
          return compact ? (
            <div key={rowKey}>{content}</div>
          ) : (
            <li key={rowKey}>{content}</li>
          );
        });
        return (
          <section
            key={
              group.container_id
                ? `container-${group.container_id}`
                : `${group.label}-${index}`
            }
          >
            <h3>{group.label}</h3>
            {compact ? (
              <div className="record-fact-list">{items}</div>
            ) : (
              <ul className="character-sheet__rows">{items}</ul>
            )}
          </section>
        );
      })}
    </div>
  );
}

function SheetRow({
  row,
  recordKey,
  generation,
}: {
  row: SheetRowView;
  recordKey?: string;
  generation?: string;
}) {
  const label = (
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
  if (row.summary_value !== undefined) {
    const hasDetails =
      row.facts.length > 1 ||
      (row.facts.length === 1 && row.facts[0].value !== row.summary_value);
    return (
      <FactRow label={label}>
        {hasDetails ? (
          <ValueExplanation
            label={`Show details for ${row.label}`}
            title={row.label}
            content={<SheetFacts facts={row.facts} />}
          >
            {row.summary_value}
          </ValueExplanation>
        ) : (
          row.summary_value
        )}
      </FactRow>
    );
  }
  return (
    <>
      <strong>{label}</strong>
      {row.facts.length > 0 && <SheetFacts facts={row.facts} />}
    </>
  );
}

function SheetFacts({ facts }: { facts: SheetFactView[] }) {
  return (
    <dl className="character-sheet__values">
      {facts.map((fact, index) => (
        <div key={`${fact.label}-${index}`}>
          <dt>{fact.label}</dt>
          <dd>{fact.value}</dd>
        </div>
      ))}
    </dl>
  );
}

function ImportDetails({ details }: { details: ImportDetailsView }) {
  if (details.fields.length === 0)
    return <p>There is no saved export for this character.</p>;
  if (!details.previous_fields) return <ImportFields fields={details.fields} />;
  return (
    <Collapse
      defaultActiveKey={["incoming"]}
      items={[
        {
          key: "incoming",
          label: "Incoming export",
          children: <ImportFields fields={details.fields} />,
        },
        {
          key: "previous",
          label: "Previous export",
          children: <ImportFields fields={details.previous_fields} />,
        },
      ]}
    />
  );
}

function ImportFields({ fields }: { fields: ImportFieldView[] }) {
  return (
    <ul className="character-sheet__details">
      {fields.map((field, index) => (
        <li key={`${field.label}-${index}`}>
          {field.collection ? (
            <ImportCollection field={field} />
          ) : (
            <>
              <span>{field.label} </span>
              <code>{field.value}</code>
            </>
          )}
        </li>
      ))}
    </ul>
  );
}

function ImportCollection({ field }: { field: ImportFieldView }) {
  const [opened, setOpened] = useState(false);
  if (!field.children.length)
    return (
      <>
        <span>{field.label} </span>
        <code>{field.collection === "array" ? "[]" : "{}"}</code>
      </>
    );
  return (
    <details
      onToggle={(event) => {
        if (event.currentTarget.open) setOpened(true);
      }}
    >
      <summary>
        {field.label} ({field.children.length})
      </summary>
      {opened && <ImportFields fields={field.children} />}
    </details>
  );
}
