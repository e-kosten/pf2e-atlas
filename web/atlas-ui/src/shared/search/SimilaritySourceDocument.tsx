import { useLayoutEffect, useRef } from "react";
import { useQuery } from "@tanstack/react-query";
import { Alert, Button, Spin } from "antd";
import type { SimilarityComponentView, SimilaritySeed } from "../../generated/atlas";
import { ErrorNotice } from "../feedback/ErrorNotice";
import { similarityTraitSourceQueryOptions } from "./similarityQueries";
import { rootNavigation, useReferenceReading } from "../records/useReferenceReading";
import { ReferenceReader } from "../records/ReferenceReader";
import { DocumentIdentityHeader } from "../records/DocumentIdentityHeader";
import { PreparedContentBody } from "../records/PreparedContent";
import { useRecordPreview } from "../records/useRecordPreview";
import { RecordReferenceDrawer } from "../records/RecordReferenceDrawer";
import { recordDestination } from "../../app/destinations";
import { navigateDestination } from "../../app/navigation";
import { isArtifactGenerationChanged } from "../queries/referenceQueries";
import { recordReadingIdentity, useRetainedRecord } from "../records/useRetainedRecord";
import {
  SourceSectionSelectionContext,
  type SourceSectionSelection,
} from "../records/SourceSectionSelectionContext";

export type SimilaritySourceTarget = {
  part: SimilarityComponentView;
  visit: number;
  opener?: HTMLElement;
} | null;

function mayHandOffSourceFocus(
  target: NonNullable<SimilaritySourceTarget>,
  document: Document,
) {
  const focused = document.activeElement;
  return focused === target.opener || focused === document.body || focused === null;
}

/** Source reading shares the exact reference requests, rendering and nested Drawer. */
export function SimilaritySourceDocument({
  seed,
  generation,
  target,
  active,
  selection = null,
  onReloadSections,
}: {
  seed: SimilaritySeed;
  generation: string | null;
  target: SimilaritySourceTarget;
  active: boolean;
  selection?: SourceSectionSelection | null;
  onReloadSections: () => void;
}) {
  return seed.kind === "record" ? (
    <RecordSourceDocument
      key={JSON.stringify([seed.record_key, generation])}
      recordKey={seed.record_key}
      generation={generation}
      target={target}
      selection={selection}
      active={active}
      onReloadSections={onReloadSections}
    />
  ) : (
    <TraitSourceDocument
      key={JSON.stringify([seed.trait_key, generation])}
      traitKey={seed.trait_key}
      generation={generation}
      target={target}
      selection={selection}
      active={active}
      onReloadSections={onReloadSections}
    />
  );
}

function RecordSourceDocument({
  recordKey,
  generation,
  target,
  active,
  selection = null,
  onReloadSections,
}: {
  recordKey: string;
  generation: string | null;
  target: SimilaritySourceTarget;
  active: boolean;
  selection?: SourceSectionSelection | null;
  onReloadSections: () => void;
}) {
  const reading = useReferenceReading(
    { navigation: rootNavigation(recordKey) },
    true,
    0,
    generation,
  );
  const { inspect } = reading;
  const root = useRef<HTMLDivElement>(null);
  useLayoutEffect(() => {
    if (target)
      inspect({
        navigation: target.part.navigation ?? rootNavigation(recordKey),
      });
  }, [target, recordKey, inspect]);
  const focusedVisit = useRef<number | null>(null);
  useLayoutEffect(() => {
    if (
      !target ||
      !active ||
      reading.loading ||
      !reading.detail ||
      JSON.stringify(reading.request?.navigation) !==
        JSON.stringify(target.part.navigation ?? rootNavigation(recordKey)) ||
      recordReadingIdentity(recordKey, reading.detail.selected) !==
        recordReadingIdentity(
          recordKey,
          target.part.navigation ?? rootNavigation(recordKey),
        ) ||
      focusedVisit.current === target.visit
    )
      return;
    // A source label may have become hidden by the phone's Document switch.
    // Enter the named document only for that explicit navigation.
    const heading = root.current?.querySelector<HTMLElement>(
      ".record-reference__identity h2",
    );
    if (heading) {
      focusedVisit.current = target.visit;
      if (mayHandOffSourceFocus(target, heading.ownerDocument)) {
        heading.tabIndex = -1;
        heading.focus({ preventScroll: true });
      }
    }
  }, [target, active, reading.loading, reading.detail, reading.request, recordKey]);
  const comparisonActive =
    reading.detail?.artifact_generation === generation &&
    reading.detail.selected.record_key === recordKey &&
    selection?.inventory.artifact_generation === generation &&
    selection.inventory.seed.kind === "record" &&
    selection.inventory.seed.record_key === recordKey;
  return (
    <div
      ref={root}
      className={`search-settings__record-source${comparisonActive ? " comparison-source" : ""}`}
    >
      <SourceSectionSelectionContext
        value={
          reading.detail?.artifact_generation === generation &&
          reading.detail.selected.record_key === recordKey
            ? { seed: { kind: "record", record_key: recordKey }, generation, selection }
            : null
        }
      >
        <ReferenceReader
          reading={
            isArtifactGenerationChanged(reading.error)
              ? { ...reading, retry: onReloadSections }
              : reading
          }
          retryLabel={
            isArtifactGenerationChanged(reading.error) ? "Reload sections" : "Retry"
          }
          selectedRecordKey={recordKey}
          active={active}
          label="Source document"
          returnToSelectionLabel="Whole document"
          showRelated={false}
          status={
            reading.detail && (
              <span className="search-settings__source-context">
                {reading.detail.record.title}
              </span>
            )
          }
        />
      </SourceSectionSelectionContext>
    </div>
  );
}

