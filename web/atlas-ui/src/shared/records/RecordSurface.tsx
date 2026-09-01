import { Alert } from "antd";
import { memo } from "react";
import type React from "react";
import type { RecordSurfaceView } from "../../generated/atlas";
import {
  CreatureDetailSurface,
  RecordHeader,
  SearchCompactSurface,
} from "./CreatureRecordSurface";
import { EncounterParticipantSurface } from "./EncounterRecordSurface";

type RecordSurfaceProps = {
  onReference: (recordKey: string) => void;
  showTitle?: boolean;
  surface: RecordSurfaceView;
  slots?: Record<string, React.ReactNode>;
};

export const RecordSurface = memo(function RecordSurface({
  onReference,
  showTitle = true,
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

  const body = surface.presentation.body;
  if (surface.profile === "search_compact") {
    return (
      <SearchCompactSurface
        body={body}
        metadata={surface.metadata}
        onReference={onReference}
      />
    );
  }

  if (surface.profile === "encounter_participant") {
    return (
      <EncounterParticipantSurface
        body={body}
        metadata={surface.metadata}
        onReference={onReference}
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
