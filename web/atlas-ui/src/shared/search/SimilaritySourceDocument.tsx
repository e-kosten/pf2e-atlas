import { useLayoutEffect, useMemo, useRef } from "react";
import type { SimilarityComponentView, SimilaritySeed } from "../../generated/atlas";
import { documentSeed, seedNavigation, sameSimilaritySeed } from "./similaritySource";
import { useReferenceReading } from "../records/useReferenceReading";
import { ReferenceReader } from "../records/ReferenceReader";
import { ReferenceHelpHost } from "../records/ReferenceHelpHost";
import { isArtifactGenerationChanged } from "../queries/referenceQueries";
import { recordReadingIdentity } from "../records/useRetainedRecord";
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
  return (
    <ReferenceHelpHost generation={generation ?? undefined}>
      <RecordSourceDocument
        key={JSON.stringify([seed, generation])}
        seed={seed}
        generation={generation}
        target={target}
        selection={selection}
        active={active}
        onReloadSections={onReloadSections}
      />
    </ReferenceHelpHost>
  );
}

function RecordSourceDocument({
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
  const recordKey = seed.record_key;
  const document = useMemo(() => seedNavigation(seed), [seed]);
  const reading = useReferenceReading(
    { navigation: document },
    true,
    0,
    generation,
    recordKey,
  );
  const { inspect } = reading;
  const root = useRef<HTMLDivElement>(null);
  useLayoutEffect(() => {
    if (target)
      inspect({
        navigation: target.part.navigation ?? document,
      });
  }, [target, document, inspect]);
  const focusedVisit = useRef<number | null>(null);
  useLayoutEffect(() => {
    if (
      !target ||
      !active ||
      reading.loading ||
      !reading.detail ||
      JSON.stringify(reading.request?.navigation) !==
        JSON.stringify(target.part.navigation ?? document) ||
      recordReadingIdentity(recordKey, reading.detail.selected) !==
        recordReadingIdentity(recordKey, target.part.navigation ?? document) ||
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
  }, [
    target,
    active,
    reading.loading,
    reading.detail,
    reading.request,
    document,
    recordKey,
  ]);
  const readingSource =
    reading.detail?.artifact_generation === generation &&
    reading.detail.selected.record_key === recordKey &&
    (!seed.embedded_path?.length ||
      sameSimilaritySeed(documentSeed(reading.detail.selected), seed));
  const comparisonActive =
    readingSource &&
    selection?.inventory.artifact_generation === generation &&
    sameSimilaritySeed(selection.inventory.seed, seed);
  return (
    <div
      ref={root}
      className={`search-settings__record-source${comparisonActive ? " comparison-source" : ""}`}
    >
      <SourceSectionSelectionContext
        value={readingSource ? { seed, generation, selection } : null}
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
          selectedDocument={document}
          active={active}
          label="Source document"
          returnToSelectionLabel="Whole document"
          showRelated={false}
          status={
            reading.detail && (
              <span className="search-settings__source-context">
                {reading.detail.presentation.identity.title}
              </span>
            )
          }
        />
      </SourceSectionSelectionContext>
    </div>
  );
}
