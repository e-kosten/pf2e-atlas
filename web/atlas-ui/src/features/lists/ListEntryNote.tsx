import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Button, Input, Space, Typography } from "antd";
import type { GetRef } from "antd";
import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { getSavedListItemNote, updateSavedListItemNote } from "../../api/atlasApi";
import { invalidateSavedListQueries } from "./savedListQueries";

export function ListEntryNote({
  listRef,
  recordKey,
  hidden = false,
}: {
  listRef: string;
  recordKey: string;
  hidden?: boolean;
}) {
  const client = useQueryClient();
  const editButton = useRef<HTMLButtonElement>(null);
  const editor = useRef<GetRef<typeof Input.TextArea>>(null);
  const restoreEditFocus = useRef(false);
  function finishEditing() {
    restoreEditFocus.current = true;
    setDraft(null);
  }
  const queryKey = ["saved-list-item-note", listRef, recordKey];
  const note = useQuery({
    queryKey,
    queryFn: () => getSavedListItemNote(listRef, recordKey),
  });
  const [draft, setDraft] = useState<string | null>(null);
  const editing = draft !== null;
  useLayoutEffect(() => {
    if (!editing && !hidden && restoreEditFocus.current) {
      editButton.current?.focus({ preventScroll: true });
      restoreEditFocus.current = false;
    }
  }, [editing, hidden]);
  useEffect(() => {
    if (editing && !hidden) editor.current?.focus({ preventScroll: true });
  }, [editing, hidden]);
  const save = useMutation({
    mutationFn: (text: string) =>
      updateSavedListItemNote({
        list_ref: note.data!.list_key,
        record_key: recordKey,
        note: text.trim() ? text : null,
      }),
    onSuccess: (view) => {
      client.setQueryData(queryKey, view);
      finishEditing();
      void invalidateSavedListQueries(client, [listRef, view.list_key, view.slug]);
    },
  });
  if (hidden) return null;
  return (
    <section
      className="list-entry-note"
      aria-label="List entry note"
      aria-busy={save.isPending}
    >
      <div className="list-entry-note__header">
        <Typography.Text strong>Note</Typography.Text>
        {draft === null && note.data && (
          <Button
            ref={editButton}
            type="text"
            onClick={() => {
              save.reset();
              setDraft(note.data.note ?? "");
            }}
          >
            {note.data.note ? "Edit" : "Add note"}
          </Button>
        )}
      </div>
      {note.error && (
        <ErrorNotice
          error={note.error}
          title={note.data ? "Unable to refresh note" : "Unable to load note"}
          action={<Button onClick={() => void note.refetch()}>Retry</Button>}
        />
      )}
      {save.error && <ErrorNotice error={save.error} title="Unable to save note" />}
      {draft !== null ? (
        <>
          <Input.TextArea
            aria-label="Note"
            ref={editor}
            value={draft}
            autoSize={{ minRows: 2, maxRows: 6 }}
            disabled={save.isPending}
            onChange={(event) => setDraft(event.target.value)}
          />
          <Space>
            <Button
              type="primary"
              loading={save.isPending}
              onClick={() => save.mutate(draft)}
            >
              Save
            </Button>
            <Button
              disabled={save.isPending}
              onClick={() => {
                finishEditing();
                save.reset();
              }}
            >
              Cancel
            </Button>
          </Space>
        </>
      ) : note.data?.note ? (
        <div className="list-entry-note__text">{note.data.note}</div>
      ) : null}
    </section>
  );
}
