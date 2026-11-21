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
  SheetGroupView,
  SheetRowView,
  SheetSectionKind,
} from "../../generated/atlas";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { referenceQueryMeta } from "../../shared/queries/referenceQueries";
import { ReferenceHelpLink } from "../../shared/records/ReferenceHelp";
import { landReadingTarget } from "../../shared/records/landReadingTarget";
import { FactRow } from "../../shared/records/presentationFacts";
import { ValueExplanation } from "../../shared/ui/ValueExplanation";
import { LoadingStatus } from "../../shared/ui/LoadingStatus";

import { CharacterSpellRanks } from "./CharacterSpellRanks";
import { CharacterInventory } from "./CharacterInventory";
import { CharacterProgression } from "./CharacterProgression";
import { SheetFacts, SheetLabel } from "./CharacterSheetParts";

type SheetTab = SheetSectionKind | "abilities" | "details" | "review";
const sectionLabels: Record<SheetSectionKind, string> = {
  stats: "Stats",
  progression: "Progression",
  magic: "Magic",
  inventory: "Inventory",
  companions: "Companions",
  proficiencies: "Proficiencies",
};

export function SavedCharacterSheet({
  character,
  defaultTab,
  emptySheetAction,
  abilityRoute,
  abilities,
  active = true,
  refreshCharacter,
}: {
  character: PcReferenceView;
  emptySheetAction?: ReactNode;
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
      emptySheetAction={emptySheetAction}
      refreshCharacter={refreshCharacter}
      active={active}
    />
  );
}

