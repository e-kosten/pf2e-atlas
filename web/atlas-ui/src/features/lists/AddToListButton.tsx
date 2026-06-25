import { ListPlus } from "lucide-react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Modal, Select, Tooltip } from "antd";
import { useState } from "react";
import { addSavedListItem } from "../../api/atlasApi";
import { PaneIconButton } from "../../shared/ui/actions/PaneAction";
import { useSavedLists } from "./savedListQueries";

export function AddToListButton({ recordKey }: { recordKey: string }) {
  const [open, setOpen] = useState(false);
  const [selectedSlug, setSelectedSlug] = useState<string | null>(null);
  const queryClient = useQueryClient();
  const lists = useSavedLists({ enabled: open });
  const addItem = useMutation({
    mutationFn: (slug: string) =>
      addSavedListItem({ list_ref: slug, record_ref: recordKey }),
    onSuccess: async (_view, slug) => {
      await queryClient.invalidateQueries({ queryKey: ["saved-list", slug] });
      await queryClient.invalidateQueries({ queryKey: ["saved-lists"] });
      setOpen(false);
      setSelectedSlug(null);
    },
  });

  return (
    <>
      <Tooltip title="Add to saved list">
        <PaneIconButton
          icon={<ListPlus size={16} />}
          label="Add to saved list"
          onClick={() => setOpen(true)}
        />
      </Tooltip>
      <Modal
        okButtonProps={{ disabled: selectedSlug === null, loading: addItem.isPending }}
        okText="Add"
        onCancel={() => {
          setOpen(false);
          setSelectedSlug(null);
        }}
        onOk={() => {
          if (selectedSlug !== null) {
            addItem.mutate(selectedSlug);
          }
        }}
        open={open}
        title="Add to List"
      >
        <div className="list-picker">
          <Select
            loading={lists.isLoading || lists.isFetching}
            onChange={(slug) => setSelectedSlug(slug)}
            options={(lists.data?.lists ?? []).map((list) => ({
              label: list.name,
              value: list.slug,
            }))}
            placeholder="Select a saved list"
            value={selectedSlug}
          />
          {addItem.error && <div className="error-banner">{addItem.error.message}</div>}
        </div>
      </Modal>
    </>
  );
}
