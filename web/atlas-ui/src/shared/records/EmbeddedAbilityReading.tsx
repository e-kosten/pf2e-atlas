import { ErrorNotice } from "../feedback/ErrorNotice";
import { Alert, Button, Collapse, Space } from "antd";
import { LoadingStatus } from "../ui/LoadingStatus";
import { useInfiniteQuery } from "@tanstack/react-query";
import { useEffect, useMemo, useRef, useState } from "react";
import { getEmbeddedReading } from "../../api/atlasApi";
import type {
  ActorActivityView,
  ActorAbilityGroupView,
  EmbeddedReadingSelection,
  RecordNavigationView,
} from "../../generated/atlas";
import { ActorActivity } from "./ActorReference";
import { ContentFields } from "./RecordPresentation";
import type { RecordReferenceHandler } from "./PreparedContent";
import { activityAnchor, activityAddress } from "./embeddedNavigation";
import { isArtifactGenerationChanged } from "../queries/immutableReferenceQueries";

export type EmbeddedReadingContext = {
  renderAbilityControls?: (navigation: RecordNavigationView) => {
    activation?: import("react").ReactNode;
    usage?: import("react").ReactNode;
  };
  generation: string;
  parent: RecordNavigationView;
  landing?: RecordNavigationView;
  // The reading host can restore a history bookmark only after this target's
  // actual content is mounted; the target remains part of the bounded request.
  onLandingReady?: (target?: HTMLElement) => void;
  reloadParent?: () => void;
  reloadParentLabel?: string;
};

