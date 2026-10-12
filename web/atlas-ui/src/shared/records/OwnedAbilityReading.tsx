import { Alert, Button, Collapse, Space, Spin } from "antd";
import { useInfiniteQuery } from "@tanstack/react-query";
import { useEffect, useMemo, useRef, useState } from "react";
import { getOwnedReading } from "../../api/atlasApi";
import type {
  ActorActivityView,
  OwnedReadingSelection,
  RecordNavigationView,
} from "../../generated/atlas";
import { ActorActivity } from "./ActorReference";
import { ContentFields } from "./RecordPresentation";
import type { RecordReferenceHandler } from "./PreparedContent";
import { activityAnchor, activityAddress } from "./ownedNavigation";
import { isArtifactGenerationChanged } from "../queries/immutableReferenceQueries";

export type OwnedReadingContext = {
  generation: string;
  parent: RecordNavigationView;
  landing?: RecordNavigationView;
  reloadParent?: () => void;
  reloadParentLabel?: string;
};

export function OwnedAbilityReading({
  abilities,
  context,
  onReference,
}: {
  abilities: ActorActivityView[];
  context?: OwnedReadingContext;
  onReference: RecordReferenceHandler;
}) {
  if (!context)
    return (
      <section className="actor-reference__activities">
        <h3>Abilities</h3>
        <Alert
          type="info"
          title="Ability content unavailable"
          description="Open the parent reference to read its authored abilities."
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
      context={context}
      onReference={onReference}
    />
  );
}
function OwnedAbilityBatch({
  abilities,
  context,
  onReference,
}: {
  abilities: ActorActivityView[];
  context: OwnedReadingContext;
  onReference: RecordReferenceHandler;
}) {
  const [open, setOpen] = useState(true);
  const [retryDirection, setRetryDirection] = useState<"next" | "previous">("next");
  const section = useRef<HTMLDivElement>(null);
  const sentinel = useRef<HTMLDivElement>(null);
  const landed = useRef(false);
  const landing = context.landing;
  const target = useMemo(() => (landing ? activityAddress(landing) : null), [landing]);
  const identity = JSON.stringify([
    context?.generation,
    context?.parent,
    "abilities",
    target,
  ]);
  const queryKey = useMemo(() => ["owned-reading", identity], [identity]);
  const reading = useInfiniteQuery({
    queryKey,
    enabled: open && Boolean(context),
    initialPageParam: {
      kind: "abilities",
      cursor: null,
      target,
    } as OwnedReadingSelection,
    queryFn: ({ pageParam, signal }) =>
      getOwnedReading(
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
            cursor: last.next,
            target: null,
          } as OwnedReadingSelection)
        : undefined,
    getPreviousPageParam: (first) =>
      first.previous
        ? ({
            kind: "abilities",
            cursor: first.previous,
            target: null,
          } as OwnedReadingSelection)
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
  }, [target, context?.generation]);
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
      element.scrollIntoView?.({ block: "start" });
      element.focus({ preventScroll: true });
    }
  }, [target, entries.length, open]);
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
  return (
    <div ref={section} className="owned-ability-reading">
      <Collapse
        ghost
        size="small"
        activeKey={open ? ["abilities"] : []}
        onChange={(keys) => setOpen(keys.includes("abilities"))}
        items={[
          {
            key: "abilities",
            label: <h3>Abilities</h3>,
            styles: { header: { paddingInline: 0 }, body: { paddingInline: 0 } },
            children: (
              <>
                {!context && (
                  <Alert
                    type="info"
                    title="Ability content unavailable"
                    description="Open the parent reference to read its authored abilities."
                  />
                )}
                {first && first.start > 0 && (
                  <div className="owned-reading-continuation">
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
                      activityAnchor(candidate.navigation) ===
                      activityAnchor(entry.navigation),
                  );
                  return activity ? (
                    <section
                      key={activityAnchor(entry.navigation)}
                      className="owned-ability-reading__entry"
                    >
                      <ActorActivity
                        activity={activity}
                        onReference={onReference}
                        inline
                      />
                      <ContentFields
                        inlineAbility
                        fields={entry.content}
                        relationships={entry.relationships}
                        onReference={onReference}
                      />
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
                  <Alert
                    type="error"
                    title="Unable to load ability content"
                    description={reading.error.message}
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
                {loading && (
                  <div role="status">
                    <Spin size="small" /> Loading abilities…
                  </div>
                )}
                {last && last.end < last.total && (
                  <div
                    ref={sentinel}
                    className="owned-reading-continuation"
                    aria-label="Continue reading abilities"
                  >
                    <p>More abilities continue below as you read.</p>
                  </div>
                )}
              </>
            ),
          },
        ]}
      />
    </div>
  );
}