function TraitSourceDocument({
  traitKey,
  generation,
  target,
  active,
  selection = null,
  onReloadSections,
}: {
  traitKey: string;
  generation: string | null;
  target: SimilaritySourceTarget;
  active: boolean;
  selection?: SourceSectionSelection | null;
  onReloadSections: () => void;
}) {
  const query = useQuery(similarityTraitSourceQueryOptions(traitKey, generation));
  const detail = useRetainedRecord(
    JSON.stringify([traitKey, generation]),
    query.data,
    query.error,
  );
  const preview = useRecordPreview();
  const heading = useRef<HTMLHeadingElement>(null);
  const focusedVisit = useRef<number | null>(null);
  useLayoutEffect(() => {
    if (target && active && detail && focusedVisit.current !== target.visit) {
      focusedVisit.current = target.visit;
      if (
        heading.current &&
        mayHandOffSourceFocus(target, heading.current.ownerDocument)
      )
        heading.current.focus({ preventScroll: true });
    }
  }, [target, active, detail]);
  const comparisonActive =
    detail?.artifact_generation === generation &&
    selection?.inventory.artifact_generation === generation &&
    selection.inventory.seed.kind === "trait" &&
    selection.inventory.seed.trait_key === traitKey;
  return (
    <section
      className={`search-settings__trait-source${comparisonActive ? " comparison-source" : ""}`}
      aria-label="Trait document"
      aria-busy={query.isFetching}
    >
      {query.error && (
        <ErrorNotice
          error={query.error}
          title={detail ? "Unable to refresh this trait" : "Unable to load this trait"}
          description={detail ? "Previously loaded content remains below." : undefined}
          action={
            <Button
              onClick={() =>
                isArtifactGenerationChanged(query.error)
                  ? onReloadSections()
                  : void query.refetch()
              }
            >
              {isArtifactGenerationChanged(query.error) ? "Reload sections" : "Retry"}
            </Button>
          }
        />
      )}
      {detail ? (
        <article className="record-reference">
          <DocumentIdentityHeader
            title={detail.label}
            classification="Trait"
            headingRef={heading}
          />
          {query.isFetching && <Spin delay={500} size="small" />}
          {detail.content ? (
            <SourceSectionSelectionContext
              value={
                detail.artifact_generation === generation
                  ? {
                      seed: { kind: "trait", trait_key: traitKey },
                      generation,
                      selection,
                    }
                  : null
              }
            >
              <PreparedContentBody
                body={detail.content.body}
                sourceLocation={{ trait_key: traitKey }}
                sectionRendering={detail.content.section_rendering}
                references={detail.content.references}
                onReference={preview.open}
              />
            </SourceSectionSelectionContext>
          ) : (
            <Alert
              type="info"
              title="No definition is available in this reference collection."
            />
          )}
        </article>
      ) : (
        query.isPending && <Spin delay={500} description="Loading trait…" />
      )}
      {preview.recordKey && (
        <RecordReferenceDrawer
          anchor={preview.anchor}
          requestedNavigation={preview.selection}
          detail={preview.detail}
          loading={preview.loading}
          error={preview.error}
          onRetry={preview.retry}
          onClose={preview.close}
          onReference={preview.open}
          onRankChange={preview.changeRank}
          landing={preview.landing}
          readingHistory={preview.readingHistory}
          onOpenFullPage={() => {
            navigateDestination(
              recordDestination({
                recordKey: preview.recordKey!,
                selection: preview.selection,
                landing: preview.landing,
              }),
            );
            preview.close();
          }}
        />
      )}
    </section>
  );
}
