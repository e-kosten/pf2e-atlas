import { Alert } from "antd";
import { memo } from "react";
import type {
  EncounterSpellCastRequest,
  RecordSurfaceView,
} from "../../generated/atlas";
import {
  CreatureDetailSurface,
  RecordHeader,
  SearchCompactSurface,
} from "./CreatureRecordSurface";
import {
  EncounterParticipantSurface,
  type EncounterRecordSurfaceSlots,
} from "./EncounterRecordSurface";
import {
  HazardDetailSurface,
  HazardEncounterSurface,
  HazardSearchCompactSurface,
} from "./HazardRecordSurface";
import {
  SpellDetailSurface,
  type SpellFormSelection,
  SpellSearchCompactSurface,
} from "./SpellRecordSurface";

type RecordSurfaceProps = {
  onReference: (recordKey: string) => void;
  onSpellFormSelection?: (selection: SpellFormSelection) => void;
  onSpellCast?: (request: EncounterSpellCastRequest) => void;
  showTitle?: boolean;
  spellCatalog?: RecordSurfaceView;
  spellFormSelection?: SpellFormSelection;
  spellFormSelectionLoading?: boolean;
  surface: RecordSurfaceView;
  slots?: EncounterRecordSurfaceSlots;
};

export const RecordSurface = memo(function RecordSurface({
  onReference,
  onSpellFormSelection,
  onSpellCast,
  showTitle = true,
  spellCatalog,
  spellFormSelection,
  spellFormSelectionLoading,
  surface,
  slots = {},
}: RecordSurfaceProps) {
  if (surface.presentation.presentation_type === "unavailable") {
    return (
      <article className="record-surface record-surface--unavailable">
        <RecordHeader metadata={surface.metadata} showTitle={showTitle} />
        <Alert
          description={surface.presentation.unavailable.message}
          message="This record family is not available in the typed record surface yet."
          showIcon
          type="info"
        />
      </article>
    );
  }

  if (surface.presentation.presentation_type === "hazard") {
    const body = surface.presentation.body;
    if (surface.profile === "search_compact") {
      return <HazardSearchCompactSurface body={body} metadata={surface.metadata} />;
    }
    if (surface.profile === "encounter_participant") {
      return (
        <HazardEncounterSurface
          body={body}
          metadata={surface.metadata}
          onReference={onReference}
          runtime={surface.encounter}
          slots={slots}
        />
      );
    }
    return (
      <HazardDetailSurface
        body={body}
        metadata={surface.metadata}
        onReference={onReference}
        showTitle={showTitle}
      />
    );
  }

  if (surface.presentation.presentation_type === "spell") {
    const body = surface.presentation.body;
    if (surface.profile === "search_compact") {
      return <SpellSearchCompactSurface body={body} metadata={surface.metadata} />;
    }
    const catalog =
      spellCatalog?.presentation.presentation_type === "spell"
        ? spellCatalog.presentation.body
        : body;
    return (
      <SpellDetailSurface
        body={body}
        catalog={catalog}
        metadata={surface.metadata}
        onReference={onReference}
        onSelectionChange={onSpellFormSelection}
        selection={spellFormSelection}
        selectionLoading={spellFormSelectionLoading}
        showTitle={showTitle}
      />
    );
  }

  const body = surface.presentation.body;
  if (surface.profile === "search_compact") {
    return <SearchCompactSurface body={body} metadata={surface.metadata} />;
  }

  if (surface.profile === "encounter_participant") {
    return (
      <EncounterParticipantSurface
        body={body}
        metadata={surface.metadata}
        onReference={onReference}
        onSpellCast={onSpellCast}
        runtime={surface.encounter}
        slots={slots}
      />
    );
  }

  return (
    <CreatureDetailSurface
      body={body}
      metadata={surface.metadata}
      onReference={onReference}
      showTitle={showTitle}
    />
  );
});
