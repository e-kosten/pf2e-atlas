import { isUnconfirmedMutation } from "../../api/apiError";
import { Button, Input, Space, Typography } from "antd";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useId, useRef, useState } from "react";
import {
  addEncounterCharacter,
  getEncounter,
  getParties,
  previewPathbuilderPc,
  savePathbuilderPc,
} from "../../api/atlasApi";
import type {
  CharacterMutationView,
  EncounterCharacterMutationView,
  PcReferenceView,
  PcAbilityView,
  PcAbilitySelection,
  PathbuilderPreviewView,
  PartyView,
} from "../../generated/atlas";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { useSavedWritesPaused } from "../../shared/feedback/SavedWritesPaused";
import { AtlasModal } from "../../shared/ui/overlays/AtlasModal";
import { refreshCharacters } from "./characterQueries";
import { PcAbilityBrowser } from "./PcAbilityBrowser";

type ImportReview = {
  view: PathbuilderPreviewView;
  party?: PartyView;
  encounterRevision?: number;
};

export function PathbuilderImport({
  party,
  character,
  encounter,
  onClose,
  onSaved,
  onEncounterAdded,
}: {
  party?: PartyView;
  character?: PcReferenceView;
  encounter?: { encounterKey: string; revision: number };
  onClose: () => void;
  onSaved?: (key?: string) => void;
  onEncounterAdded?: (result: { characterKey: string; participantKey: string }) => void;
}) {
  const textareaId = useId();
  const [requestId] = useState(() => crypto.randomUUID());
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
    mutationFn: async ({
      ability,
      selection,
      recordTitle,
    }: {
      ability: PcAbilityView;
      selection: PcAbilitySelection;
      recordTitle?: string;
    }) => {
      if (selection.kind !== "manual" || !recordTitle) return;
      if (selection.expected_artifact_generation !== reviewedGeneration.current) {
        throw new Error(
          "Reference data changed. Review the character again before linking.",
        );
      }
      setRuleLinks((links) => ({
        ...links,
        [ability.ability_key]: { key: selection.record_key, title: recordTitle },
      }));
    },
  });
  const paused = useSavedWritesPaused();
  const client = useQueryClient();
  const preview = useMutation({
    mutationFn: async (json: string): Promise<ImportReview> => {
      const view = await previewPathbuilderPc(json, character?.character_key);
      const target = party
        ? (await getParties()).parties.find(
            (item) => item.party_key === party.party_key,
          )
        : undefined;
      if (party && !target)
        throw new Error("This party is no longer available. Your export is retained.");
      return {
        view,
        party: target,
        encounterRevision: encounter
          ? (await getEncounter(encounter.encounterKey)).revision
          : undefined,
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
  type ImportIntent = {
    review: ImportReview;
    json: string;
    ruleLinks: Record<string, string>;
  };
  const submitted = useRef<ImportIntent | null>(null);
  const save = useMutation({
    retry: false,
    mutationFn: async (
      latest: ImportIntent,
    ): Promise<CharacterMutationView | EncounterCharacterMutationView> => {
      submitted.current ??= latest;
      const { review: reviewed, json, ruleLinks } = submitted.current;
      const submittedLinks = Object.keys(ruleLinks).length ? ruleLinks : undefined;
      if (encounter) {
        const added = await addEncounterCharacter(encounter.encounterKey, {
          request_id: requestId,
          expected_encounter_revision: reviewed.encounterRevision!,
          source: {
            kind: "pathbuilder",
            json,
            rule_links: submittedLinks,
            expected_artifact_generation: reviewed.view.artifact_generation,
          },
        });
        return added;
      }
      return savePathbuilderPc({
        request_id: requestId,
        json,
        rule_links: submittedLinks,
        character_key: character?.character_key,
        expected_character_revision: reviewed.view.character_revision,
        party: reviewed.party
          ? {
              party_key: reviewed.party.party_key,
              expected_revision: reviewed.party.revision,
            }
          : undefined,
        expected_artifact_generation: reviewed.view.artifact_generation,
      });
    },
    onError: (error) => {
      if (!isUnconfirmedMutation(error)) submitted.current = null;
      void refreshCharacters(client);
    },
    onSuccess: async (saved) => {
      await refreshCharacters(client);
      if (!saved.available) return;
      setClosing(true);
      if ("participant_key" in saved)
        onEncounterAdded?.({
          characterKey: saved.character_key,
          participantKey: saved.participant_key,
        });
      onSaved?.(saved.character_key);
    },
  });
  const uncertain = Boolean(save.error && isUnconfirmedMutation(save.error));
  const canReview = Boolean(save.error && !isUnconfirmedMutation(save.error));
  const busy = preview.isPending || save.isPending || chooseLink.isPending;
  const draft = review.data?.view;
  const stageAbility = (ability: PcAbilityView): PcAbilityView => {
    const chosen = ruleLinks[ability.ability_key];
    return chosen
      ? { ...ability, record_key: chosen.key, record_title: chosen.title, manual: true }
      : ability;
  };
  const abilities = draft?.abilities.map(stageAbility) ?? [];
  const linked = abilities.filter(
    (ability) => ability.record_key && ability.record_title,
  ).length;
  const retainedManualCount =
    draft?.abilities.filter(
      (ability) => ability.manual && !ruleLinks[ability.ability_key],
    ).length ?? 0;
  const commitLabel = character
    ? "Refresh character"
    : encounter
      ? "Add to encounter"
      : party
        ? "Add to party"
        : "Add character";
  return (
    <AtlasModal
      open={!closing}
      title={character ? `Refresh ${character.name}` : "Import from Pathbuilder"}
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
              onClick={() => {
                if (!review.data) return;
                save.mutate(
                  submitted.current ?? {
                    review: review.data,
                    json: text,
                    ruleLinks: Object.fromEntries(
                      Object.entries(ruleLinks).map(([key, link]) => [key, link.key]),
                    ),
                  },
                );
              }}
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
            <label htmlFor={textareaId}>Pathbuilder JSON</label>
            <Input.TextArea
              id={textareaId}
              value={text}
              rows={8}
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
                    onClick={() => {
                      submitted.current = null;
                      preview.mutate(text);
                    }}
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
            <PcAbilityBrowser
              key={draft.artifact_generation}
              generation={draft.artifact_generation}
              active={!closing}
              onReviewImport={
                !busy && !uncertain ? () => preview.mutate(text) : undefined
              }
              doc={{
                imported: true,
                abilities,
                sections: draft.sections.map((section) => ({
                  ...section,
                  abilities: section.abilities.map(stageAbility),
                })),
              }}
              link={chooseLink}
              linkErrorAction={
                <Button
                  loading={preview.isPending ? { delay: 500 } : false}
                  disabled={preview.isPending}
                  onClick={() => preview.mutate(text)}
                >
                  Review latest changes
                </Button>
              }
              blocked={busy || uncertain || canReview || Boolean(review.error)}
              review
            />
          </section>
        )}
      </div>
    </AtlasModal>
  );
}
