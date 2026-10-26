import { useId, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import {
  Alert,
  App,
  Badge,
  Button,
  Checkbox,
  Collapse,
  Form,
  InputNumber,
  Modal,
  Select,
  Slider,
  Space,
  Spin,
  Tooltip,
  Typography,
  theme,
} from "antd";
import { SlidersHorizontal } from "lucide-react";
import type {
  SearchRanking,
  SimilarityPartView,
  SimilaritySeed,
} from "../../generated/atlas";
import {
  INVALID_SIMILARITY_SETTINGS,
  UNKNOWN_SEARCH_RANKING,
  type SearchFormState,
} from "../filters/searchState";
import { getSimilarityParts } from "../../api/atlasApi";
import { useSearchProfile } from "./SearchProfileContext";
import {
  DEFAULT_SIMILARITY_PREFERENCES,
  type SimilarityPreferences,
} from "./searchPreferences";
import {
  selectedSimilarityParts,
  similarityFromPreferences,
  similarityLabel,
  similarityPreferences,
} from "./similarityState";
import { isArtifactGenerationChanged } from "../queries/immutableReferenceQueries";
import { HelpTooltip } from "../ui/HelpTooltip";

type SettingsDraft = {
  model: string | null;
  ranking: SearchRanking;
  similarity: SimilarityPreferences;
  parts: string[] | null;
  generation?: string;
};

export function SearchSettings({
  search,
  seed = null,
  generation = null,
  onApply,
  disabled = false,
}: {
  search: SearchFormState;
  seed?: SimilaritySeed | null;
  generation?: string | null;
  onApply: (search: SearchFormState, model: string) => void;
  disabled?: boolean;
}) {
  const profile = useSearchProfile();
  const { message } = App.useApp();
  const { token } = theme.useToken();
  const id = useId();
  const trait = seed?.kind === "trait";
  const preferredSimilarity = trait
    ? profile.preferences.traitSimilarity
    : profile.preferences.recordSimilarity;
  const appliedSimilarity =
    search.similarity ?? similarityFromPreferences(preferredSimilarity, trait);
  const appliedPreferences = similarityPreferences(
    appliedSimilarity,
    preferredSimilarity,
  );
  const appliedParts = selectedSimilarityParts(appliedSimilarity);
  const ranking = search.ranking ?? profile.preferences.ranking;
  const [draft, setDraft] = useState<SettingsDraft | null>(null);
  const [partsOpen, setPartsOpen] = useState(false);
  const [reloadInventory, setReloadInventory] = useState(false);
  const inventory = useQuery({
    queryKey: [
      "similarity-parts",
      seed,
      draft?.model,
      reloadInventory ? null : (draft?.generation ?? generation),
    ],
    enabled:
      draft !== null &&
      seed !== null &&
      draft.similarity.method !== "baseline" &&
      (partsOpen || draft.parts !== null) &&
      draft.model !== null,
    staleTime: Infinity,
    queryFn: ({ signal }) =>
      getSimilarityParts(
        {
          seed: seed!,
          search_profile: draft!.model!,
          expected_artifact_generation: reloadInventory
            ? null
            : (draft!.generation ?? generation),
        },
        signal,
      ),
  });
  const allParts = inventory.data?.parts ?? [];
  const available = allParts.filter((part) => part.has_vectors);
  const selectedCount = draft?.parts === null ? available.length : draft?.parts.length;
  const usesParts = draft !== null && draft.similarity.method !== "baseline";
  const invalidParts = usesParts && draft?.parts?.length === 0;
  const staleParts =
    usesParts &&
    draft?.parts !== null &&
    draft?.parts !== undefined &&
    inventory.data !== undefined &&
    draft.parts.some(
      (selector) => !available.some((part) => part.selector === selector),
    );
  const uncheckedParts =
    draft !== null &&
    draft.similarity.method !== "baseline" &&
    draft.parts !== null &&
    (!inventory.data || inventory.isFetching || inventory.error !== null);
  const invalidDecay =
    draft?.similarity.method === "passages" &&
    (!Number.isFinite(draft.similarity.decay) ||
      draft.similarity.decay < 0 ||
      draft.similarity.decay > 1);
  const customized =
    (profile.id !== null &&
      profile.defaultId !== null &&
      profile.id !== profile.defaultId) ||
    (seed
      ? appliedPreferences.method !== "passages" ||
        appliedPreferences.decay !== 0.5 ||
        appliedPreferences.candidateParts !== "any" ||
        appliedParts !== null
      : ranking !== "score");
  const validModel = profile.profiles.some(
    (item) => item.id === draft?.model && item.stored_vectors,
  );
  const modelLabel =
    profile.profiles.find((item) => item.id === profile.id)?.label ?? profile.id;
  const summary = [
    modelLabel,
    seed
      ? similarityLabel(appliedSimilarity, trait)
      : ranking === "rrf"
        ? "Rank-based ranking"
        : "Score-based ranking",
    seed && appliedSimilarity.method !== "baseline"
      ? appliedParts
        ? `${appliedParts.length} selected ${appliedParts.length === 1 ? "part" : "parts"}`
        : "All parts"
      : null,
  ]
    .filter(Boolean)
    .join("; ");
  function openSettings() {
    setPartsOpen(appliedParts !== null);
    setReloadInventory(false);
    setDraft({
      model: profile.id,
      ranking,
      similarity: appliedPreferences,
      parts: appliedParts,
      generation: search.similarityGeneration,
    });
  }
  function apply() {
    if (
      !draft?.model ||
      !validModel ||
      invalidParts ||
      staleParts ||
      uncheckedParts ||
      invalidDecay
    )
      return;
    const nextSimilarity = seed
      ? similarityFromPreferences(draft.similarity, trait, draft.parts)
      : search.similarity;
    const nextPreferences = {
      ...profile.preferences,
      model: draft.model,
      ...(seed
        ? trait
          ? { traitSimilarity: draft.similarity }
          : { recordSimilarity: draft.similarity }
        : { ranking: draft.ranking }),
    };
    const saved = profile.applyPreferences(nextPreferences);
    onApply(
      {
        ...search,
        ranking: seed ? search.ranking : draft.ranking,
        ...(seed
          ? {
              similarity: nextSimilarity,
              similarityGeneration:
                nextSimilarity?.method !== "baseline" && draft.parts !== null
                  ? (draft.generation ?? inventory.data?.artifact_generation)
                  : undefined,
            }
          : {}),
        filterError:
          search.filterError === UNKNOWN_SEARCH_RANKING ||
          search.filterError === INVALID_SIMILARITY_SETTINGS
            ? null
            : search.filterError,
      },
      draft.model,
    );
    setDraft(null);
    if (!saved)
      void message.warning(
        "Search settings apply to this tab, but could not be remembered in this browser.",
      );
  }
  const modelOptions: { value: string; label: string; disabled?: boolean }[] =
    profile.profiles
      .filter((item) => item.stored_vectors)
      .map((item) => ({ value: item.id, label: item.label }));
  if (draft?.model && !modelOptions.some((item) => item.value === draft.model))
    modelOptions.push({ value: draft.model, label: draft.model, disabled: true });
  const selectionSummary =
    draft?.parts === null
      ? "All parts"
      : `${draft?.parts.length ?? 0} selected ${draft?.parts.length === 1 ? "part" : "parts"}`;
  const methodHelp =
    draft?.similarity.method === "passages"
      ? "Passage matches compares each selected part with its closest matching content in a result, then combines the part matches using Match decay."
      : draft?.similarity.method === "content-average"
        ? "Content average blends the meaning of the selected parts into an overall comparison. With Corresponding parts, it blends and compares each matching type of part separately."
        : trait
          ? "Trait definition compares the trait’s definition with content in other records. It uses the trait’s label when no definition is available."
          : "Name and identity compares the record’s name and identity information with content in other records.";
  return (
    <>
      <Tooltip title={`Search settings: ${summary}`}>
        <Badge dot={customized} color={token.colorPrimaryText}>
          <Button
            icon={<SlidersHorizontal size={18} />}
            aria-label="Search settings"
            aria-describedby={`${id}-summary`}
            disabled={disabled || profile.loading}
            onClick={openSettings}
          />
        </Badge>
      </Tooltip>
      <span id={`${id}-summary`} className="sr-only">
        {customized ? "Customized. " : ""}
        {summary}
      </span>
      <Modal
        title="Search settings"
        open={draft !== null}
        onCancel={() => setDraft(null)}
        onOk={apply}
        okText="Apply"
        width="min(560px, calc(100vw - 32px))"
        destroyOnHidden
        styles={{ body: { maxHeight: "min(70dvh, 650px)", overflowY: "auto" } }}
        footer={(_, { OkBtn, CancelBtn }) => (
          <div className="search-settings__footer">
            <Button
              onClick={() => {
                setDraft((current) =>
                  current
                    ? {
                        ...current,
                        model: profile.defaultId,
                        ranking: "score",
                        similarity: DEFAULT_SIMILARITY_PREFERENCES,
                        parts: null,
                        generation: undefined,
                      }
                    : null,
                );
              }}
            >
              Reset
            </Button>
            <Space>
              <CancelBtn />
              <OkBtn />
            </Space>
          </div>
        )}
        okButtonProps={{
          disabled:
            !draft?.model ||
            !validModel ||
            invalidParts ||
            staleParts ||
            uncheckedParts ||
            invalidDecay,
        }}
      >
        {draft && (
          <Form layout="vertical" component="div" className="search-settings">
            {profile.error && draft.model === profile.id && (
              <Alert type="error" title={profile.error} />
            )}
            {!validModel && !(profile.error && draft.model === profile.id) && (
              <Alert
                type="error"
                title={
                  modelOptions.length
                    ? "Choose an available embedding model, or use Reset."
                    : "No search model is stored in this collection. Check setup to continue."
                }
              />
            )}
            {profile.profiles.filter((item) => item.stored_vectors).length > 1 && (
              <Form.Item
                label={
                  <SettingLabel
                    label="Embedding model"
                    help="Changes how the search compares meaning, so choosing another model can change which results appear."
                  />
                }
                htmlFor={`${id}-model`}
              >
                <Select
                  id={`${id}-model`}
                  aria-label="Search model"
                  value={draft.model}
                  options={modelOptions}
                  onChange={(model) => setDraft({ ...draft, model })}
                />
              </Form.Item>
            )}
            {!seed ? (
              <Form.Item
                label={
                  <SettingLabel
                    label="Ranking"
                    help={
                      draft.ranking === "rrf"
                        ? "Rank-based combines the result positions of keyword and meaning-based matches."
                        : "Score-based combines the strength of keyword and meaning-based matches."
                    }
                  />
                }
                htmlFor={`${id}-ranking`}
              >
                <Select
                  id={`${id}-ranking`}
                  aria-label="Search ranking"
                  value={draft.ranking}
                  options={[
                    { value: "score", label: "Score-based" },
                    { value: "rrf", label: "Rank-based" },
                  ]}
                  onChange={(next) => setDraft({ ...draft, ranking: next })}
                />
              </Form.Item>
            ) : (
              <>
                <Form.Item
                  label={<SettingLabel label="Comparison method" help={methodHelp} />}
                  htmlFor={`${id}-method`}
                >
                  <Select
                    id={`${id}-method`}
                    aria-label="Comparison method"
                    value={draft.similarity.method}
                    options={[
                      { value: "passages", label: "Passage matches" },
                      { value: "content-average", label: "Content average" },
                      {
                        value: "baseline",
                        label: trait ? "Trait definition" : "Name and identity",
                      },
                    ]}
                    onChange={(method) =>
                      setDraft({
                        ...draft,
                        similarity: { ...draft.similarity, method },
                      })
                    }
                  />
                </Form.Item>
                {draft.similarity.method === "passages" && (
                  <Form.Item
                    label={
                      <SettingLabel
                        label="Match decay"
                        help="Parts are ordered by match strength for each result. 0 uses only the strongest part; 1 gives all selected parts equal weight. Values in between give each additional part less weight."
                      />
                    }
                    validateStatus={invalidDecay ? "error" : undefined}
                    help={invalidDecay ? "Enter a number from 0 through 1." : undefined}
                  >
                    <div className="search-settings__decay">
                      <Slider
                        aria-label="Match decay slider"
                        min={0}
                        max={1}
                        step={0.05}
                        value={invalidDecay ? 0 : draft.similarity.decay}
                        onChange={(decay) =>
                          setDraft({
                            ...draft,
                            similarity: { ...draft.similarity, decay },
                          })
                        }
                      />
                      <InputNumber
                        aria-label="Match decay"
                        min={0}
                        max={1}
                        step={0.05}
                        value={
                          Number.isFinite(draft.similarity.decay)
                            ? draft.similarity.decay
                            : null
                        }
                        onInput={(text) => {
                          setDraft({
                            ...draft,
                            similarity: {
                              ...draft.similarity,
                              decay: text.trim() ? Number(text) : Number.NaN,
                            },
                          });
                        }}
                        onChange={(decay) => {
                          setDraft({
                            ...draft,
                            similarity: {
                              ...draft.similarity,
                              decay: typeof decay === "number" ? decay : Number.NaN,
                            },
                          });
                        }}
                      />
                    </div>
                    <div className="search-settings__scale">
                      <span>Strongest part</span>
                      <span>All parts equally</span>
                    </div>
                    {!invalidDecay && (
                      <Typography.Text type="secondary">
                        {selectedCount === 1
                          ? "One part selected; decay has no effect."
                          : `Contributions: 1, ${Number(draft.similarity.decay.toFixed(4))}, ${Number((draft.similarity.decay ** 2).toFixed(4))}…`}
                      </Typography.Text>
                    )}
                  </Form.Item>
                )}
                {draft.similarity.method === "baseline" ? (
                  <Typography.Paragraph type="secondary">
                    {trait
                      ? "Uses the trait’s definition, or its label when no definition is available."
                      : "Uses the record’s name and identity information."}
                  </Typography.Paragraph>
                ) : (
                  <Collapse
                    activeKey={partsOpen ? ["parts"] : []}
                    onChange={(keys) => setPartsOpen(keys.includes("parts"))}
                    items={[
                      {
                        key: "parts",
                        label: (
                          <SettingLabel
                            label="Compare parts"
                            help={`Choose which sections of this ${trait ? "trait" : "record"} supply the comparison content. All parts uses every section that can be compared. This selection applies only to this ${trait ? "trait" : "record"}.`}
                          />
                        ),
                        extra: (
                          <Typography.Text type="secondary">
                            {selectionSummary}
                          </Typography.Text>
                        ),
                        children: (
                          <>
                            {inventory.isPending && <Spin size="small" />}
                            {inventory.error && (
                              <Alert
                                type="error"
                                title="Unable to load comparison parts"
                                description={inventory.error.message}
                                action={
                                  <Button
                                    onClick={() => {
                                      if (
                                        isArtifactGenerationChanged(inventory.error)
                                      ) {
                                        setReloadInventory(true);
                                        setDraft({
                                          ...draft,
                                          parts: draft.parts === null ? null : [],
                                          generation: undefined,
                                        });
                                      } else void inventory.refetch();
                                    }}
                                  >
                                    {isArtifactGenerationChanged(inventory.error)
                                      ? "Reload parts"
                                      : "Retry"}
                                  </Button>
                                }
                              />
                            )}
                            {inventory.data && (
                              <>
                                <Checkbox
                                  checked={draft.parts === null}
                                  indeterminate={
                                    draft.parts !== null &&
                                    draft.parts.length > 0 &&
                                    draft.parts.length < available.length
                                  }
                                  onChange={(event) =>
                                    setDraft({
                                      ...draft,
                                      parts: event.target.checked ? null : [],
                                      generation: inventory.data.artifact_generation,
                                    })
                                  }
                                >
                                  All parts
                                </Checkbox>
                                <PartChoices
                                  parts={allParts}
                                  selected={draft.parts}
                                  onChange={(parts) =>
                                    setDraft({
                                      ...draft,
                                      parts,
                                      generation: inventory.data.artifact_generation,
                                    })
                                  }
                                />
                                {(invalidParts || staleParts) && (
                                  <Typography.Paragraph type="danger" role="alert">
                                    {staleParts
                                      ? "Some selected parts are unavailable. Choose the parts again."
                                      : "Select at least one part."}
                                  </Typography.Paragraph>
                                )}
                                {allParts.some((part) => part.label_only) && (
                                  <Typography.Paragraph type="secondary">
                                    This trait has no definition text; comparison uses
                                    its label.
                                  </Typography.Paragraph>
                                )}
                              </>
                            )}
                            {!trait && (
                              <Form.Item
                                label={
                                  <SettingLabel
                                    label="Candidate content"
                                    help={
                                      draft.similarity.candidateParts ===
                                      "corresponding"
                                        ? "Only parts with matching purposes can contribute, including across record kinds. For example, descriptions compare with descriptions. If a result lacks that type of part, it contributes no match."
                                        : "Any selected source part can match any content in a result, even when the sections serve different purposes."
                                    }
                                  />
                                }
                                htmlFor={`${id}-candidate`}
                              >
                                <Select
                                  id={`${id}-candidate`}
                                  aria-label="Candidate content"
                                  value={draft.similarity.candidateParts}
                                  options={[
                                    { value: "any", label: "Any content" },
                                    {
                                      value: "corresponding",
                                      label: "Corresponding parts",
                                    },
                                  ]}
                                  onChange={(candidateParts) =>
                                    setDraft({
                                      ...draft,
                                      similarity: {
                                        ...draft.similarity,
                                        candidateParts,
                                      },
                                    })
                                  }
                                />
                              </Form.Item>
                            )}
                          </>
                        ),
                      },
                    ]}
                  />
                )}
              </>
            )}
          </Form>
        )}
      </Modal>
    </>
  );
}

function SettingLabel({ label, help }: { label: string; help: string }) {
  return (
    <span className="search-settings__label">
      <span>{label}</span>
      <HelpTooltip label={`About ${label.toLowerCase()}`}>{help}</HelpTooltip>
    </span>
  );
}

function PartChoices({
  parts,
  selected,
  onChange,
}: {
  parts: SimilarityPartView[];
  selected: string[] | null;
  onChange: (parts: string[] | null) => void;
}) {
  const groups = new Map<string, { label: string; parts: SimilarityPartView[] }>();
  for (const part of parts) {
    const key = JSON.stringify(part.owners);
    const group = groups.get(key);
    if (group) group.parts.push(part);
    else groups.set(key, { label: part.owner_label || "Main entry", parts: [part] });
  }
  const available = parts.filter((part) => part.has_vectors);
  const chosen = new Set(selected ?? available.map((part) => part.selector));
  function toggle(selectors: string[], checked: boolean) {
    const allowed = new Set(available.map((part) => part.selector));
    const next = new Set([...chosen].filter((selector) => allowed.has(selector)));
    for (const selector of selectors)
      if (checked) next.add(selector);
      else next.delete(selector);
    onChange(next.size === available.length ? null : [...next]);
  }
  return (
    <div className="search-settings__parts">
      {[...groups].map(([key, { label: owner, parts: entries }]) => {
        const selectors = entries
          .filter((part) => part.has_vectors)
          .map((part) => part.selector);
        const count = selectors.filter((selector) => chosen.has(selector)).length;
        return (
          <div key={key} className="search-settings__part-group">
            <Checkbox
              checked={selectors.length > 0 && count === selectors.length}
              indeterminate={count > 0 && count < selectors.length}
              disabled={!selectors.length}
              onChange={(event) => toggle(selectors, event.target.checked)}
            >
              <strong>{owner}</strong>
            </Checkbox>
            <div className="search-settings__part-fields">
              {entries.map((part) => (
                <Checkbox
                  key={part.selector}
                  checked={chosen.has(part.selector)}
                  disabled={!part.has_vectors}
                  onChange={(event) => toggle([part.selector], event.target.checked)}
                >
                  {part.label}
                  {!part.has_vectors ? " (no stored vector)" : ""}
                </Checkbox>
              ))}
            </div>
          </div>
        );
      })}
    </div>
  );
}
