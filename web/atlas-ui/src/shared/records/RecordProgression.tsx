import { Button } from "antd";
import { ChevronDown, ChevronRight, Compass } from "lucide-react";
import { useInfiniteQuery, useQuery, useQueryClient } from "@tanstack/react-query";
import { AtlasApiError, getProgression } from "../../api/atlasApi";
import { AtlasLink } from "../../app/AtlasLink";
import { recordDestination } from "../../app/destinations";
import {
  destinationHref,
  isPrimaryActivation,
  navigateDestination,
} from "../../app/navigation";
import { useAtlasRouter } from "../../app/router";
import type {
  ProgressionPageView,
  ProgressionRequest,
  ProgressionRequirementView,
  RecordDetailView,
  RecordNavigationView,
} from "../../generated/atlas";
import { ErrorNotice } from "../feedback/ErrorNotice";
import {
  isArtifactGenerationChanged,
  referenceQueryMeta,
} from "../queries/referenceQueries";
import { LoadingStatus } from "../ui/LoadingStatus";
import { InlineContext, InlineIconLink } from "../ui/actions/InlineAction";
import { PaneIconButton } from "../ui/actions/PaneAction";
import type { RecordReferenceHandler } from "./PreparedContent";
import { useRecordReadingVisit } from "./recordExploreLocation";

interface ProgressionContext {
  generation: string;
  ancestors: readonly string[];
  path: readonly string[];
  shownPrerequisiteKeys: ReadonlySet<string>;
  expandedBranches: readonly string[];
  setBranchExpanded: (key: string, open: boolean) => void;
  onReference: RecordReferenceHandler;
  onReload?: () => void;
  reloadLabel?: string;
}

type ProgressionEntry = ProgressionPageView["entries"][number];
type ProgressionWitness = ProgressionEntry["witnesses"][number];
const NO_EXPANDED_BRANCHES: readonly string[] = [];
const NO_SHOWN_PREREQUISITES: ReadonlySet<string> = new Set();

export function RecordProgression({
  detail,
  onReference,
  onReload,
  reloadLabel,
}: {
  detail: RecordDetailView;
  onReference: RecordReferenceHandler;
  onReload?: () => void;
  reloadLabel?: string;
}) {
  const generation = detail.artifact_generation;
  const visit = useRecordReadingVisit();
  const client = useQueryClient();
  const historyKey = ["progression-expanded", generation, detail.selected, visit];
  const { data: expandedBranches } = useQuery({
    queryKey: historyKey,
    // Keep disclosure state with this visit while the reference reads are cached.
    meta: referenceQueryMeta,
    queryFn: () => NO_EXPANDED_BRANCHES,
    initialData: NO_EXPANDED_BRANCHES,
    enabled: false,
  });
  if (!detail.progression) {
    return <p>No linked progression is available for this record.</p>;
  }
  function setBranchExpanded(key: string, open: boolean) {
    client.setQueryData<readonly string[]>(
      historyKey,
      (current = NO_EXPANDED_BRANCHES) => {
        if (current.includes(key) === open) return current;
        return open ? [...current, key] : current.filter((item) => item !== key);
      },
    );
  }

  return (
    <ProgressionOverview
      selection={detail.selected}
      generation={generation}
      ancestors={[]}
      path={[]}
      shownPrerequisiteKeys={NO_SHOWN_PREREQUISITES}
      expandedBranches={expandedBranches}
      setBranchExpanded={setBranchExpanded}
      onReference={onReference}
      onReload={onReload}
      reloadLabel={reloadLabel}
    />
  );
}

async function loadProgression(request: ProgressionRequest, signal: AbortSignal) {
  const result = await getProgression(request, signal);
  if (result.artifact_generation !== request.artifact_generation) {
    throw new AtlasApiError(409, "Reference data changed", {
      code: "artifact_generation_changed",
      message: "Reference data changed.",
    });
  }
  if (result.selection.record_key !== request.selection.record_key) {
    throw new AtlasApiError(200, "Invalid progression response");
  }
  return result;
}

