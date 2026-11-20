import type { CharacterAbilityView as AbilityView } from "../../app/routeSearch";
import { Button, Dropdown, Input, List, Space, Typography } from "antd";
import { Ellipsis } from "lucide-react";
import { useId, useLayoutEffect, useRef, useState, type ReactNode } from "react";
import type {
  PcAbilitySelection,
  PcAbilityView,
  PcReferenceView,
} from "../../generated/atlas";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { isArtifactGenerationChanged } from "../../shared/queries/referenceQueries";
import { BrowseReader } from "../../shared/records/BrowseReader";
import { BrowsePaneHeader } from "../../shared/records/BrowsePaneHeader";
import {
  hasReferenceSelectionStatus,
  ReferenceSelectionStatus,
} from "../../shared/records/ReferenceSelectionStatus";
import { usePreparedReferenceSelection } from "../../shared/records/usePreparedReferenceSelection";
import {
  useReferenceReading,
  rootNavigation,
} from "../../shared/records/useReferenceReading";
import { AbilityRulePicker } from "./AbilityRulePicker";

export type { CharacterAbilityView as AbilityView } from "../../app/routeSearch";
export type AbilityNavigation = {
  view: AbilityView | null;
  open: (view: AbilityView) => void;
  close: () => void;
};

type AbilityDocument = Pick<PcReferenceView, "abilities" | "sections" | "imported">;
type AbilityLinkMutation = {
  error: Error | null;
  isPending: boolean;
  reset: () => void;
  mutate: (
    input: {
      ability: PcAbilityView;
      selection: PcAbilitySelection;
      recordTitle?: string;
    },
    options: { onSuccess: () => void },
  ) => void;
};

const abilityLabel = (ability: PcAbilityView) =>
  ability.choice ? `${ability.label} (${ability.choice})` : ability.label;

