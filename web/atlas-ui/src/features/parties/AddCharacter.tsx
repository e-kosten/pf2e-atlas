import { useRef, useState, type RefObject } from "react";
import { EntityActionsMenu } from "../../shared/ui/actions/EntityActionsMenu";
import type { PartyView } from "../../generated/atlas";
import { useSavedWritesPaused } from "../../shared/feedback/SavedWritesPaused";
import { CharacterEditor } from "./CharacterEditor";
import { CharacterPicker } from "./CharacterPicker";
import { PathbuilderImport } from "./PathbuilderImport";

export function AddCharacter({
  party,
  onSaved,
  onManual,
  disabled = false,
  triggerRef,
}: {
  party?: PartyView;
  onSaved?: (key: string) => void;
  disabled?: boolean;
  triggerRef?: RefObject<HTMLButtonElement | null>;
  onManual?: () => void;
}) {
  const paused = useSavedWritesPaused();
  const localTrigger = useRef<HTMLButtonElement>(null);
  const [mode, setMode] = useState<string>();
  return (
    <>
      <EntityActionsMenu
        label="Add PC"
        appearance="primary"
        trigger={triggerRef ?? localTrigger}
        disabled={paused || disabled}
        items={[
          ...(party ? [{ key: "existing", label: "Existing character" }] : []),
          { key: "pathbuilder", label: "From Pathbuilder" },
          { key: "manual", label: "Enter manually" },
        ]}
        onAction={(key) => {
          if (key === "manual" && onManual) onManual();
          else setMode(key);
        }}
      />
      {mode === "manual" && (
        <CharacterEditor
          party={party}
          onClose={() => setMode(undefined)}
          onSaved={(key) => onSaved?.(key)}
        />
      )}
      {mode === "pathbuilder" && (
        <PathbuilderImport
          party={party}
          onClose={() => setMode(undefined)}
          onSaved={(key) => {
            if (key) onSaved?.(key);
          }}
        />
      )}
      {mode === "existing" && party && (
        <CharacterPicker party={party} onClose={() => setMode(undefined)} />
      )}
    </>
  );
}
