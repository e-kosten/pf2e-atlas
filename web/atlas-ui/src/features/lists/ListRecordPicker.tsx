import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Alert, Form } from "antd";
import { useRef, useState } from "react";
import { addSavedListItem } from "../../api/atlasApi";
import type { SavedListSummaryView } from "../../generated/atlas";
import {
  buildSearchRequest,
  DEFAULT_SEARCH_STATE,
} from "../../shared/filters/searchState";
import { SearchPickerModal } from "../../shared/ui/pickers/SearchPickerModal";
import { invalidateSavedListQueries } from "./savedListQueries";

/** The shared picker owns browsing; this list owner alone commits membership. */
export function ListRecordPicker({
  list,
  open,
  onClose,
}: {
  list: SavedListSummaryView | undefined;
  open: boolean;
  onClose: () => void;
}) {
  const [form] = Form.useForm();
  const client = useQueryClient();
  const [candidate, setCandidate] = useState<string | null>(null);
  const [receipt, setReceipt] = useState<{ record: string; message: string } | null>(
    null,
  );
  const submitting = useRef(false);
  const add = useMutation({
    mutationFn: (request: { list_ref: string; record_ref: string }) =>
      addSavedListItem(request),
    onSuccess: async (view, request) => {
      setReceipt({
        record: request.record_ref,
        message:
          view.outcome === "already_present"
            ? "Already in this list."
            : "Added to list.",
      });
      await invalidateSavedListQueries(client, [
        request.list_ref,
        view.slug,
        view.list_key,
      ]);
    },
  });
  function close() {
    if (submitting.current) return;
    setReceipt(null);
    add.reset();
    setCandidate(null);
    onClose();
  }
  async function submit() {
    if (!candidate || !list || submitting.current || receipt?.record === candidate)
      return;
    submitting.current = true;
    setReceipt(null);
    try {
      await add.mutateAsync({ list_ref: list.slug, record_ref: candidate });
    } catch {
      /* Mutation error stays with Add for retry. */
    } finally {
      submitting.current = false;
    }
  }
  return (
    <SearchPickerModal
      title="Add records"
      open={open}
      form={form}
      disabled={add.isPending}
      onCancel={close}
      onFinish={() => void submit()}
      selectedRecordKey={candidate}
      onSelectedRecordKeyChange={(record) => {
        setCandidate(record);
        add.reset();
        setReceipt(null);
      }}
      emptyPrompt="Search or choose filters."
      emptyResults="No matching records."
      buildRequest={(query) =>
        buildSearchRequest({ ...DEFAULT_SEARCH_STATE, query }, 1)
      }
      okButtonProps={{
        loading: add.isPending,
        disabled: !list || receipt?.record === candidate,
      }}
    >
      {receipt && <span role="status">{receipt.message}</span>}
      {add.error && (
        <Alert
          type="error"
          title="Record could not be added"
          description={add.error.message}
        />
      )}
    </SearchPickerModal>
  );
}
