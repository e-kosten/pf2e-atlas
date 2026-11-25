import { isUnconfirmedMutation } from "../../api/apiError";
import { Button, Input, Space, Typography } from "antd";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useId, useMemo, useRef, useState } from "react";
import {
  getParty,
  getPreviewImportDetails,
  previewPathbuilderPc,
  savePathbuilderPc,
} from "../../api/atlasApi";
import type {
  PcReferenceView,
  PathbuilderPreviewView,
  PartyView,
  ImportChangesView,
} from "../../generated/atlas";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { useSavedWritesPaused } from "../../shared/feedback/SavedWritesPaused";
import { AtlasModal } from "../../shared/ui/overlays/AtlasModal";
import { refreshCharacters } from "./characterQueries";
import { CharacterSheet } from "./CharacterSheet";
import { PcAbilityBrowser, type AbilityLinkChange } from "./PcAbilityBrowser";

type ImportReview = {
  view: PathbuilderPreviewView;
  json: string;
  party?: PartyView;
};

export function PathbuilderImport({
  party,
  character,
  onClose,
  onSaved,
}: {
  party?: PartyView;
  character?: PcReferenceView;
  onClose: () => void;
  onSaved?: (key?: string) => void;
}) {
  const textareaId = useId();
  const [text, setText] = useState("");
  const [reviewing, setReviewing] = useState(false);
  const [review, setReview] = useState<{ data?: ImportReview; error?: Error }>({});
  const [closing, setClosing] = useState(false);
  const [ruleLinks, setRuleLinks] = useState<
    Record<string, { key: string; title: string }>
  >({});
  const [linksExpired, setLinksExpired] = useState(false);
  const reviewedGeneration = useRef<string>(undefined);
  const chooseLink = useMutation({
    mutationFn: async ({ abilityKey, selection, recordTitle }: AbilityLinkChange) => {
      if (selection.kind !== "manual" || !recordTitle) return;
      if (selection.expected_artifact_generation !== reviewedGeneration.current) {
        throw new Error(
          "Reference data changed. Review the character again before linking.",
        );
      }
      setRuleLinks((links) => ({
        ...links,
        [abilityKey]: { key: selection.record_key, title: recordTitle },
      }));
    },
  });
  const paused = useSavedWritesPaused();
  const client = useQueryClient();
  const preview = useMutation({
    mutationFn: async (json: string): Promise<ImportReview> => {
      const view = await previewPathbuilderPc(json, character?.character_key);
      const target = party ? await getParty(party.party_key) : undefined;
      return {
        view,
        json,
        party: target,
      };
    },
    onSuccess: (next) => {
      const { view } = next;
      if (reviewedGeneration.current !== view.artifact_generation) {
        if (Object.keys(ruleLinks).length) setLinksExpired(true);
        setRuleLinks({});
      }
      reviewedGeneration.current = view.artifact_generation;
      chooseLink.reset();
      save.reset();
      setReview({ data: next });
      setReviewing(true);
    },
    onError: (error) => setReview((current) => ({ ...current, error })),
    retry: false,
  });
  const save = useMutation({
    retry: false,
    mutationFn: savePathbuilderPc,
    onError: () => void refreshCharacters(client),
    onSuccess: async (saved) => {
      await refreshCharacters(client);
      if (!saved.available) return;
      setClosing(true);
      onSaved?.(saved.character_key);
    },
  });
  const uncertain = Boolean(save.error && isUnconfirmedMutation(save.error));
  const canReview = Boolean(save.error && !isUnconfirmedMutation(save.error));
  const busy = preview.isPending || save.isPending || chooseLink.isPending;
  const canReviewLink = !busy && !uncertain;
  function commit() {
    if (uncertain && save.variables) {
      save.mutate(save.variables);
      return;
    }
    const reviewed = review.data;
    if (!reviewed) return;
    const links = Object.entries(ruleLinks);
    const fields = {
      json: text,
      expected_artifact_generation: reviewed.view.artifact_generation,
      rule_links: links.length
        ? Object.fromEntries(links.map(([key, link]) => [key, link.key]))
        : undefined,
    };
    save.mutate({
      ...fields,
      request_id: crypto.randomUUID(),
      character_key: character?.character_key,
      expected_character_revision: reviewed.view.character_revision,
      party: reviewed.party && {
        party_key: reviewed.party.party_key,
        expected_revision: reviewed.party.revision,
      },
    });
  }
  const draft = review.data?.view;
  const sections = useMemo(
    () =>
      draft?.sections.map((section) => ({
        ...section,
        abilities: section.abilities.map((ability) => {
          const chosen = ruleLinks[ability.ability_key];
          return chosen
            ? {
                ...ability,
                record_key: chosen.key,
                record_title: chosen.title,
                manual: true,
              }
            : ability;
        }),
      })) ?? [],
    [draft, ruleLinks],
  );
  const abilities = sections.flatMap((section) => section.abilities);
  const linked = abilities.filter(
    (ability) => ability.record_key && ability.record_title,
  ).length;
  const retainedManualCount = abilities.filter(
    (ability) => ability.manual && !ruleLinks[ability.ability_key],
  ).length;
  const commitLabel = character
    ? "Update character"
    : party
      ? "Add to party"
      : "Add character";
  return (
    <AtlasModal
      open={!closing}
      title={character ? "New import from Pathbuilder" : "Import from Pathbuilder"}
      width={reviewing ? 1060 : undefined}
      layout={reviewing ? "browse" : "form"}
      onCancel={() => setClosing(true)}
      confirmDiscard={{
        when: () => Boolean(text.trim()),
        onDiscard: () => setClosing(true),
      }}
      afterClose={onClose}
      confirmLoading={busy}
      cancelButtonProps={{ disabled: busy }}
      closable={!busy}
      keyboard={!busy}
      mask={{ closable: !busy }}
      footer={(_, { CancelBtn }) =>
        reviewing ? (
          <Space wrap>
            <CancelBtn />
            <Button
              disabled={busy || uncertain}
              onClick={() => {
                setReviewing(false);
                save.reset();
              }}
            >
              Edit import
            </Button>
            <Button
              type="primary"
              loading={save.isPending}
              disabled={paused || busy || canReview || Boolean(review.error)}
              onClick={commit}
            >
              {uncertain ? "Retry to confirm" : commitLabel}
            </Button>
          </Space>
        ) : (
          <Space wrap>
            <CancelBtn />
            <Button
              type="primary"
              disabled={!text.trim() || busy}
              loading={preview.isPending ? { delay: 500 } : false}
              onClick={() => {
                save.reset();
                preview.mutate(text);
              }}
            >
              Continue
            </Button>
          </Space>
        )
      }
    >
      <div
        className={
          reviewing ? "character-import character-import--review" : "character-import"
        }
      >
        {!reviewing && (
          <div>
            {character && (
              <p>This import updates {character.name}’s saved build and abilities.</p>
            )}
            <label htmlFor={textareaId}>Pathbuilder JSON</label>
            <Input.TextArea
              id={textareaId}
              className="character-import__json"
              value={text}
              rows={16}
              autoComplete="off"
              spellCheck={false}
              disabled={busy}
              onChange={(event) => {
                setText(event.target.value);
                setRuleLinks({});
                setReview({});
                setLinksExpired(false);
                reviewedGeneration.current = undefined;
                preview.reset();
                save.reset();
              }}
            />
          </div>
        )}
        {review.error ? (
          <ErrorNotice
            error={review.error}
            title="Could not review character"
            description={reviewing ? "The previous review is still shown." : undefined}
            action={
              reviewing ? (
                <Button
                  disabled={busy}
                  loading={preview.isPending ? { delay: 500 } : false}
                  onClick={() => preview.mutate(text)}
                >
                  Retry review
                </Button>
              ) : undefined
            }
          />
        ) : (
          save.error && (
            <ErrorNotice
              error={save.error}
              title="Could not save character"
              action={
                canReview ? (
                  <Button
                    loading={preview.isPending ? { delay: 500 } : false}
                    disabled={preview.isPending}
                    onClick={() => preview.mutate(text)}
                  >
                    Review latest changes
                  </Button>
                ) : undefined
              }
            />
          )
        )}
        {save.data && !save.data.available && (
          <p role="status">
            The character created by this request was deleted. Close this import to
            start a new one.
          </p>
        )}
        {reviewing && draft && (
          <section className="character-import-review" aria-label="Import review">
            <Typography.Title level={4}>
              {draft.name} · Level {draft.level}
            </Typography.Title>
            {character &&
              (draft.previous_name !== draft.name ||
                draft.previous_level !== draft.level) && (
                <p>
                  Current character {draft.previous_name ?? character.name}
                  {draft.previous_level !== undefined &&
                    ` · Level ${draft.previous_level}`}
                </p>
              )}
            {draft.changes && (
              <p>
                {draft.changes.sections.length > 0
                  ? `Changes in ${draft.changes.sections.length} sections. See Import summary for details.`
                  : draft.changes.had_sheet
                    ? draft.changes.additional_fields_changed.length > 0
                      ? "Additional import details changed. See Import summary for details."
                      : "No changes in the exported build."
                    : "This import adds the first saved character sheet."}
                {draft.changes.removed_corrections.length > 0 &&
                  ` ${draft.changes.removed_corrections.length} saved ${draft.changes.removed_corrections.length === 1 ? "link" : "links"} will be removed.`}
              </p>
            )}
            {retainedManualCount > 0 && (
              <p>
                {retainedManualCount} saved{" "}
                {retainedManualCount === 1 ? "link retained" : "links retained"}
              </p>
            )}
            <p aria-live="polite">
              {linked} linked · {abilities.length - linked} unlinked
            </p>
            {linksExpired && (
              <p role="status">Atlas rules changed. Choose your draft links again.</p>
            )}
            <CharacterSheet
              key={`${draft.artifact_generation}-${draft.character_revision ?? "new"}`}
              generation={draft.artifact_generation}
              defaultTab="abilities"
              headingLevel={draft.level}
              sheet={draft.sheet}
              active={!closing}
              abilitySections={sections}
              priorDetailsChanged={Boolean(
                draft.changes?.additional_fields_changed.length,
              )}
              draftDetails={(signal) =>
                getPreviewImportDetails(
                  {
                    json: review.data!.json,
                    character_key: character?.character_key,
                    expected_character_revision: draft.character_revision,
                  },
                  signal,
                )
              }
              reviewSummary={
                <ImportSummary changes={draft.changes} sheet={draft.sheet} />
              }
              abilities={(available) => (
                <PcAbilityBrowser
                  generation={draft.artifact_generation}
                  active={available && !closing}
                  onReviewImport={
                    canReviewLink ? () => preview.mutate(text) : undefined
                  }
                  doc={{ imported: true, sections }}
                  link={chooseLink}
                  linkErrorAction={
                    <Button
                      loading={preview.isPending ? { delay: 500 } : false}
                      disabled={!canReviewLink}
                      onClick={() => preview.mutate(text)}
                    >
                      Review latest changes
                    </Button>
                  }
                  blocked={busy || uncertain || canReview || Boolean(review.error)}
                  review
                />
              )}
            />
          </section>
        )}
      </div>
    </AtlasModal>
  );
}

