import { useState } from "react";
import type {
  EncounterParticipantView,
  UpdateEncounterParticipantRequest,
} from "../../generated/atlas";
import { EditableValue } from "../../shared/ui/forms/EditableValue";
import { participantUpdate } from "./participantEdits";

export function EncounterParticipantName({
  participant,
  disabled,
  onUpdate,
}: {
  participant: EncounterParticipantView;
  disabled: boolean;
  onUpdate: (request: UpdateEncounterParticipantRequest) => Promise<boolean>;
}) {
  const [error, setError] = useState<string>();
  return (
    <div className="encounter-participant-name">
      <h2 aria-label={participant.display_name}>
        <EditableValue
          label="Participant name"
          value={participant.display_name}
          disabled={disabled}
          onCancel={() => setError(undefined)}
          buttonClassName="encounter-participant-name__value"
          editorClassName="encounter-participant-name__editor"
          inputClassName="encounter-participant-name__input"
          onCommit={async (input) => {
            setError(undefined);
            const name = input.trim();
            if (!name) {
              setError("Enter a participant name.");
              throw new Error("Participant name is required");
            }
            if (name === participant.display_name) return true;
            return onUpdate(participantUpdate(participant, { display_name: name }));
          }}
        />
      </h2>
      {error && <span role="alert">{error}</span>}
    </div>
  );
}