export function EmbeddedAbilityReading({
  abilities,
  group,
  title = "Abilities",
  context,
  onReference,
  essential = false,
}: {
  essential?: boolean;
  abilities: ActorActivityView[];
  group?: ActorAbilityGroupView;
  title?: string;
  context?: EmbeddedReadingContext;
  onReference: RecordReferenceHandler;
}) {
  if (!context)
    return (
      <section className="actor-reference__activities">
        <h3>{title}</h3>
        <Alert
          type="info"
          title="Ability content unavailable"
          description="Open the parent reference to read its abilities."
        />
        {abilities.map((activity) => (
          <ActorActivity
            key={activityAnchor(activity.navigation)}
            activity={activity}
            onReference={onReference}
            inline
          />
        ))}
      </section>
    );
  return (
    <OwnedAbilityBatch
      abilities={abilities}
      group={group}
      title={title}
      essential={essential}
      context={context}
      onReference={onReference}
    />
  );
}
function OwnedAbilityBatch({
  abilities,
  group,
  title = "Abilities",
  context,
  onReference,
  essential = false,
}: {
  essential?: boolean;
  abilities: ActorActivityView[];
  group?: ActorAbilityGroupView;
  title?: string;
  context: EmbeddedReadingContext;
  onReference: RecordReferenceHandler;
}) {
  const [open, setOpen] = useState(true);
  const [retryDirection, setRetryDirection] = useState<"next" | "previous">("next");
  const section = useRef<HTMLDivElement>(null);
  const sentinel = useRef<HTMLDivElement>(null);
  const landed = useRef(false);
  const landing = context.landing;
  const onLandingReady = context.onLandingReady;
  const target = useMemo(() => (landing ? activityAddress(landing) : null), [landing]);
  const identity = JSON.stringify([
    context?.generation,
    context?.parent,
    "abilities",
    group,
    target,
  ]);
  const queryKey = useMemo(() => ["embedded-reading", identity], [identity]);
  const reading = useInfiniteQuery({
    queryKey,
    enabled: open && Boolean(context),
    initialPageParam: {
      kind: "abilities",
      group: group ?? null,
      cursor: null,
      target,
    } as EmbeddedReadingSelection,
    queryFn: ({ pageParam, signal }) =>
      getEmbeddedReading(
        {
          parent: context!.parent,
          artifact_generation: context!.generation,
          selection: pageParam,
        },
        signal,
      ),
    getNextPageParam: (last) =>
      last.next
        ? ({
            kind: "abilities",
            group: group ?? null,
            cursor: last.next,
            target: null,
          } as EmbeddedReadingSelection)
        : undefined,
    getPreviousPageParam: (first) =>
      first.previous
        ? ({
            kind: "abilities",
            group: group ?? null,
            cursor: first.previous,
            target: null,
          } as EmbeddedReadingSelection)
        : undefined,
    staleTime: Infinity,
    retry: false,
  });
  const pages = reading.data?.pages ?? [];
  const entries = pages.flatMap((page) => page.entries);
  const first = pages[0];
  const last = pages[pages.length - 1];
  const loading = reading.isFetching;
  const { hasNextPage, isError, fetchNextPage } = reading;
  useEffect(() => {
    landed.current = false;
  }, [target, context?.generation, onLandingReady]);
  useEffect(() => {
    if (!target || landed.current || !entries.length || !open) return;
    const element = Array.from(
      section.current?.querySelectorAll<HTMLElement>("[data-atlas-activity-anchor]") ??
        [],
    ).find(
      (candidate) => candidate.dataset.atlasActivityAnchor === activityAnchor(target),
    );
    if (element) {
      landed.current = true;
      if (!onLandingReady) element.scrollIntoView?.({ block: "start" });
      element.focus({ preventScroll: true });
      onLandingReady?.(element);
    }
  }, [target, entries.length, open, onLandingReady]);
  useEffect(() => {
    if (!open || !hasNextPage || loading || isError) return;
    // Scroll events are captured from the real reading host. Layout/visibility
    // alone never starts another batch, so an idle visible sentinel stays finite.
    const positions = new WeakMap<EventTarget, number>();
    const scroll = (event: Event) => {
      const host = event.target;
      if (!host || !section.current || !sentinel.current) return;
      const documentHost = host === document || host === window;
      if (
        !documentHost &&
        (!(host instanceof HTMLElement) || !host.contains(section.current))
      )
        return;
      const top = documentHost ? window.scrollY : (host as HTMLElement).scrollTop;
      const previous = positions.get(host) ?? 0;
      positions.set(host, top);
      if (top <= previous) return;
      const bounds = documentHost
        ? { top: 0, bottom: window.innerHeight, height: window.innerHeight }
        : (host as HTMLElement).getBoundingClientRect();
      const marker = sentinel.current.getBoundingClientRect();
      if (
        marker.top <= bounds.bottom + bounds.height / 2 &&
        marker.bottom >= bounds.top
      ) {
        setRetryDirection("next");
        void fetchNextPage({ cancelRefetch: false });
      }
    };
    window.addEventListener("scroll", scroll, true);
    return () => window.removeEventListener("scroll", scroll, true);
  }, [open, loading, hasNextPage, isError, fetchNextPage]);
  const content = (
    <>
      {!context && (
        <Alert
          type="info"
          title="Ability content unavailable"
          description="Open the parent reference to read its abilities."
        />
      )}
      {first && first.start > 0 && (
        <div className="embedded-reading-continuation">
          <p>Earlier abilities are outside this reading range.</p>
          <Button
            size="small"
            disabled={isArtifactGenerationChanged(reading.error)}
            loading={reading.isFetchingPreviousPage}
            onClick={() => {
              setRetryDirection("previous");
              void reading.fetchPreviousPage({ cancelRefetch: false });
            }}
          >
            Read earlier abilities
          </Button>
        </div>
      )}
      {entries.map((entry) => {
        const activity = abilities.find(
          (candidate) =>
            activityAnchor(candidate.navigation) === activityAnchor(entry.navigation),
        );
        const controls = activity
          ? context.renderAbilityControls?.(activity.navigation)
          : undefined;
        return activity ? (
          <section
            key={activityAnchor(entry.navigation)}
            className="embedded-ability-reading__entry"
          >
            <ActorActivity
              activity={activity}
              onReference={onReference}
              inline
              activationControls={controls?.activation}
              hideFrequency={Boolean(controls?.usage)}
            />
            <ContentFields
              inlineAbility
              fields={entry.content}
              relationships={entry.relationships}
              onReference={onReference}
            />
            {controls?.usage && (
              <div className="embedded-ability-reading__usage">{controls.usage}</div>
            )}
          </section>
        ) : (
          <Alert
            key={activityAnchor(entry.navigation)}
            type="info"
            title="Activity no longer available"
            description="Reload the parent reference to refresh this reading range."
          />
        );
      })}
      {reading.isError && (
        <ErrorNotice
          error={reading.error}
          title="Unable to load ability content"
          action={
            <Space wrap>
              <Button
                size="small"
                disabled={isArtifactGenerationChanged(reading.error)}
                onClick={() => {
                  if (!pages.length) void reading.refetch();
                  else if (retryDirection === "previous")
                    void reading.fetchPreviousPage();
                  else void reading.fetchNextPage();
                }}
              >
                Retry
              </Button>
              {context.reloadParent && (
                <Button size="small" onClick={context.reloadParent}>
                  {context.reloadParentLabel ?? "Reload reference"}
                </Button>
              )}
            </Space>
          }
        />
      )}
      {loading && <LoadingStatus label="Loading abilities…" />}
      {last && last.end < last.total && (
        <div
          ref={sentinel}
          className="embedded-reading-continuation"
          aria-label="Continue reading abilities"
        >
          <p>More abilities continue below as you read.</p>
        </div>
      )}
    </>
  );
  return (
    <div ref={section} className="embedded-ability-reading">
      {essential ? (
        <section>
          <h3>{title}</h3>
          {content}
        </section>
      ) : (
        <Collapse
          ghost
          size="small"
          activeKey={open ? ["abilities"] : []}
          onChange={(keys) => setOpen(keys.includes("abilities"))}
          items={[
            {
              key: "abilities",
              label: <h3>{title}</h3>,
              styles: { header: { paddingInline: 0 }, body: { paddingInline: 0 } },
              children: content,
            },
          ]}
        />
      )}
    </div>
  );
}
