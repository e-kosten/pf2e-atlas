import { useId, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import {
  Alert,
  App,
  Button,
  Checkbox,
  Collapse,
  Form,
  Grid,
  InputNumber,
  Modal,
  Select,
  Segmented,
  Slider,
  Space,
  Spin,
  Switch,
  Tooltip,
  Typography,
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
import { AtlasApiError, getSimilarityParts } from "../../api/atlasApi";
import { useSearchProfile } from "./SearchProfileContext";
import {
  DEFAULT_SIMILARITY_PREFERENCES,
  DEFAULT_TRAIT_SIMILARITY_PREFERENCES,
  type SimilarityPreferences,
} from "./searchPreferences";
import {
  selectedSimilarityParts,
  similarityFromPreferences,
  similarityLabel,
  similarityPreferences,
} from "./similarityState";
import { isArtifactGenerationChanged } from "../queries/immutableReferenceQueries";
import { destinationHref, isPrimaryActivation } from "../../app/navigation";
import { recordDestination, traitDestination } from "../../app/destinations";
import {
  SimilaritySourceDocument,
  type SimilaritySourceTarget,
} from "./SimilaritySourceDocument";
import { HelpTooltip } from "../ui/HelpTooltip";

type SettingsDraft = {
  seedIdentity: string;
  model: string | null;
  ranking: SearchRanking;
  showSimilarityDetails: boolean;
  showSmallerWeights: boolean;
  similarity: SimilarityPreferences;
  parts: string[] | null;
  generation?: string;
};

export function SearchSettings({
  search,
  seed = null,
  seedTitle,
  generation = null,
  onApply,
  disabled = false,
}: {
  search: SearchFormState;
  seed?: SimilaritySeed | null;
  seedTitle?: string;
  generation?: string | null;
  onApply: (search: SearchFormState, model: string) => void;
  disabled?: boolean;
}) {
  const profile = useSearchProfile();
  const { message } = App.useApp();
  const id = useId();
  const wide = Boolean(Grid.useBreakpoint().lg);
  const [sourceView, setSourceView] = useState("sections");
  const [sourceTarget, setSourceTarget] = useState<SimilaritySourceTarget>(null);
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
  const [sourceVisited, setSourceVisited] = useState(false);
  const [sourceSnapshot, setSourceSnapshot] = useState<{
    seedIdentity: string;
    generation: string;
  } | null>(null);
  const seedIdentity = JSON.stringify(seed);
  if (draft && draft.seedIdentity !== seedIdentity) setDraft(null);
  const [reloadInventory, setReloadInventory] = useState(0);
  const [sourceRetired, setSourceRetired] = useState(false);
  const inventory = useQuery({
    queryKey: [
      "similarity-parts",
      seed,
      draft?.model,
      reloadInventory ? null : (draft?.generation ?? generation),
      reloadInventory,
    ],
    enabled:
      draft !== null &&
      seed !== null &&
      draft.similarity.method !== "baseline" &&
      (partsOpen || draft.parts !== null) &&
      draft.model !== null,
    staleTime: Infinity,
    queryFn: async ({ signal }) => {
      const value = await getSimilarityParts(
        {
          seed: seed!,
          search_profile: draft!.model!,
          expected_artifact_generation: reloadInventory
            ? null
            : (draft!.generation ?? generation),
        },
        signal,
      );
      if (
        JSON.stringify(value.seed) !== JSON.stringify(seed) ||
        value.search_profile !== draft!.model
      )
        throw new Error("The comparison sections do not match this source. Retry.");
      const expected = reloadInventory ? null : (draft!.generation ?? generation);
      if (expected && value.artifact_generation !== expected)
        throw new AtlasApiError(409, "The reference collection changed.", {
          code: "artifact_generation_changed",
          message: "The reference collection changed.",
          recoverable_action: "refresh_results",
        });
      return value;
    },
  });
  const inventoryRetired =
    sourceRetired || isArtifactGenerationChanged(inventory.error);
  if (isArtifactGenerationChanged(inventory.error) && !sourceRetired) {
    setSourceRetired(true);
    setSourceSnapshot(null);
    setSourceTarget(null);
  }
  const allParts = inventory.data?.parts ?? [];
  const available = allParts.filter((part) => part.has_vectors);
  const selectedCount = draft?.parts === null ? available.length : draft?.parts.length;
  const usesParts = draft !== null && draft.similarity.method !== "baseline";
  const showingSource = Boolean(seed && usesParts && partsOpen);
  if (
    draft &&
    !inventoryRetired &&
    inventory.data &&
    (sourceSnapshot?.seedIdentity !== seedIdentity ||
      sourceSnapshot.generation !== inventory.data.artifact_generation)
  )
    setSourceSnapshot({ seedIdentity, generation: inventory.data.artifact_generation });
  const sourceGeneration = inventoryRetired
    ? null
    : reloadInventory > 0
      ? (inventory.data?.artifact_generation ?? null)
      : (inventory.data?.artifact_generation ??
        (sourceSnapshot?.seedIdentity === seedIdentity
          ? sourceSnapshot.generation
          : null) ??
        draft?.generation ??
        generation);
  const sourceIdentity = JSON.stringify([seed, sourceGeneration]);
  const [targetIdentity, setTargetIdentity] = useState<string | null>(null);
  const currentTarget = targetIdentity === sourceIdentity ? sourceTarget : null;
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
    (inventoryRetired ||
      !inventory.data ||
      inventory.isFetching ||
      inventory.error !== null);
  const invalidDecay =
    draft?.similarity.method === "passages" &&
    (!Number.isFinite(draft.similarity.decay) ||
      draft.similarity.decay < 0 ||
      draft.similarity.decay > 1);
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
        ? `${appliedParts.length} selected ${appliedParts.length === 1 ? "section" : "sections"}`
        : "All sections"
      : null,
  ]
    .filter(Boolean)
    .join("; ");
  function openSettings() {
    setSourceView("sections");
    setSourceTarget(null);
    setSourceSnapshot(null);
    setPartsOpen(appliedParts !== null);
    setSourceVisited(appliedParts !== null);
    setReloadInventory(0);
    setSourceRetired(false);
    setDraft({
      seedIdentity,
      model: profile.id,
      ranking,
      showSimilarityDetails: profile.preferences.showSimilarityDetails,
      showSmallerWeights: profile.preferences.showSmallerWeights,
      similarity: appliedPreferences,
      parts: appliedParts,
      generation: search.similarityGeneration,
    });
  }
  function reloadSections() {
    setSourceRetired(false);
    setSourceTarget(null);
    setTargetIdentity(null);
    setSourceSnapshot(null);
    setReloadInventory((current) => current + 1);
    setDraft((current) =>
      current
        ? {
            ...current,
            parts: current.parts === null ? null : [],
            generation: undefined,
          }
        : null,
    );
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
    const presentationOnly =
      (draft.showSimilarityDetails !== profile.preferences.showSimilarityDetails ||
        draft.showSmallerWeights !== profile.preferences.showSmallerWeights) &&
      !search.filterError &&
      draft.model === profile.id &&
      draft.ranking === ranking &&
      draft.similarity.method === appliedPreferences.method &&
      draft.similarity.decay === appliedPreferences.decay &&
      draft.similarity.candidateParts === appliedPreferences.candidateParts &&
      JSON.stringify(draft.parts) === JSON.stringify(appliedParts) &&
      draft.generation === search.similarityGeneration;
    const nextPreferences = {
      ...profile.preferences,
      model: draft.model,
      showSimilarityDetails: draft.showSimilarityDetails,
      showSmallerWeights: draft.showSmallerWeights,
      ...(seed
        ? trait
          ? { traitSimilarity: draft.similarity }
          : { recordSimilarity: draft.similarity }
        : { ranking: draft.ranking }),
    };
    const saved = profile.applyPreferences(nextPreferences);
    if (!presentationOnly)
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
      ? "All sections"
      : `${draft?.parts.length ?? 0} selected ${draft?.parts.length === 1 ? "section" : "sections"}`;
  const sourcePossessive = seedTitle ? `${seedTitle}’s` : "the record’s";
  const partsHelp = `Choose which sections of this ${trait ? "trait" : "record"} to use when finding similar records.`;
  const methodHelp =
    draft?.similarity.method === "passages"
      ? "Looks for passages similar to each selected section. Each selected section can match a different passage in the results."
      : draft?.similarity.method === "content-average"
        ? "Averages the selected sections into an overall comparison. Specific details can become less prominent when several different sections are combined."
        : trait
          ? "Uses the trait’s definition as the starting point, or its name when no definition is available."
          : `Uses ${sourcePossessive} name and identifying information. Its description and abilities aren’t used.`;
  return (
    <>
      <Tooltip title={`Search settings: ${summary}`}>
        <Button
          icon={<SlidersHorizontal size={18} />}
          aria-label="Search settings"
          aria-describedby={`${id}-summary`}
          disabled={disabled || profile.loading}
          onClick={openSettings}
        />
      </Tooltip>
      <span id={`${id}-summary`} className="sr-only">
        {summary}
      </span>
      <Modal
        title="Search settings"
        open={draft !== null}
        onCancel={() => setDraft(null)}
        onOk={apply}
        okText="Apply"
        width={
          showingSource
            ? "min(1180px, calc(100vw - 32px))"
            : "min(560px, calc(100vw - 32px))"
        }
        destroyOnHidden
        classNames={{
          body: showingSource
            ? "search-settings-dialog__body search-settings-dialog__body--source"
            : "search-settings-dialog__body",
        }}
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
                        showSimilarityDetails: false,
                        showSmallerWeights: false,
                        similarity: trait
                          ? DEFAULT_TRAIT_SIMILARITY_PREFERENCES
                          : DEFAULT_SIMILARITY_PREFERENCES,
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
          <>
            {showingSource && !wide && (
              <Segmented
                aria-label="Search settings view"
                block
                value={sourceView}
                options={[
                  { label: "Search settings", value: "sections" },
                  { label: "Source document", value: "document" },
                ]}
                onChange={setSourceView}
              />
            )}
            <div className={showingSource ? "search-settings__workspace" : undefined}>
              <div
                className="search-settings__controls"
                hidden={showingSource && !wide && sourceView !== "sections"}
              >
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
                  {profile.profiles.filter((item) => item.stored_vectors).length >
                    1 && (
                    <Form.Item
                      label={
                        <SettingLabel
                          label="Embedding model"
                          help="Changes which embedding model is used for search and document ranking."
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
                          help="Search combines word matches with meaning matches. Score-based considers how strongly each record matches. Rank-based considers how high each record appears in those two lists."
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
                        label={
                          <SettingLabel label="Comparison method" help={methodHelp} />
                        }
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
                              help="Sections are ordered by match strength for each result. 0 uses only the strongest section; 1 gives all selected sections equal weight. Values in between give each additional section less weight."
                            />
                          }
                          validateStatus={invalidDecay ? "error" : undefined}
                          help={
                            invalidDecay
                              ? "Enter a number from 0 through 1."
                              : undefined
                          }
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
                                    decay:
                                      typeof decay === "number" ? decay : Number.NaN,
                                  },
                                });
                              }}
                            />
                          </div>
                          <div className="search-settings__scale">
                            <span>Strongest section</span>
                            <span>All sections equally</span>
                          </div>
                          {!invalidDecay && (
                            <Typography.Text type="secondary">
                              {selectedCount === 1
                                ? "One section selected; decay has no effect."
                                : `Contributions: 1, ${Number(draft.similarity.decay.toFixed(4))}, ${Number((draft.similarity.decay ** 2).toFixed(4))}…`}
                            </Typography.Text>
                          )}
                        </Form.Item>
                      )}
                      {draft.similarity.method !== "baseline" && (
                        <Collapse
                          activeKey={partsOpen ? ["parts"] : []}
                          onChange={(keys) => {
                            setPartsOpen(keys.includes("parts"));
                            if (keys.includes("parts")) setSourceVisited(true);
                          }}
                          items={[
                            {
                              key: "parts",
                              label: (
                                <SettingLabel
                                  label="Sections to compare"
                                  help={partsHelp}
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
                                  {(inventory.error || inventoryRetired) && (
                                    <Alert
                                      type="error"
                                      title="Unable to load comparison sections"
                                      description={
                                        inventory.error?.message ??
                                        "The reference collection changed."
                                      }
                                      action={
                                        <Button
                                          onClick={() => {
                                            if (inventoryRetired) {
                                              reloadSections();
                                            } else void inventory.refetch();
                                          }}
                                        >
                                          {inventoryRetired
                                            ? "Reload sections"
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
                                            generation:
                                              inventory.data.artifact_generation,
                                          })
                                        }
                                      >
                                        All sections
                                      </Checkbox>
                                      <PartChoices
                                        parts={allParts}
                                        seed={seed!}
                                        onReference={(part, opener) => {
                                          setSourceTarget((current) => ({
                                            part,
                                            opener,
                                            visit: (current?.visit ?? 0) + 1,
                                          }));
                                          setTargetIdentity(sourceIdentity);
                                          setSourceView("document");
                                        }}
                                        selected={draft.parts}
                                        onChange={(parts) =>
                                          setDraft({
                                            ...draft,
                                            parts,
                                            generation:
                                              inventory.data.artifact_generation,
                                          })
                                        }
                                      />
                                      {(invalidParts || staleParts) && (
                                        <Typography.Paragraph
                                          type="danger"
                                          role="alert"
                                        >
                                          {staleParts
                                            ? "Some selected sections are unavailable. Choose the sections again."
                                            : "Select at least one section."}
                                        </Typography.Paragraph>
                                      )}
                                      {allParts.some((part) => part.label_only) && (
                                        <Typography.Paragraph type="secondary">
                                          This trait has no definition text; comparison
                                          uses its label.
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
                                              ? "Only sections with matching purposes can contribute, including across record kinds. For example, descriptions compare with descriptions. If a result lacks that type of section, it contributes no match."
                                              : "Any selected section can match any content in a result, even when the sections serve different purposes."
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
                                            label: "Corresponding sections",
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
                  {seed && (
                    <Form.Item
                      label={
                        <SettingLabel
                          label="Show similarity details"
                          help="Replaces the list of matching sections with scores, weights, and source and result excerpts."
                        />
                      }
                      htmlFor={`${id}-similarity-details`}
                    >
                      <Switch
                        id={`${id}-similarity-details`}
                        aria-label="Show similarity details"
                        checked={draft.showSimilarityDetails}
                        onChange={(showSimilarityDetails) =>
                          setDraft({ ...draft, showSimilarityDetails })
                        }
                      />
                    </Form.Item>
                  )}
                  {seed && draft.showSimilarityDetails && (
                    <Form.Item
                      label={
                        <SettingLabel
                          label="Show smaller weights"
                          help="Includes rows whose weight rounds to 0.0%. All sections still contribute to ranking."
                        />
                      }
                      htmlFor={`${id}-smaller-weights`}
                    >
                      <Switch
                        id={`${id}-smaller-weights`}
                        aria-label="Show smaller weights"
                        checked={draft.showSmallerWeights}
                        onChange={(showSmallerWeights) =>
                          setDraft({ ...draft, showSmallerWeights })
                        }
                      />
                    </Form.Item>
                  )}
                </Form>
              </div>
              {seed && (
                <div
                  className="search-settings__document"
                  hidden={!showingSource || (!wide && sourceView !== "document")}
                >
                  <p className="search-settings__comparison-key">
                    Only highlighted sections are used for comparison.
                  </p>
                  {sourceVisited &&
                    sourceGeneration !== null &&
                    !isArtifactGenerationChanged(inventory.error) && (
                      <SimilaritySourceDocument
                        key={sourceIdentity}
                        seed={seed}
                        generation={sourceGeneration}
                        target={currentTarget}
                        selection={
                          inventory.data &&
                          inventory.data.artifact_generation === sourceGeneration &&
                          !inventoryRetired &&
                          !inventory.isFetching &&
                          inventory.error === null
                            ? { inventory: inventory.data, selected: draft.parts }
                            : null
                        }
                        active={showingSource && (wide || sourceView === "document")}
                        onReloadSections={reloadSections}
                      />
                    )}
                  {showingSource &&
                    !wide &&
                    sourceView === "document" &&
                    inventoryRetired && (
                      <Alert
                        type="error"
                        title="Reference collection changed"
                        description="Reload the sections to read the current source."
                        action={
                          <Button onClick={reloadSections}>Reload sections</Button>
                        }
                      />
                    )}
                  {showingSource && inventory.isPending && (
                    <Spin delay={500} description="Loading document…" />
                  )}
                </div>
              )}
            </div>
          </>
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
  onReference,
  seed,
}: {
  seed: SimilaritySeed;
  onReference: (part: SimilarityPartView, opener: HTMLElement) => void;
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
        const singleSection = entries.length === 1;
        return (
          <div key={key} className="search-settings__part-group">
            {!singleSection && (
              <div className="search-settings__part-choice">
                <Checkbox
                  aria-label={`All sections in ${owner}`}
                  checked={selectors.length > 0 && count === selectors.length}
                  indeterminate={count > 0 && count < selectors.length}
                  disabled={!selectors.length}
                  onChange={(event) => toggle(selectors, event.target.checked)}
                />
                <strong>{owner}</strong>
              </div>
            )}
            <div className={singleSection ? undefined : "search-settings__part-fields"}>
              {entries.map((part) => (
                <div key={part.selector} className="search-settings__part-choice">
                  <Checkbox
                    aria-label={singleSection ? `${owner}: ${part.label}` : part.label}
                    checked={chosen.has(part.selector)}
                    disabled={!part.has_vectors}
                    onChange={(event) => toggle([part.selector], event.target.checked)}
                  />
                  <span>
                    <Typography.Link
                      strong={singleSection}
                      href={destinationHref(
                        seed.kind === "trait"
                          ? traitDestination({ traitKey: seed.trait_key })
                          : recordDestination({
                              recordKey: seed.record_key,
                              selection: part.navigation ?? undefined,
                            }),
                      )}
                      onClick={(event) => {
                        if (!isPrimaryActivation(event)) return;
                        event.preventDefault();
                        onReference(part, event.currentTarget);
                      }}
                    >
                      {singleSection ? owner : part.label}
                    </Typography.Link>
                    {singleSection && (
                      <>
                        {" "}
                        <Typography.Text type="secondary">{part.label}</Typography.Text>
                      </>
                    )}
                    {!part.has_vectors ? " (no stored vector)" : ""}
                  </span>
                </div>
              ))}
            </div>
          </div>
        );
      })}
    </div>
  );
}
