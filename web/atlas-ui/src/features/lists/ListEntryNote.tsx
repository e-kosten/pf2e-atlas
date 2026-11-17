import {
  MarkdownDocument,
  MarkdownExcerpt,
} from "../../shared/documents/MarkdownDocument";
import { MarkdownEditor } from "../../shared/documents/MarkdownEditor";
import { ReaderFooterContext } from "../../shared/records/ReaderFooterContext";
import { useSavedWritesPaused } from "../../shared/feedback/SavedWritesPaused";
import { useTabCloseGuard } from "../../app/DesktopTabContext";
import {
  skipToken,
  useMutation,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";
import { Alert, Button, Flex, Input, Space, Typography } from "antd";
import type { GetRef } from "antd";
import { ArrowLeft, Pencil } from "lucide-react";
import { useContext, useLayoutEffect, useRef, useState, type RefObject } from "react";
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
  const [pane, setPane] = useState<{ identity: string; mode: "read" | "edit" }>();
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
      draft,
    }: {
      request: UpdateSavedListItemNoteRequest;
      draft: Draft;
    }) => updateSavedListItemNote(draft.entry.listRef, draft.entry.recordKey, request),
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
  const paused = useSavedWritesPaused();
  const pending = Boolean(currentSave && save.isPending);
  const dirty = active !== null && active.text !== (active.expected ?? "");
  useTabCloseGuard(() => (save.isPending ? "pending" : dirty ? "dirty" : undefined));

  const identity =
    entry && !detached ? JSON.stringify([entry.listRef, entry.recordKey]) : undefined;
  const paneMode = pane?.identity === identity ? pane?.mode : undefined;
  const removedWhileReading = paneMode === "read" && removed && !active;
  const reading = paneMode === "read" && (!!note.data?.note || !!active) && !removed;
  const expanded = reading || (paneMode === "edit" && !!active);
  if (pane && !expanded && !removedWhileReading) setPane(undefined);

  function saveNote(draft: Draft, expected: string | null) {
    if (paused) return;
    save.mutate({
      draft,
      request: {
        note: { expected, value: noteValue(draft.text) },
      },
    });
  }

  return {
    entry,
    note,
    expanded,
    reading,
    removedWhileReading,
    openPane(mode: "read" | "edit") {
      if (identity) setPane({ identity, mode });
    },
    closePane() {
      setPane(undefined);
    },
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
      if (!dirty) {
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

function ListNoteReadout({
  source,
  expanded,
  overflow,
  onMeasure,
  openButton,
}: {
  source: string;
  expanded: boolean;
  overflow: boolean;
  onMeasure: (source: string | undefined) => void;
  openButton: RefObject<HTMLButtonElement | null>;
}) {
  const frame = useRef<HTMLDivElement>(null);
  const content = useRef<HTMLDivElement>(null);
  const focusOpener = useRef(false);
  useLayoutEffect(() => {
    if (expanded || !frame.current || !content.current) return;
    const box = frame.current;
    const contentElement = content.current;
    const measure = () => {
      if (!box.getClientRects().length) return;
      const budget = Number.parseFloat(getComputedStyle(box).maxBlockSize);
      const clipped = contentElement.getBoundingClientRect().height > budget + 0.5;
      if (clipped && contentElement.contains(box.ownerDocument.activeElement))
        focusOpener.current = true;
      onMeasure(clipped ? source : undefined);
    };
    measure();
    const observer =
      typeof ResizeObserver === "undefined" ? null : new ResizeObserver(measure);
    observer?.observe(box);
    observer?.observe(contentElement);
    return () => observer?.disconnect();
  }, [expanded, onMeasure, source]);

  const compact = overflow && !expanded;
  useLayoutEffect(() => {
    if (compact && focusOpener.current) {
      focusOpener.current = false;
      openButton.current?.focus({ preventScroll: true });
    }
  }, [compact, openButton]);

  return (
    <div
      className="list-entry-note__text list-entry-note__preview"
      data-compact={!expanded || undefined}
      ref={frame}
    >
      <div
        className="list-entry-note__source"
        data-measuring={compact || undefined}
        inert={compact}
        aria-hidden={compact || undefined}
        ref={content}
      >
        <MarkdownDocument source={source} linkFocusFallback={openButton} />
      </div>
      {compact && (
        <div className="list-entry-note__excerpt">
          <MarkdownExcerpt source={source} />
        </div>
      )}
    </div>
  );
}

export function ListEntryNoteEditor({
  state,
  hidden = false,
}: {
  state: ReturnType<typeof useListEntryNote>;
  hidden?: boolean;
}) {
  const footer = useContext(ReaderFooterContext);
  const paused = useSavedWritesPaused();
  const editButton = useRef<HTMLButtonElement>(null);
  const openButton = useRef<HTMLButtonElement>(null);
  const expandButton = useRef<HTMLButtonElement>(null);
  const [overflowSource, setOverflowSource] = useState<string>();
  const heading = useRef<HTMLHeadingElement>(null);
  const region = useRef<HTMLElement>(null);
  const lastEditorFocus = useRef<HTMLElement | null>(null);
  const saveButton = useRef<HTMLButtonElement>(null);
  const reviewTitle = useRef<HTMLSpanElement>(null);
  const last = useRef({
    visible: false,
    expanded: false,
    editing: false,
    detached: false,
  });
  const restoreEditFocus = useRef(false);
  const wasReviewOpen = useRef(false);
  const { draft, note, editor, pending, removed, expanded, reading, detached } = state;
  const visible = (footer?.active ?? true) && !hidden;
  const editing = !!draft;
  const readingFullNote = expanded && reading && !editing;
  const savedNote = note.data?.note;
  const overflow = !!savedNote && overflowSource === savedNote;
  const review = draft?.review;
  const hasConflict = !!review;
  const reviewOpen = !!review && !!draft?.reviewOpen && !removed && visible;

  useLayoutEffect(() => {
    const previous = last.current;
    last.current = { visible, expanded, editing, detached };
    if (previous.editing && !editing) restoreEditFocus.current = true;
    if (!visible) return;

    if (
      editing &&
      (!previous.editing || !previous.visible || (detached && !previous.detached))
    ) {
      if (reviewOpen) return;
      const target = previous.editing ? lastEditorFocus.current : null;
      if (target?.isConnected && target.getClientRects().length)
        target.focus({ preventScroll: true });
      else if (editor.current) editor.current.focus({ preventScroll: true });
      else region.current?.focus({ preventScroll: true });
      return;
    }
    if (restoreEditFocus.current && !editing) {
      restoreEditFocus.current = false;
      const target = editButton.current;
      (target && !target.disabled ? target : region.current)?.focus({
        preventScroll: true,
      });
      return;
    }
    if (!editing && expanded && (!previous.expanded || !previous.visible))
      heading.current?.focus({ preventScroll: true });
    if (previous.expanded && !expanded && !editing) {
      const frame = requestAnimationFrame(() => {
        const target = openButton.current ?? editButton.current;
        (target && !target.disabled ? target : region.current)?.focus({
          preventScroll: true,
        });
      });
      return () => cancelAnimationFrame(frame);
    }
  }, [detached, editing, editor, expanded, reviewOpen, visible]);

  useLayoutEffect(() => {
    const closed = wasReviewOpen.current && !reviewOpen;
    wasReviewOpen.current = reviewOpen;
    if (!closed || !visible) return;
    const frame = requestAnimationFrame(() => {
      const target = hasConflict
        ? (saveButton.current ?? editor.current)
        : (editor.current ?? editButton.current);
      target?.focus({ preventScroll: true });
    });
    return () => cancelAnimationFrame(frame);
  }, [editor, hasConflict, reviewOpen, visible]);

  if (hidden || !state.entry) return null;
  return (
    <section
      ref={region}
      className="list-entry-note"
      data-expanded={expanded || undefined}
      aria-label="List entry note"
      aria-busy={pending}
      tabIndex={-1}
      onFocusCapture={(event) => {
        if (editing && event.currentTarget.contains(event.target))
          lastEditorFocus.current = event.target;
      }}
    >
      {readingFullNote && (
        <Space wrap>
          <Button
            icon={<ArrowLeft size={16} aria-hidden="true" />}
            onClick={state.closePane}
          >
            Back to record
          </Button>
          <Button
            ref={editButton}
            icon={<Pencil size={16} aria-hidden="true" />}
            disabled={paused}
            onClick={state.begin}
          >
            Edit
          </Button>
        </Space>
      )}
      <div className="list-entry-note__header">
        {expanded ? (
          <h2 ref={heading} tabIndex={-1}>
            {draft?.title ?? note.data?.title}
          </h2>
        ) : (
          <Typography.Text strong>
            {state.detached && draft ? `Note for ${draft.title}` : "Note"}
          </Typography.Text>
        )}
        {editing && !reading && !state.detached && !removed && (
          <Button
            ref={expandButton}
            type="text"
            disabled={pending}
            onClick={() => {
              if (expanded) state.closePane();
              else {
                footer?.rememberReaderPosition();
                state.openPane("edit");
              }
              expandButton.current?.focus({ preventScroll: true });
            }}
          >
            {expanded ? "Collapse editor" : "Expand editor"}
          </Button>
        )}
        {!readingFullNote && !draft && note.data && !removed && (
          <Space wrap>
            {!expanded && overflow && (
              <Button
                ref={openButton}
                type="link"
                onClick={() => {
                  footer?.rememberReaderPosition();
                  state.openPane("read");
                }}
              >
                Read full note
              </Button>
            )}
            <Button
              ref={editButton}
              type="text"
              disabled={paused}
              onClick={state.begin}
            >
              {savedNote ? "Edit" : "Add note"}
            </Button>
          </Space>
        )}
      </div>
      <div
        className="list-entry-note__body"
        role={expanded && !editing ? "region" : undefined}
        aria-label={expanded && !editing ? "Note text" : undefined}
        tabIndex={expanded && !editing ? 0 : undefined}
      >
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
          <MarkdownEditor
            label="Note"
            editorRef={editor}
            value={draft.text}
            disabled={pending}
            readOnly={removed || paused}
            onChange={state.change}
            sizing={expanded ? "roomy" : "compact"}
          />
        ) : !removed && savedNote ? (
          <ListNoteReadout
            source={savedNote}
            expanded={expanded}
            overflow={overflow}
            onMeasure={setOverflowSource}
            openButton={openButton}
          />
        ) : null}
      </div>
      {draft && (
        <Space wrap>
          {!removed && (
            <Button
              ref={saveButton}
              type="primary"
              disabled={paused}
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
      )}
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
