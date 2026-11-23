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
        "Removes this character from all parties. Existing encounters keep their stats and lose access to their abilities.",
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
