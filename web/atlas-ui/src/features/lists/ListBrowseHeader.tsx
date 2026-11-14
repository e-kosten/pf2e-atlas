import { CopyEntityLink } from "../../app/CopyEntityLink";
import { useSavedWritesPaused } from "../../shared/feedback/SavedWritesPaused";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { isPrimaryActivation, navigateDestination } from "../../app/navigation";
import { listsDestination } from "../../app/destinations";
import { WorkspaceContextBar } from "../../shared/layout/WorkspaceContextBar";
import { Button, Popover, Tag, Typography } from "antd";
import { useRef, useState } from "react";
import { LoadingStatus } from "../../shared/ui/LoadingStatus";
import { ArrowLeft } from "lucide-react";
import type { SavedListSummaryView } from "../../generated/atlas";
export function ListBrowseHeader({
  error,
  list,
  loading,
  onAddRecords,
  onEdit,
  editPreparing,
  editing,
  onRetry,
}: {
  error?: Error | null;
  list: SavedListSummaryView | undefined;
  loading: boolean;
  onAddRecords: () => void;
  onEdit: (opener: HTMLElement) => void;
  editPreparing?: boolean;
  editing?: boolean;
  onRetry: () => void;
}) {
  const paused = useSavedWritesPaused();
  const errorNotice = error ? (
    <ErrorNotice
      error={error}
      title={list ? "Unable to refresh this list" : "Unable to load this list"}
      action={<Button onClick={onRetry}>Retry list</Button>}
    />
  ) : null;
  return (
    <div className="list-browse-header">
      <WorkspaceContextBar
        title={list?.name}
        parentLink={
          <Typography.Link
            className="list-browse-header__parent-link"
            href="/lists"
            onClick={(event) => {
              if (!isPrimaryActivation(event)) return;
              event.preventDefault();
              navigateDestination(listsDestination());
            }}
          >
            <ArrowLeft size={14} aria-hidden="true" />
            Saved lists
          </Typography.Link>
        }
        actions={
          list && (
            <>
              {!editing && (
                <CopyEntityLink
                  primary
                  kind="list"
                  identity={list.list_key}
                  title={list.name}
                />
              )}
              {(list.description || list.tags.length > 0) && <ListAbout list={list} />}
              <Button
                type="text"
                className="list-context-secondary"
                aria-busy={editPreparing}
                disabled={paused}
                loading={editPreparing ? { delay: 200 } : false}
                onClick={(event) => {
                  event.currentTarget.focus({ preventScroll: true });
                  onEdit(event.currentTarget);
                }}
              >
                Edit list
              </Button>
              <Button type="primary" disabled={paused} onClick={onAddRecords}>
                Add records
              </Button>
            </>
          )
        }
      />
      {errorNotice}
      {loading && !list && <LoadingStatus label="Loading list…" />}
      {!loading && !list && !errorNotice && <div>List not found</div>}
    </div>
  );
}

function ListAbout({ list }: { list: SavedListSummaryView }) {
  const [open, setOpen] = useState(false);
  const trigger = useRef<HTMLButtonElement>(null);
  const panel = useRef<HTMLDivElement>(null);
  function handleEscape(event: React.KeyboardEvent) {
    if (!open || event.key !== "Escape" || event.defaultPrevented) return;
    event.preventDefault();
    setOpen(false);
    trigger.current?.focus({ preventScroll: true });
  }
  return (
    <Popover
      trigger="click"
      placement="bottomRight"
      open={open}
      onOpenChange={setOpen}
      afterOpenChange={(visible) => {
        if (visible && document.activeElement === trigger.current)
          panel.current?.focus({ preventScroll: true });
      }}
      title="About"
      content={
        // eslint-disable-next-line jsx-a11y/no-noninteractive-element-interactions -- Escape closes the focused disclosure.
        <div
          className="list-about"
          role="dialog"
          aria-label="About"
          tabIndex={-1}
          ref={panel}
          onKeyDown={handleEscape}
        >
          {list.description && <p>{list.description}</p>}
          {list.tags.length > 0 && (
            <div className="list-about__tags" role="group" aria-label="List tags">
              {list.tags.map((tag) => (
                <Tag key={tag}>{tag}</Tag>
              ))}
            </div>
          )}
        </div>
      }
    >
      <Button
        ref={trigger}
        type="text"
        className="list-context-secondary"
        aria-expanded={open}
        onKeyDown={handleEscape}
      >
        About
      </Button>
    </Popover>
  );
}