function ProgressionOverview({
  selection,
  requirements,
  witnesses,
  ...context
}: ProgressionContext & {
  selection: RecordNavigationView;
  requirements?: ProgressionRequirementView[];
  witnesses?: ProgressionWitness[];
}) {
  const { generation, ancestors } = context;
  const Heading = ancestors.length ? "h4" : "h3";
  const query = useQuery({
    meta: referenceQueryMeta,
    queryKey: ["progression", generation, selection, "overview"],
    queryFn: ({ signal }) =>
      loadProgression({ selection, artifact_generation: generation }, signal),
    staleTime: Infinity,
    retry: false,
  });
  const branch = {
    ...context,
    ancestors: [...ancestors, selection.record_key],
    shownPrerequisiteKeys: NO_SHOWN_PREREQUISITES,
  };
  const generationChanged = isArtifactGenerationChanged(query.error);
  const data = generationChanged ? undefined : query.data;
  const shownRequirements = generationChanged
    ? undefined
    : (requirements ?? data?.prerequisites);
  const flatRequirements = flatPrerequisites(shownRequirements);
  const additionalRequirements = alsoRequires(
    flatRequirements,
    witnesses,
    context.shownPrerequisiteKeys,
  );
  const visibleRequirements = additionalRequirements ?? shownRequirements;
  const requiredBy = data?.required_by;
  const access = data?.available_through;
  const pools = data?.ancestry_feat_pools ?? [];
  const hasConnections = Boolean(
    visibleRequirements?.length ||
    requiredBy?.entries.length ||
    access?.entries.length ||
    pools.length,
  );
  // Carry only exact, unconditional facts along forward progression branches.
  const shownPrerequisiteKeys = new Set(context.shownPrerequisiteKeys);
  shownPrerequisiteKeys.add(`record:${selection.record_key}`);
  for (const requirement of flatRequirements ?? []) {
    const key = prerequisiteKey(requirement);
    if (key !== undefined) shownPrerequisiteKeys.add(key);
  }

  return (
    <div className="record-progression">
      {visibleRequirements && visibleRequirements.length > 0 && (
        <section
          className={
            additionalRequirements ? "record-progression__additional" : undefined
          }
        >
          <Heading>
            {additionalRequirements ? "Also requires" : "Prerequisites"}
          </Heading>
          <RequirementList
            entries={visibleRequirements}
            context={branch}
            witnesses={additionalRequirements ? undefined : witnesses}
          />
        </section>
      )}
      {query.isPending && <LoadingStatus label="Loading progression…" />}
      {query.error && (
        <ProgressionError
          error={query.error}
          retry={() => void query.refetch()}
          context={context}
        />
      )}
      {requiredBy && requiredBy.entries.length > 0 && (
        <section>
          <Heading>{ancestors.length ? "Leads to" : "Required by"}</Heading>
          <ProgressionResults
            selection={selection}
            context={{ ...branch, shownPrerequisiteKeys }}
            direction="required_by"
            initialPage={requiredBy}
          />
        </section>
      )}
      {pools.length > 0 && (
        <section>
          <Heading>Ancestry feat access</Heading>
          <ul className="record-progression__pools">
            {pools.map((pool) => (
              <li key={pool.trait_key}>
                <AncestryFeatPool
                  label={pool.label}
                  traitKey={pool.trait_key}
                  selection={selection}
                  context={branch}
                />
              </li>
            ))}
          </ul>
        </section>
      )}
      {access && access.entries.length > 0 && (
        <section>
          <Heading>Available through</Heading>
          <ProgressionResults
            selection={selection}
            context={branch}
            direction="available_through"
            initialPage={access}
          />
        </section>
      )}
      {data && !hasConnections && (
        <p className="record-progression__empty">
          {additionalRequirements
            ? "No further progression."
            : "No linked progression is available for this record."}
        </p>
      )}
    </div>
  );
}

function ProgressionError({
  error,
  retry,
  context: { onReload, reloadLabel = "Reload record" },
}: {
  error: Error;
  retry: () => void;
  context: ProgressionContext;
}) {
  const reload =
    isArtifactGenerationChanged(error) ||
    (error instanceof AtlasApiError &&
      error.appError?.recoverable_action === "reload_record");
  return (
    <ErrorNotice
      error={error}
      title="Unable to load progression"
      action={
        <Button
          size="small"
          onClick={reload ? onReload : retry}
          disabled={reload && !onReload}
        >
          {reload ? reloadLabel : "Retry"}
        </Button>
      }
    />
  );
}

