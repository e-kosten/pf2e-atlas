import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import {
  isPrimaryActivation,
  navigateDestination,
  destinationHref,
} from "../../app/navigation";
import { listsDestination, listEditDestination } from "../../app/destinations";
import { PrimaryToolbar } from "../../shared/ui/PrimaryToolbar";
import { Button, Spin, Typography } from "antd";
import { ArrowLeft, Pencil, Plus } from "lucide-react";
import { ListTagSummary } from "./ListTagSummary";
import type { SavedListSummaryView } from "../../generated/atlas";
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
    <ErrorNotice
      error={error}
      title={list ? "Unable to refresh this list" : "Unable to load this list"}
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
            if (!isPrimaryActivation(event)) return;
            event.preventDefault();
            navigateDestination(listsDestination());
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
                href={destinationHref(listEditDestination({ slug: list.slug }))}
                icon={<Pencil size={16} />}
                onClick={(event) => {
                  if (!isPrimaryActivation(event)) return;
                  event.preventDefault();
                  navigateDestination(
                    listEditDestination({
                      slug: list.slug,
                    }),
                  );
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
