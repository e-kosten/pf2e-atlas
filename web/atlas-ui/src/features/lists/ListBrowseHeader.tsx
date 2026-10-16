import { PrimaryToolbar } from "../../shared/ui/PrimaryToolbar";
import { Alert, Button, Spin, Typography } from "antd";
import { ArrowLeft, Pencil, Plus } from "lucide-react";
import { ListTagSummary } from "./ListTagSummary";
import type { SavedListSummaryView } from "../../generated/atlas";
import {
  listEditPath,
  navigateToAtlasRoute,
  shouldHandleAtlasRouteClick,
} from "../../app/routes";
import { FilterControls } from "../../shared/filters/FilterPanel";
import type { FilterPanelState } from "../../shared/filters/filterControls";

export function ListBrowseHeader({
  error,
  filterState,
  list,
  loading,
  onAddRecords,
  onRetry,
}: {
  error?: Error | null;
  filterState: FilterPanelState;
  list: SavedListSummaryView | undefined;
  loading: boolean;
  onAddRecords: () => void;
  onRetry: () => void;
}) {
  const errorNotice = error ? (
    <Alert
      type="error"
      showIcon
      title={list ? "Unable to refresh this list" : "Unable to load this list"}
      description={error.message}
      action={<Button onClick={onRetry}>Retry list</Button>}
    />
  ) : null;
  return (
    <section className="list-browse-header">
      <div className="list-browse-header__parent">
        <Typography.Link
          className="list-browse-header__parent-link"
          href="/lists"
          onClick={(event) => {
            if (!shouldHandleAtlasRouteClick(event)) return;
            event.preventDefault();
            navigateToAtlasRoute({ kind: "lists" });
          }}
        >
          <ArrowLeft size={14} aria-hidden="true" />
          Saved lists
        </Typography.Link>
      </div>
      {errorNotice}
      {loading ? (
        <div className="list-browse-header__identity" role="status">
          <Spin size="small" aria-label="Loading list" /> Loading list…
        </div>
      ) : !list ? (
        <div className="list-browse-header__identity">
          {!errorNotice && "List not found"}
        </div>
      ) : (
        <>
          <div className="list-browse-header__identity">
            <h2>{list.name}</h2>
            <PrimaryToolbar className="list-browse-header__actions">
              <Button type="primary" icon={<Plus size={16} />} onClick={onAddRecords}>
                Add records
              </Button>
              <Button
                href={listEditPath(list.slug)}
                icon={<Pencil size={16} />}
                onClick={(event) => {
                  if (!shouldHandleAtlasRouteClick(event)) return;
                  event.preventDefault();
                  navigateToAtlasRoute({ kind: "listEdit", slug: list.slug });
                }}
              >
                Edit list
              </Button>
            </PrimaryToolbar>
          </div>
          {(list.description || list.tags.length > 0) && (
            <div className="list-browse-header__metadata">
              {list.description && (
                <Typography.Paragraph
                  className="list-browse-header__description-text"
                  ellipsis={{
                    rows: 2,
                    expandable: "collapsible",
                    symbol: (expanded) => (expanded ? "Read less" : "Read more"),
                  }}
                >
                  {list.description}
                </Typography.Paragraph>
              )}
              <ListTagSummary tags={list.tags} lines={2} />
            </div>
          )}
          <FilterControls
            filterState={filterState}
            includeSearch
            layout="inline-toolbar"
          />
        </>
      )}
    </section>
  );
}