function AncestryFeatPool({
  label,
  traitKey,
  selection,
  context,
}: {
  label: string;
  traitKey: string;
  selection: RecordNavigationView;
  context: ProgressionContext;
}) {
  const poolContext = { ...context, path: [...context.path, "pool", traitKey] };
  const key = JSON.stringify(poolContext.path);
  const expanded = context.expandedBranches.includes(key);
  return (
    <details
      open={expanded}
      onToggle={(event) => context.setBranchExpanded(key, event.currentTarget.open)}
    >
      <summary>{label}</summary>
      {expanded && (
        <ProgressionResults
          selection={selection}
          context={poolContext}
          direction="pool"
          traitKey={traitKey}
        />
      )}
    </details>
  );
}

function ProgressionResults({
  selection,
  context,
  direction,
  traitKey,
  initialPage,
}: {
  selection: RecordNavigationView;
  context: ProgressionContext;
  direction: "required_by" | "pool" | "available_through";
  traitKey?: string;
  initialPage?: ProgressionPageView;
}) {
  const { generation } = context;
  const query = useInfiniteQuery({
    meta: referenceQueryMeta,
    queryKey: ["progression", generation, selection, direction, traitKey],
    queryFn: async ({ signal, pageParam }) => {
      const result = await loadProgression(
        {
          selection,
          artifact_generation: generation,
          direction,
          trait_key: traitKey,
          offset: pageParam,
        },
        signal,
      );
      return result[direction];
    },
    initialPageParam: 0,
    getNextPageParam: (page) => page.next_offset ?? undefined,
    initialData: initialPage ? { pages: [initialPage], pageParams: [0] } : undefined,
    staleTime: Infinity,
    retry: false,
  });
  const pages = isArtifactGenerationChanged(query.error)
    ? undefined
    : query.data?.pages;
  const entries = pages?.flatMap((page) => page.entries) ?? [];

  return (
    <>
      {entries.length > 0 && (
        <ul className="record-progression__relations">
          {entries.map((entry) => (
            <li key={entry.navigation.record_key}>
              <ProgressionReference
                context={{
                  ...context,
                  path: [...context.path, direction, entry.navigation.record_key],
                }}
                label={entry.title}
                navigation={entry.navigation}
                requirements={entry.requirements}
                witnesses={entry.witnesses}
                level={entry.level}
              />
            </li>
          ))}
        </ul>
      )}
      {pages && entries.length === 0 && <p>No linked feats in this pool.</p>}
      {query.isPending && <LoadingStatus label="Loading progression…" />}
      {query.error && (
        <ProgressionError
          error={query.error}
          retry={() =>
            void (query.isFetchNextPageError ? query.fetchNextPage() : query.refetch())
          }
          context={context}
        />
      )}
      {!!pages && query.hasNextPage && !query.error && (
        <Button
          size="small"
          onClick={() => void query.fetchNextPage()}
          loading={query.isFetchingNextPage}
        >
          Show more
        </Button>
      )}
    </>
  );
}

interface RequirementNode {
  item: ProgressionRequirementView;
  children: RequirementNode[];
}

function flatPrerequisites(entries: ProgressionRequirementView[] | undefined) {
  if (!entries?.length || entries[0].depth !== 0) return;
  const [first, ...rest] = entries;
  if (first.kind === "all_of") {
    if (
      rest.every(
        ({ depth, kind }) => depth === 1 && kind !== "all_of" && kind !== "any_of",
      )
    ) {
      return rest;
    }
  } else if (first.kind !== "any_of" && entries.length === 1) {
    return entries;
  }
}

function prerequisiteKey({ kind, label, navigation }: ProgressionRequirementView) {
  if (navigation) return `record:${navigation.record_key}`;
  if (kind === "proficiency" || kind === "attribute_minimum") return `${kind}:${label}`;
}

function alsoRequires(
  leaves: ProgressionRequirementView[] | undefined,
  witnesses: ProgressionWitness[] | undefined,
  shownPrerequisiteKeys: ReadonlySet<string>,
) {
  const match = witnesses?.[0];
  if (
    !leaves ||
    witnesses?.length !== 1 ||
    !match ||
    match.alternative ||
    !leaves.some(({ path, kind }) => path === match.path && kind === "record")
  )
    return;
  return leaves
    .filter((leaf) => {
      const key = prerequisiteKey(leaf);
      return (
        leaf.path !== match.path &&
        (key === undefined || !shownPrerequisiteKeys.has(key))
      );
    })
    .map((leaf) => ({ ...leaf, depth: 0 }));
}

