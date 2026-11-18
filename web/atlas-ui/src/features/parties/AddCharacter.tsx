import { Button, Dropdown } from "antd";
import { useState } from "react";
import type { PartyView } from "../../generated/atlas";
import { useSavedWritesPaused } from "../../shared/feedback/SavedWritesPaused";
import { CharacterEditor } from "./CharacterEditor";
import { CharacterPicker } from "./CharacterPicker";
import { PathbuilderImport } from "./PathbuilderImport";

export function AddCharacter({
  party,
  onSaved,
  onManual,
}: {
  party?: PartyView;
  onSaved: (key: string) => void;
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
        <Button type="primary" disabled={paused}>
          Add PC
        </Button>
      </Dropdown>
      {mode === "manual" && (
        <CharacterEditor
          party={party}
          onClose={() => setMode(undefined)}
          onSaved={onSaved}
        />
      )}
      {mode === "pathbuilder" && (
        <PathbuilderImport
          party={party}
          onClose={() => setMode(undefined)}
          onSaved={(key) => {
            if (key) onSaved(key);
          }}
        />
      )}
      {mode === "existing" && party && (
        <CharacterPicker party={party} onClose={() => setMode(undefined)} />
      )}
    </>
  );
}