export function PcAbilityBrowser({
  doc,
  toolbar,
  link,
  linkErrorAction,
  blocked = false,
  review = false,
  navigation,
  active = true,
  generation,
  revision,
  onReviewImport,
}: {
  doc?: AbilityDocument | null;
  toolbar?: ReactNode;
  link: AbilityLinkMutation;
  linkErrorAction?: ReactNode;
  blocked?: boolean;
  review?: boolean;
  navigation?: AbilityNavigation;
  active?: boolean;
  generation?: string;
  revision?: number;
  onReviewImport?: () => void;
}) {
  const titleId = useId();
  const [filter, setFilter] = useState("");
  const [localView, setLocalView] = useState<AbilityView | null>(null);
  const requestedView = navigation ? navigation.view : localView;
  const selected = doc?.abilities.find(
    (ability) => ability.ability_key === requestedView?.abilityKey,
  );
  const view = selected ? requestedView : null;
  const finding = view?.kind === "find";
  const recordKey =
    view?.kind === "read" && selected?.record_title ? selected.record_key : undefined;
  const reading = useReferenceReading(
    null,
    active && Boolean(recordKey),
    0,
    generation,
  );
  const selection = usePreparedReferenceSelection(
    reading,
    active && !finding,
    JSON.stringify([requestedView, revision]),
    generation ?? null,
  );
  const selectRecord = reading.select;
  const abilityRows = useRef(new Map<string, HTMLElement>());
  const abilityFilter = useRef<React.ComponentRef<typeof Input>>(null);
  const root = useRef<HTMLDivElement>(null);
  const opener = useRef<HTMLElement | null>(null);
  const linkedAbility = useRef<string | null>(null);
  const currentView = useRef(view);
  const previousView = useRef<AbilityView | null>(null);
  const busy = blocked || link.isPending;
  const reviewChangedGeneration =
    onReviewImport && isArtifactGenerationChanged(selection.current?.error);

  useLayoutEffect(() => {
    selectRecord(recordKey ? { navigation: rootNavigation(recordKey) } : null);
  }, [recordKey, selectRecord, view?.abilityKey]);

  useLayoutEffect(() => {
    currentView.current = view;
    if (view) {
      previousView.current = view;
      return;
    }
    const previous = previousView.current;
    if (!previous) return;
    const frame = requestAnimationFrame(() => {
      const linkedKey = linkedAbility.current;
      const previousRow = abilityRows.current.get(previous.abilityKey);
      const target = linkedKey
        ? abilityRows.current
            .get(linkedKey)
            ?.querySelector<HTMLElement>(".character-ability-link")
        : opener.current?.isConnected
          ? opener.current
          : previousRow?.querySelector<HTMLElement>("button");
      // A successful link write can close the task before the updated row commits.
      // Leave the focus return pending until the linked ability is rendered.
      if (
        linkedKey &&
        !target &&
        doc?.abilities.some(
          (ability) => ability.ability_key === linkedKey && !ability.record_title,
        )
      )
        return;
      previousView.current = null;
      linkedAbility.current = null;
      if (target) target.focus({ preventScroll: true });
      else abilityFilter.current?.focus({ preventScroll: true });
      opener.current = null;
    });
    return () => cancelAnimationFrame(frame);
  }, [doc?.abilities, view]);

  function open(next: AbilityView, anchor?: HTMLElement | null) {
    if (!view && anchor) opener.current = anchor;
    if (navigation) navigation.open(next);
    else setLocalView(next);
  }

  function close() {
    selection.cancel();
    link.reset();
    if (navigation) navigation.close();
    else setLocalView(null);
  }

  function startSearch(ability: PcAbilityView, anchor?: HTMLElement | null) {
    selection.cancel();
    link.reset();
    open({ kind: "find", abilityKey: ability.ability_key }, anchor);
  }

  function submitLink(
    ability: PcAbilityView,
    selection: PcAbilitySelection,
    recordTitle?: string,
  ) {
    link.mutate(
      { ability, selection, recordTitle },
      {
        onSuccess: () => {
          const active = currentView.current;
          if (active?.kind !== "find" || active.abilityKey !== ability.ability_key)
            return;
          linkedAbility.current = ability.ability_key;
          close();
        },
      },
    );
  }

  const term = filter.trim().toLocaleLowerCase();
  const visible = doc?.abilities.filter((ability) =>
    [abilityLabel(ability), ability.record_title].some((label) =>
      label?.toLocaleLowerCase().includes(term),
    ),
  );
  const visibleKeys = new Set(visible?.map((ability) => ability.ability_key));

  return (
    <div ref={root} className="pc-ability-browser">
      {link.error && (
        <ErrorNotice
          error={link.error}
          title={
            review
              ? "Review the import again to link this rule"
              : "Could not save rule link"
          }
          action={linkErrorAction}
        />
      )}
      {requestedView && (!view || (view.kind === "read" && !recordKey)) && doc && (
        <ErrorNotice
          title="Ability reference unavailable"
          error={new Error("Ability reference unavailable")}
          action={<Button onClick={close}>Dismiss</Button>}
        />
      )}
      <div className="pc-ability-browser__list" hidden={finding}>
        <BrowseReader
          resultsLabel="PC abilities"
          resultsIdentity={filter}
          resultsLoading={false}
          selectedRecordKey={recordKey ?? null}
          reading={reading}
          fullWidthUntilSelection
          observeWidth
          returnLabel="Back to abilities"
          onReaderClose={close}
          onClearSelection={close}
          renderResultsPaneHeader={() => (
            <BrowsePaneHeader className="reference-browser__toolbar">
              <Space orientation="vertical" size="small" style={{ width: "100%" }}>
                {toolbar}
                {doc?.imported && (
                  <>
                    <Input
                      aria-label="Find an ability"
                      ref={abilityFilter}
                      placeholder="Find an ability"
                      value={filter}
                      onChange={(event) => setFilter(event.target.value)}
                      allowClear
                    />
                    <nav
                      className="character-reference__sections"
                      aria-label="Ability sections"
                    >
                      {doc.sections
                        .filter((section) =>
                          section.abilities.some((ability) =>
                            visibleKeys.has(ability.ability_key),
                          ),
                        )
                        .map((section) => {
                          const id = `${titleId}-${section.kind}`;
                          return (
                            <a
                              key={section.kind}
                              href={`#${id}`}
                              onClick={(event) => {
                                event.preventDefault();
                                const target = root.current?.querySelector<HTMLElement>(
                                  `[id="${id}"]`,
                                );
                                const controls = event.currentTarget.closest(
                                  ".reference-results__controls",
                                );
                                const scroller = target?.closest(
                                  ".reference-results__scroll",
                                );
                                if (
                                  target &&
                                  scroller &&
                                  controls &&
                                  getComputedStyle(controls).position === "sticky"
                                ) {
                                  scroller.scrollTop +=
                                    target.getBoundingClientRect().top -
                                    controls.getBoundingClientRect().bottom;
                                } else {
                                  target?.scrollIntoView({ block: "start" });
                                }
                              }}
                            >
                              {section.label}
                            </a>
                          );
                        })}
                    </nav>
                  </>
                )}
              </Space>
            </BrowsePaneHeader>
          )}
        >
          <div className="pc-ability-browser__rows">
            {doc?.imported ? (
              <>
                {doc.sections.map((section) => {
                  const sectionAbilities = section.abilities.filter((ability) =>
                    visibleKeys.has(ability.ability_key),
                  );
                  if (!sectionAbilities.length) return null;
                  return (
                    <section
                      key={section.kind}
                      id={`${titleId}-${section.kind}`}
                      className="character-ability-section"
                    >
                      <h3>
                        {section.label} <span>{sectionAbilities.length}</span>
                      </h3>
                      <List
                        size="small"
                        dataSource={sectionAbilities}
                        locale={{
                          emptyText: filter
                            ? "No matching abilities."
                            : "No abilities in this export.",
                        }}
                        renderItem={(ability) => {
                          const label = abilityLabel(ability);
                          const canUseAutomatic =
                            ability.manual && ability.automatic_match_available;
                          const preparing =
                            selection.current?.request.navigation.record_key ===
                              ability.record_key &&
                            selection.current?.name === label &&
                            hasReferenceSelectionStatus(selection.current);
                          return (
                            <List.Item key={ability.ability_key}>
                              <div
                                className="character-ability-row"
                                ref={(node) => {
                                  if (node)
                                    abilityRows.current.set(ability.ability_key, node);
                                  else abilityRows.current.delete(ability.ability_key);
                                }}
                              >
                                <div>
                                  {ability.record_key && ability.record_title ? (
                                    <Button
                                      type="link"
                                      className="character-ability-link"
                                      aria-current={
                                        view?.kind === "read" &&
                                        view.abilityKey === ability.ability_key
                                          ? "true"
                                          : undefined
                                      }
                                      style={{
                                        paddingInline: 0,
                                        whiteSpace: "normal",
                                        height: "auto",
                                        textAlign: "start",
                                      }}
                                      data-record-key={ability.record_key}
                                      onClick={(event) => {
                                        const anchor = event.currentTarget;
                                        void selection.select({
                                          request: {
                                            navigation: rootNavigation(
                                              ability.record_key!,
                                            ),
                                          },
                                          name: label,
                                          generation: generation ?? null,
                                          commit: () =>
                                            open(
                                              {
                                                kind: "read",
                                                abilityKey: ability.ability_key,
                                              },
                                              anchor,
                                            ),
                                        });
                                      }}
                                    >
                                      {label}
                                    </Button>
                                  ) : (
                                    <>
                                      <Typography.Text>{label}</Typography.Text>
                                      {review && (
                                        <span className="character-secondary">
                                          {ability.record_key
                                            ? "Reference unavailable"
                                            : ability.ambiguous
                                              ? "Multiple matches"
                                              : "No automatic match"}
                                        </span>
                                      )}
                                    </>
                                  )}
                                  {ability.record_title &&
                                    ability.record_title.trim().toLocaleLowerCase() !==
                                      ability.label.trim().toLocaleLowerCase() && (
                                      <span className="character-secondary">
                                        {ability.record_title}
                                      </span>
                                    )}
                                </div>
                                {preparing ? (
                                  <ReferenceSelectionStatus
                                    selection={selection}
                                    generationChangedAction={
                                      reviewChangedGeneration
                                        ? {
                                            label: "Review latest changes",
                                            message: "Reference collection changed",
                                          }
                                        : undefined
                                    }
                                    onRetry={
                                      reviewChangedGeneration
                                        ? () => {
                                            selection.cancel();
                                            onReviewImport();
                                          }
                                        : undefined
                                    }
                                  />
                                ) : (
                                  <Space wrap>
                                    {review ? (
                                      <Button
                                        size="small"
                                        disabled={busy}
                                        aria-label={`${ability.record_title ? "Change link" : "Link rule"} for ${label}`}
                                        onClick={(event) =>
                                          startSearch(ability, event.currentTarget)
                                        }
                                      >
                                        {ability.record_title
                                          ? "Change link"
                                          : "Link rule…"}
                                      </Button>
                                    ) : ability.record_key && ability.record_title ? (
                                      <Dropdown
                                        trigger={["click"]}
                                        menu={{
                                          items: [
                                            { key: "change", label: "Change link" },
                                            ...(canUseAutomatic
                                              ? [
                                                  {
                                                    key: "automatic",
                                                    label: "Use automatic match",
                                                  },
                                                ]
                                              : []),
                                          ],
                                          onClick: ({ key }) => {
                                            if (key === "change")
                                              startSearch(
                                                ability,
                                                abilityRows.current
                                                  .get(ability.ability_key)
                                                  ?.querySelector<HTMLElement>(
                                                    ".character-link-options",
                                                  ),
                                              );
                                            else
                                              submitLink(ability, {
                                                kind: "automatic",
                                              });
                                          },
                                        }}
                                      >
                                        <Button
                                          size="small"
                                          className="character-link-options"
                                          aria-label={`Link options for ${label}`}
                                          type="text"
                                          icon={
                                            <Ellipsis size={16} aria-hidden="true" />
                                          }
                                          disabled={busy}
                                        ></Button>
                                      </Dropdown>
                                    ) : (
                                      <>
                                        <Button
                                          size="small"
                                          disabled={link.isPending}
                                          aria-label={`Search for ${label}`}
                                          onClick={(event) =>
                                            startSearch(ability, event.currentTarget)
                                          }
                                        >
                                          {ability.record_key
                                            ? "Find replacement"
                                            : "Find rule"}
                                        </Button>
                                        {canUseAutomatic && (
                                          <Button
                                            size="small"
                                            disabled={busy}
                                            onClick={() =>
                                              submitLink(ability, { kind: "automatic" })
                                            }
                                          >
                                            Use automatic match
                                          </Button>
                                        )}
                                      </>
                                    )}
                                  </Space>
                                )}
                              </div>
                            </List.Item>
                          );
                        }}
                      />
                    </section>
                  );
                })}
                {visible?.length === 0 && (
                  <p>
                    {filter ? "No matching abilities." : "No abilities in this export."}
                  </p>
                )}
              </>
            ) : doc ? (
              <Typography.Text type="secondary">
                This PC has no Pathbuilder reference yet.
              </Typography.Text>
            ) : null}
          </div>
        </BrowseReader>
      </div>
      {finding && selected && (
        <AbilityRulePicker
          key={selected.ability_key}
          label={abilityLabel(selected)}
          query={view.query ?? selected.label}
          onSearch={(query) => open({ ...view, query })}
          onBack={close}
          onLink={({ key, title, generation }) =>
            submitLink(
              selected,
              {
                kind: "manual",
                record_key: key,
                expected_artifact_generation: generation,
              },
              title,
            )
          }
          disabled={busy}
        />
      )}
    </div>
  );
}