function RequirementList({
  entries,
  context,
  witnesses,
}: {
  entries: ProgressionRequirementView[];
  context: ProgressionContext;
  witnesses?: ProgressionWitness[];
}) {
  const roots: RequirementNode[] = [];
  const stack: RequirementNode[] = [];
  for (const item of entries) {
    const node = { item, children: [] };
    stack.length = Math.min(stack.length, item.depth);
    const parent = stack[stack.length - 1];
    (parent ? parent.children : roots).push(node);
    stack.push(node);
  }
  const witnessByPath = new Map(witnesses?.map((match) => [match.path, match]));
  return <RequirementNodes nodes={roots} context={context} witnesses={witnessByPath} />;
}

function RequirementNodes({
  nodes,
  context,
  witnesses,
}: {
  nodes: RequirementNode[];
  context: ProgressionContext;
  witnesses: Map<string, ProgressionWitness>;
}) {
  return (
    <ul className="record-progression__requirements">
      {nodes.map(({ item, children }) => {
        const witness = witnesses.get(item.path);
        return (
          <li key={item.path}>
            {item.navigation ? (
              <ProgressionReference
                label={item.label}
                navigation={item.navigation}
                context={{
                  ...context,
                  path: [...context.path, "requirement", item.path],
                }}
              />
            ) : item.kind === "all_of" || item.kind === "any_of" ? (
              <strong>{item.label}</strong>
            ) : (
              <span>{item.label}</span>
            )}
            {witness && (
              <span className="record-progression__witness">
                {witness.alternative
                  ? "Alternative prerequisite"
                  : "Matching prerequisite"}
              </span>
            )}
            {children.length > 0 && (
              <RequirementNodes
                nodes={children}
                context={context}
                witnesses={witnesses}
              />
            )}
          </li>
        );
      })}
    </ul>
  );
}

function ProgressionReference({
  context,
  label,
  navigation,
  requirements,
  witnesses,
  level,
}: {
  context: ProgressionContext;
  label: string;
  navigation: RecordNavigationView;
  requirements?: ProgressionRequirementView[];
  witnesses?: ProgressionWitness[];
  level?: number | null;
}) {
  const router = useAtlasRouter();
  const key = JSON.stringify(context.path);
  const expanded = context.expandedBranches.includes(key);
  const cycle = context.ancestors.includes(navigation.record_key);
  const alternatives =
    witnesses?.length && witnesses.every((entry) => entry.alternative);
  const address = { recordKey: navigation.record_key, selection: navigation };
  const destination = recordDestination({
    ...address,
    view: "explore",
    tab: "progression",
  });
  const explore = context.ancestors[0] !== navigation.record_key && (
    <InlineIconLink
      label={"Explore progression for " + label}
      title="Explore progression"
      icon={<Compass aria-hidden="true" />}
      href={destinationHref(destination)}
      onClick={(event) => {
        if (!isPrimaryActivation(event)) return;
        event.preventDefault();
        void navigateDestination(destination, undefined, router);
      }}
    />
  );

  return (
    <div className="record-progression__branch">
      <div className="record-progression__relation">
        <div className="record-progression__identity">
          <InlineContext actions={explore}>
            <AtlasLink
              {...recordDestination(address)}
              className="record-progression__title"
              onClick={(event) => {
                if (!isPrimaryActivation(event)) return;
                event.preventDefault();
                context.onReference(
                  navigation.record_key,
                  event.currentTarget,
                  navigation,
                );
              }}
            >
              {label}
            </AtlasLink>
          </InlineContext>
          {Boolean(alternatives) && !expanded && (
            <span className="record-progression__witness">
              Alternative prerequisite
            </span>
          )}
        </div>
        {level != null && (
          <span className="record-progression__level">Level {level}</span>
        )}
        {!cycle && (
          <PaneIconButton
            label={expanded ? "Collapse " + label : "Expand " + label}
            aria-expanded={expanded}
            onClick={() => context.setBranchExpanded(key, !expanded)}
            icon={expanded ? <ChevronDown size={16} /> : <ChevronRight size={16} />}
          />
        )}
      </div>
      {expanded && (
        <div className="record-progression__branches">
          <ProgressionOverview
            {...context}
            selection={navigation}
            requirements={requirements}
            witnesses={witnesses}
          />
        </div>
      )}
    </div>
  );
}
