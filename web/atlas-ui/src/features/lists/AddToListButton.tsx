import { ListPlus } from "lucide-react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Alert, Button, Modal, Select, Tooltip, Typography } from "antd";
import { useRef, useState } from "react";
import { addSavedListItem } from "../../api/atlasApi";
import { PaneIconButton } from "../../shared/ui/actions/PaneAction";
import { invalidateSavedListQueries, useSavedLists } from "./savedListQueries";
import {
  listsPath,
  navigateToAtlasRoute,
  shouldHandleAtlasRouteClick,
} from "../../app/routes";

export function AddToListButton({ recordKey }: { recordKey: string }) {
  const [open, setOpen] = useState(false);
  const [selectedSlug, setSelectedSlug] = useState<string | null>(null);
  const queryClient = useQueryClient();
  const inFlight = useRef(false);
  const lists = useSavedLists({ enabled: open });
  const addItem = useMutation({
    mutationFn: (slug: string) =>
      addSavedListItem({ list_ref: slug, record_ref: recordKey }),
    onSuccess: async (view, slug) => {
      await invalidateSavedListQueries(queryClient, [slug, view.slug, view.list_key]);
      setOpen(false);
      setSelectedSlug(null);
    },
    onSettled: () => {
      inFlight.current = false;
    },
  });

  return (
    <>
      <Tooltip title="Add to saved list" trigger={["hover", "focus"]}>
        <PaneIconButton
          icon={<ListPlus size={16} />}
          label="Add to saved list"
          onClick={(event) => {
            event.currentTarget.focus({ preventScroll: true });
            setOpen(true);
          }}
        />
      </Tooltip>
      <Modal
        transitionName=""
        confirmLoading={addItem.isPending}
        okButtonProps={{ disabled: selectedSlug === null || !lists.data?.lists.length }}
        cancelButtonProps={{ disabled: addItem.isPending }}
        closable={{ disabled: addItem.isPending }}
        keyboard={!addItem.isPending}
        mask={{ closable: !addItem.isPending }}
        okText="Add"
        onCancel={() => {
          if (inFlight.current) return;
          setOpen(false);
          setSelectedSlug(null);
          addItem.reset();
        }}
        onOk={() => {
          if (selectedSlug !== null && !inFlight.current && lists.data?.lists.length) {
            inFlight.current = true;
            addItem.mutate(selectedSlug);
          }
        }}
        open={open}
        title="Add to List"
      >
        <div className="list-picker">
          <Select
            aria-label="Saved list"
            disabled={addItem.isPending || !lists.data?.lists.length}
            loading={lists.isLoading || lists.isFetching}
            onChange={(slug) => setSelectedSlug(slug)}
            options={(lists.data?.lists ?? []).map((list) => ({
              label: list.name,
              value: list.slug,
            }))}
            placeholder="Select a saved list"
            value={selectedSlug}
          />
          {lists.error && (
            <Alert
              type="error"
              title={
                lists.data
                  ? "Saved lists could not refresh"
                  : "Saved lists could not load"
              }
              description={
                lists.data ? "Previously loaded lists remain available." : undefined
              }
              action={
                <Button
                  size="small"
                  loading={lists.isFetching}
                  onClick={() => void lists.refetch()}
                >
                  Retry
                </Button>
              }
            />
          )}
          {lists.data?.lists.length === 0 && !lists.error && (
            <Typography.Paragraph>
              No saved lists yet.{" "}
              <Typography.Link
                href={listsPath()}
                onClick={(event) => {
                  if (!shouldHandleAtlasRouteClick(event)) return;
                  event.preventDefault();
                  navigateToAtlasRoute({ kind: "lists" });
                }}
              >
                Open saved lists
              </Typography.Link>
            </Typography.Paragraph>
          )}
          {addItem.error && <Alert type="error" title={addItem.error.message} />}
        </div>
      </Modal>
    </>
  );
}