export function CharacterSheet({
  defaultTab,
  emptySheetAction,
  abilityRoute,
  abilitySections,
  abilities,
  sheet: draftSheet,
  draftDetails,
  priorDetailsChanged = false,
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
  priorDetailsChanged?: boolean;
  saved?: PcReferenceView;
  emptySheetAction?: ReactNode;
  refreshCharacter?: () => Promise<number | undefined>;
  active?: boolean;
  reviewSummary?: ReactNode;
  generation?: string;
}) {
  const [selected, setSelected] = useState<SheetTab>(() =>
    abilityRoute?.active ? "abilities" : defaultTab,
  );
  // Revision-keyed sheet reads can briefly unmount the Magic pane during a refresh.
  const [openPreparations, setOpenPreparations] = useState<string[]>([]);
  const skillsHeadingId = useId();
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
    enabled: active && Boolean(saved?.imported),
    retry: false,
    meta: referenceQueryMeta,
  });
  const sheet = draftSheet ?? sheetQuery.data?.sheet;
  const showAbilities = Boolean(
    abilityRoute?.active || abilitySections.some((section) => section.abilities.length),
  );
  const showDetails = Boolean(sheet?.has_additional_details || priorDetailsChanged);
  const sections: SheetSectionKind[] = [
    "inventory",
    "magic",
    "progression",
    "companions",
    "proficiencies",
  ];
  const visibleSections = sections.filter((kind) =>
    sheet?.sections.some(
      (section) => section.kind === kind && section.groups.length > 0,
    ),
  );
  const visible = new Set<SheetTab>(["stats", ...visibleSections]);
  if (showAbilities) visible.add("abilities");
  if (showDetails) visible.add("details");
  if (reviewSummary) visible.add("review");
  const selectedTab = abilityRoute?.active
    ? "abilities"
    : visible.has(selected)
      ? selected
      : "stats";
  const detailsQuery = useQuery({
    queryKey: characterKey
      ? ["character-import-details", characterKey, revision]
      : ["draft-import-details", draftId, generation],
    queryFn: ({ signal }) =>
      draftDetails
        ? draftDetails(signal)
        : getCharacterImportDetails(characterKey!, revision!, signal),
    enabled: active && showDetails && selectedTab === "details",
    retry: false,
    gcTime: draftDetails ? 0 : undefined,
  });
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

  function renderGroups(kind: SheetSectionKind, groups: SheetGroupView[]) {
    if (kind === "inventory")
      return (
        <CharacterInventory groups={groups} links={links} generation={generation} />
      );
    if (kind === "progression")
      return (
        <CharacterProgression groups={groups} links={links} generation={generation} />
      );
    const skills = groups.find((group) => group.kind === "skills");
    return (
      <div className={`character-sheet__groups character-sheet__groups--${kind}`}>
        <div className="character-sheet__group-stack">
          {groups.map((group, index) => {
            if (group === skills) return null;
            const key = `${group.label}-${index}`;
            return (
              <SheetGroup key={key} group={group} links={links} generation={generation}>
                {group.spell_ranks?.length ? (
                  <CharacterSpellRanks
                    ranks={group.spell_ranks}
                    links={links}
                    generation={generation}
                    open={openPreparations.includes(key)}
                    onOpenChange={(open) =>
                      setOpenPreparations((current) =>
                        open
                          ? [...current, key]
                          : current.filter((entry) => entry !== key),
                      )
                    }
                  />
                ) : null}
              </SheetGroup>
            );
          })}
        </div>
        {skills && (
          <SheetGroup
            group={skills}
            links={links}
            generation={generation}
            headingId={skillsHeadingId}
          />
        )}
      </div>
    );
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
    if (!sheet && emptySheetAction)
      return (
        <>
          <p>
            {saved?.imported
              ? "This character has an abilities reference."
              : "No character sheet yet."}
          </p>
          {emptySheetAction}
        </>
      );
    if (!sheet)
      return (
        <p>
          {saved?.imported
            ? "This character has an abilities reference. Use New import from Pathbuilder on the character page to add the full sheet."
            : "Import a build from Pathbuilder to add a character sheet."}
        </p>
      );
    const groups = sheet.sections.find((section) => section.kind === kind)?.groups;
    const identityDiffers =
      saved && (saved.name !== sheet.name || saved.level !== sheet.level);
    const hasSkills = groups?.some(
      (group) => group.kind === "skills" && group.rows.length > 0,
    );
    return (
      <>
        {kind === "stats" && (identityDiffers || hasSkills) && (
          <div className="character-sheet__snapshot">
            {identityDiffers && (
              <Typography.Text type="secondary">
                {saved.name !== sheet.name ? (
                  <>
                    Imported as <strong>{sheet.name}</strong>
                    {saved.level !== sheet.level && <> · Level {sheet.level}</>}
                  </>
                ) : (
                  <>Imported level {sheet.level}</>
                )}
              </Typography.Text>
            )}
            {hasSkills && (
              <Typography.Link
                className="character-sheet__skills-jump"
                href={`#${skillsHeadingId}`}
                onClick={(event) => {
                  event.preventDefault();
                  const heading = document.getElementById(skillsHeadingId);
                  if (!heading) return;
                  heading.focus({ preventScroll: true });
                  const boundary = heading.closest<HTMLElement>(
                    '[role="dialog"], .atlas-app__content',
                  );
                  if (boundary) landReadingTarget(heading, boundary);
                  else heading.scrollIntoView({ block: "start" });
                }}
              >
                Jump to Skills
              </Typography.Link>
            )}
          </div>
        )}
        {groups?.length ? (
          renderGroups(kind, groups)
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
        ...(showAbilities
          ? defaultTab === "stats"
            ? [stats, lookup]
            : [lookup, stats]
          : [stats]),
        ...visibleSections.map((kind) => ({
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
        ...(showDetails
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

function SheetGroup({
  group,
  links,
  generation,
  headingId,
  children,
}: {
  group: SheetGroupView;
  links: Map<string, string | undefined>;
  generation?: string;
  headingId?: string;
  children?: ReactNode;
}) {
  const skills = group.kind === "skills";
  const aligned = skills || group.kind === "proficiencies";
  const compact =
    !aligned && group.rows.every((row) => row.summary_value !== undefined);
  const items = group.rows.map((row, index) => {
    const content = <SheetRow row={row} links={links} generation={generation} />;
    const key = `${row.label}-${index}`;
    return compact ? <div key={key}>{content}</div> : <li key={key}>{content}</li>;
  });
  return (
    <section className={skills ? "character-sheet__skills" : undefined}>
      <h3 id={headingId} tabIndex={headingId ? -1 : undefined}>
        {group.label}
      </h3>
      {items.length > 0 &&
        (compact ? (
          <div className="record-fact-list">{items}</div>
        ) : (
          <ul
            className={aligned ? "character-sheet__rank-list" : "character-sheet__rows"}
          >
            {items}
          </ul>
        ))}
      {children}
    </section>
  );
}

function SheetRow({
  row,
  links,
  generation,
}: {
  row: SheetRowView;
  links: Map<string, string | undefined>;
  generation?: string;
}) {
  const label = <SheetLabel row={row} links={links} generation={generation} />;
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
          <ReferenceHelpLink help={row.summary_help} generation={generation}>
            {row.summary_value}
          </ReferenceHelpLink>
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

function ImportDetails({ details }: { details: ImportDetailsView }) {
  const previous = details.previous_fields ?? [];
  if (!previous.length) {
    return <ImportFields fields={details.fields} />;
  }
  if (!details.fields.length) {
    return (
      <>
        <p>These details appear in the previous import only.</p>
        <ImportFields fields={previous} />
      </>
    );
  }
  return (
    <Collapse
      defaultActiveKey={["incoming"]}
      items={[
        {
          key: "incoming",
          label: "Incoming import",
          children: <ImportFields fields={details.fields} />,
        },
        {
          key: "previous",
          label: "Previous import",
          children: <ImportFields fields={previous} />,
        },
      ]}
    />
  );
}

function ImportFields({ fields }: { fields: ImportFieldView[] }) {
  return (
    <ul className="character-sheet__details">
      {fields.map((field, index) => {
        const label = field.display_label ?? field.label;
        return (
          <li key={`${field.label}-${index}`}>
            {field.collection ? (
              <ImportCollection field={field} label={label} />
            ) : field.scalar_kind === "source_fragment" ? (
              <details>
                <summary>{label}</summary>
                <pre className="character-sheet__source-fragment">{field.value}</pre>
              </details>
            ) : (
              <FactRow label={label}>
                {field.scalar_kind === "text" &&
                field.value !== undefined &&
                /^(?:true|false|null|-?\d+(?:\.\d+)?(?:e[+-]?\d+)?)$/i.test(field.value)
                  ? JSON.stringify(field.value)
                  : field.value}
              </FactRow>
            )}
          </li>
        );
      })}
    </ul>
  );
}

function ImportCollection({ field, label }: { field: ImportFieldView; label: string }) {
  const [opened, setOpened] = useState(false);
  return (
    <details
      onToggle={(event) => {
        if (event.currentTarget.open) setOpened(true);
      }}
    >
      <summary>{label}</summary>
      {opened && <ImportFields fields={field.children} />}
    </details>
  );
}
