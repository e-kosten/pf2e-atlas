import { useQueryClient } from "@tanstack/react-query";
import { deleteCharacter } from "../../api/atlasApi";
import type { CharacterSummaryView } from "../../generated/atlas";
import { useConfirmDangerAction } from "../../shared/ui/actions/useConfirmDangerAction";
import { refreshCharacters } from "./characterQueries";

export function useDeleteCharacter(onDeleted?: () => Promise<void>) {
  const client = useQueryClient();
  const { confirm, dialog } = useConfirmDangerAction();

  function confirmDelete(
    character: Pick<CharacterSummaryView, "character_key" | "name">,
    revision: number,
  ) {
    confirm({
      title: `Delete ${character.name}?`,
      okText: "Delete character",
      content:
        "This removes the character from all parties and detaches its live rules from encounters. Encounter participants keep their names, levels, HP and initiative.",
      onConfirm: async () => {
        try {
          await deleteCharacter(character.character_key, {
            expected_revision: revision,
          });
          await onDeleted?.();
        } finally {
          await refreshCharacters(client);
        }
      },
    });
  }
  return { confirmDelete, dialog };
}
