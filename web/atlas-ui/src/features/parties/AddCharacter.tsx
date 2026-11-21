import { Button, Dropdown } from "antd";
import { useState, type Ref } from "react";
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
  triggerRef?: Ref<HTMLButtonElement>;
  onManual?: () => void;
}) {
  const paused = useSavedWritesPaused();
  const [mode, setMode] = useState<string>();
  return (
    <>
      <Dropdown
        trigger={["click"]}
        menu={{
          items: [
            ...(party ? [{ key: "existing", label: "Existing character" }] : []),
            { key: "pathbuilder", label: "From Pathbuilder" },
            { key: "manual", label: "Enter manually" },
          ],
          onClick: ({ key }) => {
            if (key === "manual" && onManual) onManual();
            else setMode(key);
          },
        }}
      >
        <Button ref={triggerRef} type="primary" disabled={paused || disabled}>
          Add PC
        </Button>
      </Dropdown>
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