const summaryLabels = {
  stats: "Stats",
  progression: "Progression",
  magic: "Magic",
  inventory: "Inventory",
  companions: "Companions",
  proficiencies: "Proficiencies",
} as const;

function ImportSummary({
  changes,
  sheet,
}: {
  changes?: ImportChangesView;
  sheet: PathbuilderPreviewView["sheet"];
}) {
  return (
    <div className="character-import-summary">
      {changes ? (
        <>
          {!changes.had_sheet && (
            <p>
              This character has no previous sheet to compare. The import adds the first
              saved snapshot.
            </p>
          )}
          {changes.had_sheet &&
            changes.sections.length === 0 &&
            changes.additional_fields_changed.length === 0 && (
              <p>No changes in the exported build.</p>
            )}
          {changes.sections.map((section) => (
            <section key={section.label}>
              <h3>{section.label}</h3>
              {(
                [
                  ["Added", section.added],
                  ["Changed", section.changed],
                  ["Removed", section.removed],
                ] as const
              ).map(
                ([label, items]) =>
                  items.length > 0 && (
                    <details key={label}>
                      <summary>
                        {label} ({items.length})
                      </summary>
                      <ul>
                        {items.map((item, index) => (
                          <li key={`${item}-${index}`}>{item}</li>
                        ))}
                      </ul>
                    </details>
                  ),
              )}
            </section>
          ))}
          {changes.removed_corrections.length > 0 && (
            <section>
              <h3>Corrected links to be removed</h3>
              <ul>
                {changes.removed_corrections.map((label, index) => (
                  <li key={`${label}-${index}`}>{label}</li>
                ))}
              </ul>
            </section>
          )}
          {changes.additional_fields_changed.length > 0 && (
            <details>
              <summary>
                Changed additional details ({changes.additional_fields_changed.length})
              </summary>
              <p>Open Additional import details to review the affected values.</p>
              <ul>
                {changes.additional_fields_changed.map((field) => (
                  <li key={field}>{field}</li>
                ))}
              </ul>
            </details>
          )}
        </>
      ) : (
        <ul>
          {sheet.sections
            .filter((section) => section.groups.length > 0)
            .map((section) => {
              let entries = 0;
              for (const group of section.groups) {
                entries += group.rows.length;
                for (const rank of group.spell_ranks ?? []) {
                  for (const spell of [...rank.known, ...rank.prepared]) {
                    entries += spell.count;
                  }
                }
              }
              return (
                <li key={section.kind}>
                  {summaryLabels[section.kind]} {entries}{" "}
                  {entries === 1 ? "entry" : "entries"}
                </li>
              );
            })}
        </ul>
      )}
    </div>
  );
}
