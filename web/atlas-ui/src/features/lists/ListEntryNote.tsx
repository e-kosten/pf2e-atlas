import {
  skipToken,
  useMutation,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";
import { Alert, Button, Flex, Input, Space, Typography } from "antd";
import type { GetRef } from "antd";
import { useLayoutEffect, useRef, useState } from "react";
import { getSavedListItemNote, updateSavedListItemNote } from "../../api/atlasApi";
import { AtlasApiError } from "../../api/apiError";
import type {
  SavedListItemNoteView,
  UpdateSavedListItemNoteRequest,
} from "../../generated/atlas";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { savedEditConflictFields } from "../../shared/feedback/savedEditConflict";
import { AtlasModal } from "../../shared/ui/overlays/AtlasModal";
import { invalidateSavedListQueries } from "./savedListQueries";

type NoteEntry = {
  listRef: string;
  recordKey: string;
  initialNote?: SavedListItemNoteView;
  initialNoteUpdatedAt?: number;
};

type Review =
  { status: "loading" | "failed" } | { status: "ready"; saved: string | null };

type Draft = {
  token: object;
  entry: NoteEntry;
  title: string;
  expected: string | null;
  text: string;
  review?: Review;
  reviewOpen?: boolean;
};

function noteQuery(entry: NoteEntry) {
  return {
    queryKey: ["saved-list-item-note", entry.listRef, entry.recordKey],
    queryFn: () => getSavedListItemNote(entry.listRef, entry.recordKey),
  };
}

function itemWasRemoved(error: unknown) {
  return (
    error instanceof AtlasApiError &&
    error.appError?.code === "saved_list_item_not_found"
  );
}

function noteValue(text: string) {
  return text.trim() ? text : null;
}

export function useListEntryNote({
  selected,
  selectedRecordKey,
  listRouteRef,
}: {
  selected: NoteEntry | undefined;
  selectedRecordKey: string | null;
  listRouteRef?: string;
}) {
  const client = useQueryClient();
  const editor = useRef<GetRef<typeof Input.TextArea>>(null);
  const [draft, setDraft] = useState<Draft | null>(null);
  const [retainedDraft, setRetainedDraft] = useState<object>();
  const detached =
    selectedRecordKey === null && !!draft && draft.token === retainedDraft;
  const active =
    draft && (detached || selectedRecordKey === draft.entry.recordKey) ? draft : null;
  if (draft && !active) setDraft(null);
  const entry = active?.entry ?? selected;
  const note = useQuery({
    ...(entry
      ? noteQuery(entry)
      : ({ queryKey: ["saved-list-item-note"], queryFn: skipToken } as const)),
    initialData: entry?.initialNote,
    initialDataUpdatedAt: entry?.initialNoteUpdatedAt,
  });

  async function refreshReview(original: Draft, open = false) {
    const loading: Review = { status: "loading" };
    const query = noteQuery(original.entry);
    setDraft((current) =>
      current?.token === original.token ? { ...current, review: loading } : current,
    );
    let review: Review;
    try {
      await client.cancelQueries({ queryKey: query.queryKey, exact: true });
      const fresh = await client.fetchQuery({ ...query, staleTime: 0, retry: false });
      review = { status: "ready", saved: fresh.note };
    } catch {
      review = { status: "failed" };
    }
    setDraft((current) => {
      if (current?.token !== original.token || current.review !== loading)
        return current;
      if (review.status === "ready" && review.saved === noteValue(current.text))
        return null;
      return { ...current, review, reviewOpen: open || current.reviewOpen };
    });
  }

  const save = useMutation({
    mutationFn: ({
      request,
    }: {
      request: UpdateSavedListItemNoteRequest;
      draft: Draft;
    }) => updateSavedListItemNote(request),
    retry: false,
    onSuccess: (view, { draft: saved }) => {
      client.setQueryData(noteQuery(saved.entry).queryKey, view);
      setDraft((current) => (current?.token === saved.token ? null : current));
      void invalidateSavedListQueries(client, [
        saved.entry.listRef,
        view.list_key,
        view.slug,
        ...(listRouteRef ? [listRouteRef] : []),
      ]);
    },
    onError: async (error, { draft: saved }) => {
      if (savedEditConflictFields(error)?.includes("note"))
        await refreshReview(saved, true);
    },
  });
  const currentSave = active && save.variables?.draft.token === active.token;
  const saveError = currentSave ? save.error : null;
  const removed = itemWasRemoved(note.error) || itemWasRemoved(saveError);
  const pending = Boolean(currentSave && save.isPending);

  function saveNote(draft: Draft, expected: string | null) {
    save.mutate({
      draft,
      request: {
        list_ref: draft.entry.listRef,
        record_key: draft.entry.recordKey,
        note: { expected, value: noteValue(draft.text) },
      },
    });
  }

  return {
    entry,
    note,
    editor,
    draft: active,
    detached,
    removed,
    pending,
    saveError,
    begin() {
      if (!entry || !note.data || removed) return;
      save.reset();
      setDraft({
        token: {},
        entry,
        title: note.data.title,
        expected: note.data.note,
        text: note.data.note ?? "",
      });
    },
    change(text: string) {
      setDraft((current) => (current ? { ...current, text } : current));
    },
    retainAfterRemoval(recordKey: string) {
      if (active?.entry.recordKey === recordKey) setRetainedDraft(active.token);
    },
    finish() {
      setDraft(null);
      setRetainedDraft(undefined);
      save.reset();
    },
    submit() {
      if (!active || active.review || removed || pending) return;
      if (active.text === (active.expected ?? "")) {
        setDraft(null);
        return;
      }
      saveNote(active, active.expected);
    },
    retryReview() {
      if (active) void refreshReview(active);
    },
    setReviewOpen(reviewOpen: boolean) {
      if (pending) return;
      setDraft((current) => (current?.review ? { ...current, reviewOpen } : current));
    },
    overwrite() {
      if (active?.review?.status !== "ready" || removed || pending) return;
      saveNote(active, active.review.saved);
    },
  };
}

export function ListEntryNoteEditor({
  state,
  hidden = false,
}: {
  state: ReturnType<typeof useListEntryNote>;
  hidden?: boolean;
}) {
  const editButton = useRef<HTMLButtonElement>(null);
  const saveButton = useRef<HTMLButtonElement>(null);
  const reviewTitle = useRef<HTMLSpanElement>(null);
  const wasEditing = useRef(false);
  const wasReviewOpen = useRef(false);
  const restoreEditFocus = useRef(false);
  const { draft, note, editor, pending, removed } = state;
  const editing = !!draft;
  const review = draft?.review;
  const hasConflict = !!review;
  const reviewOpen = !!review && !!draft?.reviewOpen && !removed && !hidden;

  useLayoutEffect(() => {
    if (wasEditing.current && !editing) restoreEditFocus.current = true;
    wasEditing.current = editing;
    if (hidden) return;
    if (editing) editor.current?.focus({ preventScroll: true });
    if (!editing && restoreEditFocus.current) {
      editButton.current?.focus({ preventScroll: true });
      restoreEditFocus.current = false;
    }
  }, [editing, editor, hidden]);

  useLayoutEffect(() => {
    const closed = wasReviewOpen.current && !reviewOpen;
    wasReviewOpen.current = reviewOpen;
    if (!closed || hidden) return;
    const frame = requestAnimationFrame(() => {
      const target = hasConflict
        ? (saveButton.current ?? editor.current)
        : (editor.current ?? editButton.current);
      target?.focus({ preventScroll: true });
    });
    return () => cancelAnimationFrame(frame);
  }, [editor, hasConflict, hidden, reviewOpen]);

  if (hidden || !state.entry) return null;
  return (
    <section
      className="list-entry-note"
      aria-label="List entry note"
      aria-busy={pending}
    >
      <div className="list-entry-note__header">
        <Typography.Text strong>
          {state.detached && draft ? `Note for ${draft.title}` : "Note"}
        </Typography.Text>
        {!draft && note.data && !removed && (
          <Button ref={editButton} type="text" onClick={state.begin}>
            {note.data.note ? "Edit" : "Add note"}
          </Button>
        )}
      </div>
      {removed ? (
        <Alert
          type="warning"
          title="Item removed from this list"
          description={
            draft ? "You can copy your draft before discarding it." : undefined
          }
        />
      ) : !review && note.error ? (
        <ErrorNotice
          error={note.error}
          title={note.data ? "Unable to refresh note" : "Unable to load note"}
          action={<Button onClick={() => void note.refetch()}>Retry</Button>}
        />
      ) : null}
      {!removed && !review && state.saveError && (
        <ErrorNotice error={state.saveError} title="Unable to save note" />
      )}
      {draft ? (
        <>
          <Input.TextArea
            aria-label="Note"
            ref={editor}
            value={draft.text}
            autoSize={{ minRows: 2, maxRows: 6 }}
            disabled={pending}
            readOnly={removed}
            onChange={(event) => state.change(event.target.value)}
          />
          <Space wrap>
            {!removed && (
              <Button
                ref={saveButton}
                type="primary"
                loading={pending}
                onClick={review ? () => state.setReviewOpen(true) : state.submit}
              >
                {review ? "Review conflict" : "Save"}
              </Button>
            )}
            <Button disabled={pending} onClick={state.finish}>
              {removed ? "Discard draft" : "Cancel"}
            </Button>
          </Space>
        </>
      ) : !removed && note.data?.note ? (
        <div className="list-entry-note__text">{note.data.note}</div>
      ) : null}
      <AtlasModal
        open={reviewOpen}
        title={
          <span ref={reviewTitle} tabIndex={-1}>
            Review note conflict
          </span>
        }
        mask={{ closable: false }}
        closable={!pending}
        focusable={{ focusTriggerAfterClose: false }}
        onCancel={() => state.setReviewOpen(false)}
        afterOpenChange={(open) => {
          if (open) reviewTitle.current?.focus({ preventScroll: true });
        }}
        footer={
          <Space wrap>
            <Button disabled={pending} onClick={() => state.setReviewOpen(false)}>
              Back to note
            </Button>
            {review?.status === "failed" && (
              <Button
                type="primary"
                onClick={() => {
                  reviewTitle.current?.focus({ preventScroll: true });
                  state.retryReview();
                }}
              >
                Retry loading saved note
              </Button>
            )}
            {review?.status === "ready" && (
              <>
                <Button disabled={pending} onClick={state.finish}>
                  Use saved
                </Button>
                <Button
                  type="primary"
                  loading={pending}
                  onClick={() => {
                    reviewTitle.current?.focus({ preventScroll: true });
                    state.overwrite();
                  }}
                >
                  Overwrite
                </Button>
              </>
            )}
          </Space>
        }
      >
        {review?.status === "ready" ? (
          <Flex
            vertical
            gap="middle"
            tabIndex={0}
            role="region"
            aria-label="Note versions"
          >
            <Typography.Text>
              The saved note changed while you were editing.
            </Typography.Text>
            {state.saveError && !savedEditConflictFields(state.saveError) && (
              <ErrorNotice error={state.saveError} title="Unable to save note" />
            )}
            <div>
              <Typography.Text strong>Latest saved note</Typography.Text>
              <div className="list-entry-note__text">{review.saved ?? "No note"}</div>
            </div>
            <div>
              <Typography.Text strong>Your version</Typography.Text>
              <div className="list-entry-note__text">
                {noteValue(draft?.text ?? "") ?? "No note"}
              </div>
            </div>
          </Flex>
        ) : review ? (
          <Typography.Text role="status">
            {review.status === "loading"
              ? "Loading the latest saved note…"
              : "Could not load the latest saved note."}
          </Typography.Text>
        ) : null}
      </AtlasModal>
    </section>
  );
}
