import { useEffect, useId, useRef, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import {
  Alert,
  App,
  Button,
  Checkbox,
  Collapse,
  Form,
  Grid,
  Select,
  Segmented,
  Space,
  Switch,
  Tooltip,
  Typography,
} from "antd";
import { ArrowLeft, SlidersHorizontal } from "lucide-react";
import type {
  SearchRanking,
  SearchContentScope,
  SimilaritySourceSections,
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
  DEFAULT_SEARCH_PREFERENCES,
  DEFAULT_SOURCE_SECTIONS,
  DEFAULT_SIMILARITY_PREFERENCES,
  DEFAULT_TRAIT_SIMILARITY_PREFERENCES,
  type SimilarityPreferences,
  type MatchDisplay,
} from "./searchPreferences";
import {
  selectedSimilarityParts,
  similarityFromPreferences,
  similarityLabel,
  similarityPreferences,
} from "./similarityState";
import { isArtifactGenerationChanged } from "../queries/immutableReferenceQueries";
import {
  SimilaritySourceDocument,
  type SimilaritySourceTarget,
} from "./SimilaritySourceDocument";
import { assertSimilarityInventoryIdentity } from "./searchIdentity";
import { SearchSettingLabel } from "./SearchSettingLabel";
import { SimilarityComparisonSettings } from "./SimilarityComparisonSettings";
import { AtlasModal as Modal } from "../ui/overlays/AtlasModal";
import { LoadingStatus } from "../ui/LoadingStatus";
import { ErrorNotice } from "../feedback/ErrorNotice";
import { SimilaritySectionChoices } from "./SimilaritySectionChoices";

type SettingsDraft = {
  seedIdentity: string;
  model: string | null;
  ranking: SearchRanking;
  searchContent: SearchContentScope;
  sourceSections: SimilaritySourceSections;
  matchDisplay: MatchDisplay;
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
  const appliedSourceSections =
    search.sourceSections ?? (appliedParts ? "custom" : "all");
  const searchContent = search.searchContent ?? "all";
  const [draft, setDraft] = useState<SettingsDraft | null>(null);
  const [partsOpen, setPartsOpen] = useState(false);
  const chooseSectionsRef = useRef<HTMLButtonElement>(null);
  const editorHeadingRef = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (partsOpen) editorHeadingRef.current?.focus();
    else chooseSectionsRef.current?.focus();
  }, [partsOpen]);
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
      assertSimilarityInventoryIdentity(
        value,
        seed!,
        draft!.model!,
        reloadInventory ? null : (draft!.generation ?? generation),
      );
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
  const selectedParts =
    draft?.sourceSections === "core"
      ? (inventory.data?.core.selectors ?? [])
      : draft?.sourceSections === "all"
        ? available.map((part) => part.selector)
        : (draft?.parts ?? []);
  const selectedCount = selectedParts.length;
  const allSelected =
    available.length > 0 &&
    selectedParts.length === available.length &&
    available.every((part) => selectedParts.includes(part.selector));
  const missingCore =
    draft?.sourceSections === "core" &&
    inventory.data !== undefined &&
    selectedParts.some(
      (selector) => !available.some((part) => part.selector === selector),
    );
  const identitySelected = allParts.some(
    (part) => part.kind === "identity" && selectedParts.includes(part.selector),
  );
  const usesParts = draft !== null && draft.similarity.method !== "baseline";
  const missingAllIdentity =
    usesParts &&
    draft.sourceSections === "all" &&
    allParts.some((part) => part.kind === "identity" && !part.has_vectors);
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
  const invalidParts =
    usesParts && draft?.sourceSections === "custom" && draft.parts?.length === 0;
  const staleParts =
    usesParts &&
    draft?.sourceSections === "custom" &&
    draft?.parts !== null &&
    draft?.parts !== undefined &&
    inventory.data !== undefined &&
    draft.parts.some(
      (selector) => !available.some((part) => part.selector === selector),
    );
  const uncheckedParts =
    seed !== null &&
    draft !== null &&
    draft.similarity.method !== "baseline" &&
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
      ? appliedSourceSections === "core"
        ? "Core sections"
        : appliedParts
          ? `${appliedParts.length} selections`
          : "All sections"
      : null,
  ]
    .filter(Boolean)
    .join("; ");
  function openSettings() {
    setSourceView("sections");
    setSourceTarget(null);
    setSourceSnapshot(null);
    setPartsOpen(false);
    setSourceVisited(false);
    setReloadInventory(0);
    setSourceRetired(false);
    setDraft({
      seedIdentity,
      model: profile.id,
      ranking,
      searchContent,
      sourceSections: appliedSourceSections,
      matchDisplay: profile.preferences.matchDisplay,
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
      missingCore ||
      missingAllIdentity ||
      staleParts ||
      uncheckedParts ||
      invalidDecay
    )
      return;
    const nextSimilarity = seed
      ? similarityFromPreferences(
          draft.similarity,
          trait,
          draft.sourceSections === "custom" ? draft.parts : null,
        )
      : search.similarity;
    const presentationOnly =
      (draft.matchDisplay !== profile.preferences.matchDisplay ||
        draft.showSmallerWeights !== profile.preferences.showSmallerWeights) &&
      !search.filterError &&
      draft.model === profile.id &&
      draft.ranking === ranking &&
      draft.searchContent === searchContent &&
      draft.sourceSections === appliedSourceSections &&
      draft.similarity.method === appliedPreferences.method &&
      draft.similarity.decay === appliedPreferences.decay &&
      draft.similarity.candidateParts === appliedPreferences.candidateParts &&
      JSON.stringify(draft.parts) === JSON.stringify(appliedParts) &&
      draft.generation === search.similarityGeneration;
    const nextPreferences = {
      ...profile.preferences,
      model: draft.model,
      searchContent: draft.searchContent,
      sourceSectionsByFamily:
        inventory.data &&
        draft.sourceSections !== "custom" &&
        draft.similarity.method !== "baseline"
          ? {
              ...profile.preferences.sourceSectionsByFamily,
              [inventory.data.core.family]: draft.sourceSections,
            }
          : profile.preferences.sourceSectionsByFamily,
      matchDisplay: draft.matchDisplay,
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
          searchContent: draft.searchContent,
          ...(seed
            ? {
                similarity: nextSimilarity,
                sourceSections:
                  nextSimilarity?.method === "baseline"
                    ? undefined
                    : draft.sourceSections,
                similarityGeneration:
                  nextSimilarity?.method !== "baseline" &&
                  draft.sourceSections === "custom"
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
    draft?.sourceSections === "core"
      ? "Core sections"
      : draft?.sourceSections === "all"
        ? "All sections"
        : identitySelected
          ? `${selectedCount} selections`
          : `${selectedCount} selected ${selectedCount === 1 ? "section" : "sections"}`;
  const sourcePossessive = seedTitle
    ? `${seedTitle}’s`
    : trait
      ? "the trait’s"
      : "the record’s";
  const sourceName = seedTitle || (trait ? "this trait" : "this record");
  const searchContentHelp =
    draft?.searchContent === "core"
      ? "Limits content searches to a core set of fields, like name and description, while omitting details like a creature's spells."
      : "Searches all available content, including details like a creature's spells.";
  const sourceSectionsHelp =
    draft?.sourceSections === "core"
      ? trait
        ? `Uses ${sourcePossessive} definition, or its name when no definition is available.`
        : `Uses ${sourcePossessive} name and identity plus the core sections chosen for this record kind.`
      : draft?.sourceSections === "custom"
        ? `Uses only the selected content from ${sourceName}.`
        : trait
          ? `Uses all of ${sourcePossessive} available document sections.`
          : `Uses ${sourcePossessive} name and identity plus all available document sections.`;
  const partsHelp = `Choose which sections of this ${trait ? "trait" : "record"} to use when finding similar records.`;
  const methodHelp =
    draft?.similarity.method === "passages"
      ? "Looks for passages similar to each selected section. Each selected section can match a different passage in the results."
      : draft?.similarity.method === "content-average"
        ? "Averages the selected sections into an overall comparison. Specific details can become less prominent when several different sections are combined."
        : trait
          ? "Uses the trait’s definition as the starting point, or its name when no definition is available."
          : `Uses ${sourcePossessive} name and identifying information. Its description and abilities aren’t used.`;
  const sectionStatus = draft && (
    <>
      {" "}
      {draft.sourceSections === "core" &&
        inventory.data?.core.representation === "identity_only" && (
          <Typography.Text type="secondary">
            <SearchSettingLabel
              label="Name and identity only"
              help={
                inventory.data.core.fallback_reason === "unmodelled_kind"
                  ? "Core sections aren’t defined for this record kind."
                  : "This record has no available core sections."
              }
            />
          </Typography.Text>
        )}
      {missingCore && (
        <Typography.Paragraph type="danger" role="alert">
          Some core content is unavailable for this model. Choose another model or
          update the collection.
        </Typography.Paragraph>
      )}
      {missingAllIdentity && (
        <Typography.Paragraph type="danger" role="alert">
          Name and identity is unavailable for this model. Choose another model or
          update the collection.
        </Typography.Paragraph>
      )}
      {inventory.isFetching && <LoadingStatus label="Loading comparison sections…" />}
      {(inventory.error || inventoryRetired) && (
        <ErrorNotice
          error={inventory.error}
          type="error"
          title="Unable to load comparison sections"
          description={
            inventoryRetired
              ? "The reference collection changed. Reload sections to continue."
              : undefined
          }
          action={
            <Button
              onClick={() => {
                if (inventoryRetired) {
                  reloadSections();
                } else void inventory.refetch();
              }}
            >
              {inventoryRetired ? "Reload sections" : "Retry"}
            </Button>
          }
        />
      )}
    </>
  );
  return (
    <>
      <Tooltip title={`Search settings: ${summary}`}>
        <Button
          icon={<SlidersHorizontal size={18} aria-hidden="true" />}
          aria-label="Search settings"
          aria-describedby={`${id}-summary`}
          disabled={disabled}
          onClick={openSettings}
        />
      </Tooltip>
      <span id={`${id}-summary`} className="sr-only">
        {summary}
      </span>
      <Modal
        title={showingSource ? "Choose sections" : "Search settings"}
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
        styles={
          showingSource
            ? { body: { overflowY: "hidden", overflowX: "hidden" } }
            : undefined
        }
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
                        ranking: DEFAULT_SEARCH_PREFERENCES.ranking,
                        searchContent: DEFAULT_SEARCH_PREFERENCES.searchContent,
                        sourceSections: DEFAULT_SOURCE_SECTIONS,
                        matchDisplay: DEFAULT_SEARCH_PREFERENCES.matchDisplay,
                        showSmallerWeights:
                          DEFAULT_SEARCH_PREFERENCES.showSmallerWeights,
                        similarity: trait
                          ? DEFAULT_TRAIT_SIMILARITY_PREFERENCES
                          : DEFAULT_SIMILARITY_PREFERENCES,
                        parts: null,
                        generation: undefined,
                      }
                    : null,
                );
                setPartsOpen(false);
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
            missingCore ||
            missingAllIdentity ||
            staleParts ||
            uncheckedParts ||
            invalidDecay,
        }}
      >
        {draft && (
          <>
            {showingSource && (
              <div
                className="search-settings__editor-header"
                ref={editorHeadingRef}
                tabIndex={-1}
              >
                <Button
                  icon={<ArrowLeft size={16} aria-hidden="true" />}
                  onClick={() => setPartsOpen(false)}
                  aria-label="Back to search settings"
                >
                  Search settings
                </Button>
                <SearchSettingLabel
                  label={`Sections from ${sourceName}`}
                  help={partsHelp}
                />
              </div>
            )}
            {showingSource && !wide && (
              <Segmented
                aria-label="Search settings view"
                block
                styles={{
                  label: {
                    paddingBlock: "var(--space-control)",
                    lineHeight: 1.4,
                    whiteSpace: "normal",
                    overflow: "visible",
                    textOverflow: "clip",
                  },
                }}
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
                  {showingSource ? (
                    <>
                      <div className="search-settings__section-summary">
                        <Typography.Text strong>{selectionSummary}</Typography.Text>
                      </div>
                      {sectionStatus}
                      {inventory.data && (
                        <>
                          <Checkbox
                            checked={allSelected}
                            indeterminate={!allSelected && selectedParts.length > 0}
                            onChange={(event) =>
                              setDraft({
                                ...draft,
                                sourceSections: event.target.checked ? "all" : "custom",
                                parts: event.target.checked ? null : [],
                                generation: inventory.data.artifact_generation,
                              })
                            }
                          >
                            Select all
                          </Checkbox>
                          <SimilaritySectionChoices
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
                            selected={selectedParts}
                            onChange={(parts) =>
                              setDraft({
                                ...draft,
                                parts,
                                sourceSections: "custom",
                                generation: inventory.data.artifact_generation,
                              })
                            }
                          />
                          {(invalidParts || staleParts) && (
                            <Typography.Paragraph type="danger" role="alert">
                              {staleParts
                                ? "Some selected sections are unavailable. Choose the sections again."
                                : "Select at least one section."}
                            </Typography.Paragraph>
                          )}
                          {allParts.some(
                            (part) => part.kind === "section" && part.label_only,
                          ) && (
                            <Typography.Paragraph type="secondary">
                              This trait has no definition text; comparison uses its
                              label.
                            </Typography.Paragraph>
                          )}
                        </>
                      )}
                    </>
                  ) : (
                    <>
                      {profile.error && draft.model === profile.id && (
                        <Alert type="error" title={profile.error} />
                      )}
                      {!validModel &&
                        !(profile.error && draft.model === profile.id) && (
                          <Alert
                            type="error"
                            title={
                              modelOptions.length
                                ? "Choose an available embedding model, or use Reset."
                                : "No search model is stored in this collection. Check setup to continue."
                            }
                          />
                        )}
                      {seed &&
                        (usesParts ? (
                          <Form.Item
                            label={
                              <SearchSettingLabel
                                label="Source sections"
                                help={sourceSectionsHelp}
                              />
                            }
                            htmlFor={`${id}-sections`}
                            extra={`Content from ${sourceName} used for comparison.`}
                          >
                            <div className="search-settings__source-choice">
                              <Select
                                className="search-settings__source-preset"
                                id={`${id}-sections`}
                                aria-label="Source sections"
                                value={draft.sourceSections}
                                options={[
                                  { value: "all", label: "All sections" },
                                  { value: "core", label: "Core sections" },
                                  { value: "custom", label: "Custom" },
                                ]}
                                onChange={(
                                  sourceSections: SimilaritySourceSections,
                                ) => {
                                  setDraft({
                                    ...draft,
                                    sourceSections,
                                    parts:
                                      sourceSections === "custom"
                                        ? selectedParts
                                        : null,
                                    generation:
                                      sourceSections === "custom"
                                        ? inventory.data?.artifact_generation
                                        : undefined,
                                  });
                                  if (sourceSections === "custom") {
                                    setPartsOpen(true);
                                    setSourceVisited(true);
                                    setSourceView("sections");
                                  }
                                }}
                              />
                              <Button
                                className="search-settings__choose"
                                ref={chooseSectionsRef}
                                onClick={() => {
                                  setPartsOpen(true);
                                  setSourceVisited(true);
                                  setSourceView("sections");
                                }}
                              >
                                Choose sections
                              </Button>
                            </div>
                            {sectionStatus}
                          </Form.Item>
                        ) : (
                          <Form.Item label="Source sections">
                            <Typography.Text>
                              {trait ? "Trait definition" : "Name and identity"}
                            </Typography.Text>
                          </Form.Item>
                        ))}
                      <Form.Item
                        label={
                          <SearchSettingLabel
                            label="Search content"
                            help={searchContentHelp}
                          />
                        }
                        htmlFor={`${id}-content`}
                      >
                        <Select
                          id={`${id}-content`}
                          aria-label="Search content"
                          value={draft.searchContent}
                          options={[
                            { value: "all", label: "All sections" },
                            { value: "core", label: "Core sections" },
                          ]}
                          onChange={(searchContent) =>
                            setDraft({ ...draft, searchContent })
                          }
                        />
                      </Form.Item>
                      <Collapse
                        defaultActiveKey={!validModel ? ["advanced"] : []}
                        items={[
                          {
                            key: "advanced",
                            label: "Advanced settings",
                            extra: (
                              <Typography.Text type="secondary">
                                {seed
                                  ? similarityLabel(
                                      similarityFromPreferences(
                                        draft.similarity,
                                        trait,
                                      ),
                                      trait,
                                    )
                                  : draft.ranking === "score"
                                    ? "Score-based"
                                    : "Rank-based"}
                              </Typography.Text>
                            ),
                            children: (
                              <>
                                {profile.profiles.filter((item) => item.stored_vectors)
                                  .length > 1 && (
                                  <Form.Item
                                    label={
                                      <SearchSettingLabel
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
                                      onChange={(model) =>
                                        setDraft({ ...draft, model })
                                      }
                                    />
                                  </Form.Item>
                                )}
                                {!seed ? (
                                  <Form.Item
                                    label={
                                      <SearchSettingLabel
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
                                      onChange={(next) =>
                                        setDraft({ ...draft, ranking: next })
                                      }
                                    />
                                  </Form.Item>
                                ) : (
                                  <SimilarityComparisonSettings
                                    value={draft.similarity}
                                    onChange={(similarity) =>
                                      setDraft({ ...draft, similarity })
                                    }
                                    trait={trait}
                                    methodHelp={methodHelp}
                                    selectedCount={selectedCount}
                                    identitySelected={identitySelected}
                                    invalidDecay={invalidDecay}
                                  />
                                )}
                              </>
                            ),
                          },
                          {
                            key: "details",
                            label: "Explanation details",
                            children: (
                              <>
                                <Form.Item
                                  label={
                                    <SearchSettingLabel
                                      label="Match display"
                                      help={
                                        draft.matchDisplay === "hidden"
                                          ? "Hides match information. Results and ranking stay the same."
                                          : draft.matchDisplay === "detailed"
                                            ? "Shows matching sections, plus scores, weights and excerpts for Similar results."
                                            : "Lists matching sections and excerpts."
                                      }
                                    />
                                  }
                                  htmlFor={`${id}-match-display`}
                                >
                                  <Select
                                    id={`${id}-match-display`}
                                    aria-label="Match display"
                                    value={draft.matchDisplay}
                                    options={[
                                      { value: "hidden", label: "Hidden" },
                                      { value: "sections", label: "Matching sections" },
                                      { value: "detailed", label: "Detailed" },
                                    ]}
                                    onChange={(matchDisplay: MatchDisplay) =>
                                      setDraft({ ...draft, matchDisplay })
                                    }
                                  />
                                </Form.Item>
                                {seed && draft.matchDisplay === "detailed" && (
                                  <Form.Item
                                    layout="horizontal"
                                    labelAlign="left"
                                    colon={false}
                                    labelCol={{ flex: "auto" }}
                                    wrapperCol={{ flex: "none" }}
                                    label={
                                      <SearchSettingLabel
                                        label="Show smaller weights"
                                        help="Includes rows whose weight rounds to 0.0%. All selections still contribute to ranking."
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
                              </>
                            ),
                          },
                        ]}
                      />
                    </>
                  )}
                </Form>
              </div>
              {seed && (
                <div
                  className="search-settings__document"
                  hidden={!showingSource || (!wide && sourceView !== "document")}
                >
                  <p className="search-settings__comparison-key">
                    {identitySelected
                      ? allParts.some(
                          (part) =>
                            part.kind === "section" &&
                            selectedParts.includes(part.selector),
                        )
                        ? "Name and identity and the highlighted sections are used for comparison."
                        : "Only name and identity are used for comparison."
                      : "Only highlighted sections are used for comparison."}
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
                            ? {
                                inventory: inventory.data,
                                selected:
                                  draft.sourceSections === "all" ? null : selectedParts,
                              }
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
                    <LoadingStatus label="Loading document…" />
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
