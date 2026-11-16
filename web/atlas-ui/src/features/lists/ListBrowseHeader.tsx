import { MarkdownDocument } from "../../shared/documents/MarkdownDocument";
import { AtlasDrawer } from "../../shared/ui/overlays/AtlasDrawer";
import { useAtlasRouter } from "../../app/router";
import { CopyEntityLink } from "../../app/CopyEntityLink";
import { DisplayChip } from "../../shared/ui/DisplayChip";
import { useSavedWritesPaused } from "../../shared/feedback/SavedWritesPaused";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { isPrimaryActivation, navigateDestination } from "../../app/navigation";
import { listsDestination } from "../../app/destinations";
import { WorkspaceContextBar } from "../../shared/layout/WorkspaceContextBar";
import { Button, Dropdown, Typography } from "antd";
import { useRef, useState } from "react";
import { LoadingStatus } from "../../shared/ui/LoadingStatus";
import { ArrowLeft, Ellipsis } from "lucide-react";
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
  const atlasRouter = useAtlasRouter();
  const actionsTrigger = useRef<HTMLButtonElement>(null);
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
              navigateDestination(listsDestination(), undefined, atlasRouter);
            }}
          >
            <ArrowLeft size={14} aria-hidden="true" />
            Saved lists
          </Typography.Link>
        }
        actions={
          list && (
            <>
              {(list.description || list.tags.length > 0) && (
                <ListAbout
                  list={list}
                  onEdit={onEdit}
                  editable={!paused && !editPreparing && !editing}
                />
              )}
              <CopyEntityLink
                primary={!editing}
                kind="list"
                identity={list.list_key}
                title={list.name}
                actions={(copy) => (
                  <Dropdown
                    trigger={["click"]}
                    menu={{
                      items: [
                        { key: "edit", label: "Edit list", disabled: paused },
                        ...(!editing && copy
                          ? [{ key: "copy", label: "Copy link" }]
                          : []),
                      ],
                      onClick: ({ key }) => {
                        const opener = actionsTrigger.current;
                        opener?.focus({ preventScroll: true });
                        if (key === "edit" && opener) onEdit(opener);
                        if (key === "copy") copy?.();
                      },
                    }}
                  >
                    <Button
                      ref={actionsTrigger}
                      type="text"
                      className="list-context-secondary"
                      aria-label="List actions"
                      aria-busy={editPreparing}
                      loading={editPreparing ? { delay: 200 } : false}
                      icon={<Ellipsis size={18} aria-hidden="true" />}
                    />
                  </Dropdown>
                )}
              />
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

function ListAbout({
  list,
  onEdit,
  editable,
}: {
  list: SavedListSummaryView;
  onEdit: (opener: HTMLElement) => void;
  editable: boolean;
}) {
  const [open, setOpen] = useState(false);
  const trigger = useRef<HTMLButtonElement>(null);
  return (
    <>
      <Button
        ref={trigger}
        type="text"
        className="list-context-secondary"
        aria-expanded={open}
        onClick={() => setOpen(true)}
      >
        About
      </Button>
      <AtlasDrawer
        open={open}
        onClose={() => setOpen(false)}
        title="About"
        size="min(760px, 100vw)"
        extra={
          editable && (
            <Button
              type="text"
              onClick={() => {
                setOpen(false);
                if (trigger.current) onEdit(trigger.current);
              }}
            >
              Edit list
            </Button>
          )
        }
      >
        <div className="list-about">
          {list.description && <MarkdownDocument source={list.description} />}
          {list.tags.length > 0 && (
            <div className="list-about__tags" role="group" aria-label="List tags">
              {list.tags.map((tag) => (
                <DisplayChip key={tag}>{tag}</DisplayChip>
              ))}
            </div>
          )}
        </div>
      </AtlasDrawer>
    </>
  );
}
